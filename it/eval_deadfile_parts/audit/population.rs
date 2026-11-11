//! Population B (BARS addendum 1 A): the audited rows the product still
//! reports dead once every passing §3 convention is declared an entry.
//! Per corpus a COPY of the clone (never the shared clone) and `ce
//! deadcode` of this build on it, twice: first with the corpus's own
//! `ce.toml` (it must give back the frozen slice's dead rows exactly, or
//! the copy is not the tree the exam froze), then with every file the
//! passing position rules match — the same `conventions::matches` §3
//! reads — appended to `[graph] entry_globs` as a literal. The rows that
//! drop must be exactly the dead rows among the matched files and what
//! they reach along the kept arcs; a corpus where they are not stops the
//! run. One input: the position rules; no glob list is written by hand.

use crate::common;
use crate::eval_deadfile_parts::{SLICE, corpora, doc_path};
use crate::eval_lang_parts::generate::env;
use crate::{common::ledger::out_file, eval_support::load};
use codeeraser::graph::deadcode::{self, Advisory, GraphWire};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// A copy of a clone, the index directory left behind (the copy builds
/// its own); an earlier copy is removed first.
pub fn copy_tree(from: &Path, to: &Path) {
    if to.exists() {
        std::fs::remove_dir_all(to).expect("remove the earlier copy");
    }
    std::fs::create_dir_all(to).expect("copy dir");
    for e in std::fs::read_dir(from).expect("clone").flatten() {
        let (src, dst) = (e.path(), to.join(e.file_name()));
        if e.file_name() == ".ce" {
            continue;
        }
        if e.file_type().expect("type").is_dir() {
            copy_tree(&src, &dst);
        } else {
            std::fs::copy(&src, &dst).expect("copy file");
        }
    }
}

/// A root-relative path as an `entry_globs` literal: anchored at the
/// root, every character the ce.toml dialect (gitignore globs through the
/// `ignore` crate, scan/globs.rs) reads as special written as a class.
pub fn literal(path: &str) -> String {
    let mut out = String::from("/");
    for ch in path.chars() {
        if "*?[]{}".contains(ch) {
            out.extend(['[', ch, ']']);
        } else {
            out.push(ch);
        }
    }
    out
}

/// The copy's `ce.toml`: the corpus's own text with `globs` appended to
/// its `[graph] entry_globs` (its own text back when there are none).
fn declare(copy: &Path, own: Option<&str>, globs: &[String]) {
    let path = copy.join("ce.toml");
    if globs.is_empty() {
        match own {
            Some(text) => std::fs::write(&path, text).expect("ce.toml"),
            None => drop(std::fs::remove_file(&path)),
        }
        return;
    }
    let mut table: toml::Table =
        own.map_or_else(toml::Table::new, |t| t.parse().expect("corpus ce.toml"));
    let graph = table
        .entry("graph")
        .or_insert(toml::Value::Table(toml::Table::new()));
    let graph = graph.as_table_mut().expect("[graph] is a table");
    let list = graph
        .entry("entry_globs")
        .or_insert(toml::Value::Array(Vec::new()));
    let list = list.as_array_mut().expect("entry_globs is an array");
    list.extend(globs.iter().map(|g| toml::Value::String(g.clone())));
    std::fs::write(&path, toml::to_string(&table).expect("toml")).expect("ce.toml");
}

/// The dead files `ce deadcode` (this build) reports on the copy.
fn dead_on(copy: &Path, own: Option<&str>, globs: &[String]) -> BTreeSet<String> {
    declare(copy, own, globs);
    let out = common::ce_command(copy, &[])
        .args(["deadcode", "--format", "json", "."])
        .output()
        .expect("run ce deadcode");
    let doc: Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "{}: ce deadcode said no JSON ({e}): {}",
            copy.display(),
            String::from_utf8_lossy(&out.stderr)
        )
    });
    assert!(
        doc["degraded"].is_null(),
        "{}: degraded {}",
        copy.display(),
        doc["degraded"]
    );
    named(&doc["dead"], "name")
}

/// The strings one key holds across a list of objects.
fn named(list: &Value, key: &str) -> BTreeSet<String> {
    let list = list
        .as_array()
        .unwrap_or_else(|| panic!("no list for `{key}`"));
    list.iter()
        .map(|x| {
            x[key]
                .as_str()
                .unwrap_or_else(|| panic!("no `{key}`"))
                .to_string()
        })
        .collect()
}

/// A corpus's frozen dead rows (the slice the exam drew from).
pub fn frozen_dead(corpus: &str) -> BTreeSet<String> {
    let home = doc_path(SLICE.family, corpus);
    named(&load(&out_file("CE_DEADFILE_OUT", &home))["files"], "path")
}

/// The corpora a run covers: `CE_DEADFILE_CORPORA` when set, else every one.
pub fn chosen() -> Vec<&'static str> {
    let only = std::env::var("CE_DEADFILE_CORPORA").ok();
    let only: Option<Vec<String>> = only.map(|s| s.split(',').map(str::to_string).collect());
    let all = corpora().into_iter().map(|(n, _)| n);
    all.filter(|n| only.as_ref().is_none_or(|o| o.iter().any(|x| x == n)))
        .collect()
}

/// The kept arcs as the core reads them (CE.Graph.Build: rung <= minRung
/// 5, the asset and ref-def kinds 3 and 5 inert): every file node the
/// seeds reach, the seeds included.
fn reached(w: &GraphWire, seeds: BTreeSet<i64>) -> BTreeSet<i64> {
    let (mut seen, mut todo) = (seeds.clone(), seeds.into_iter().collect::<Vec<_>>());
    while let Some(n) = todo.pop() {
        for e in w
            .edges
            .iter()
            .filter(|e| e[0] == n && e[3] <= 5 && e[2] != 3 && e[2] != 5)
        {
            if seen.insert(e[1]) {
                todo.push(e[1]);
            }
        }
    }
    seen
}

/// One corpus under one set of entries: the dead rows before and after,
/// the matched files (all, and the dead among them) and the dead rows the
/// matched files reach (themselves included).
pub struct Moved {
    pub before: BTreeSet<String>,
    pub after: BTreeSet<String>,
    pub matched: Vec<String>,
    pub matched_dead: BTreeSet<String>,
    pub reach_dead: BTreeSet<String>,
}

/// A fresh copy of a corpus and its control run with the corpus's own
/// `ce.toml`: (copy, own text, dead rows), the rows the frozen slice's.
fn controlled(corpus: &str) -> (PathBuf, Option<String>, BTreeSet<String>) {
    let (from, copy) = (
        Path::new(&env("CE_DEADFILE_ROOT")).join(corpus),
        Path::new(&env("CE_DEADFILE_COPIES")).join(corpus),
    );
    copy_tree(&from, &copy);
    let own = std::fs::read_to_string(copy.join("ce.toml")).ok();
    let before = dead_on(&copy, own.as_deref(), &[]);
    assert_eq!(
        before,
        frozen_dead(corpus),
        "{corpus}: the copy is not the frozen tree"
    );
    (copy, own, before)
}

/// The run's two refusals: no entry makes a file dead, and the rows that
/// drop are exactly the matched rows and their reach.
fn settled(corpus: &str, m: &Moved) {
    let new: Vec<&String> = m.after.difference(&m.before).collect();
    assert!(
        new.is_empty(),
        "{corpus}: an entry made files dead: {new:?}"
    );
    let dropped: BTreeSet<String> = m.before.difference(&m.after).cloned().collect();
    assert_eq!(
        dropped, m.reach_dead,
        "{corpus}: dropped rows are not the matched rows plus their reach"
    );
}

/// Population B of one corpus: every graph file node `pick` accepts
/// becomes a literal entry; the run stops unless the rows dropped are
/// exactly the dead rows among the matched files and their reach.
pub fn population_b(corpus: &str, pick: &dyn Fn(&str) -> bool) -> Moved {
    let (copy, own, before) = controlled(corpus);
    let (idx, db) = codeeraser::dedup::refreshed_index(&copy, None).expect("index");
    let w = deadcode::wire_of(&copy, &idx, &db, Advisory::No).expect("wire");
    drop(idx);
    let all = deadcode::file_nodes(&w);
    let files: Vec<&(i64, &str)> = all.iter().filter(|(_, p)| pick(p)).collect();
    let matched: Vec<String> = files.iter().map(|(_, p)| p.to_string()).collect();
    let globs: Vec<String> = matched.iter().map(|p| literal(p)).collect();
    let after = if globs.is_empty() {
        before.clone()
    } else {
        dead_on(&copy, own.as_deref(), &globs)
    };
    let dead_of = |set: &BTreeSet<i64>| -> BTreeSet<String> {
        all.iter()
            .filter(|(i, p)| set.contains(i) && before.contains(*p))
            .map(|(_, p)| p.to_string())
            .collect()
    };
    let seeds: BTreeSet<i64> = files.iter().map(|(i, _)| *i).collect();
    let (matched_dead, reach_dead) = (dead_of(&seeds), dead_of(&reached(&w, seeds)));
    let m = Moved {
        before,
        after,
        matched,
        matched_dead,
        reach_dead,
    };
    settled(corpus, &m);
    m
}

/// One corpus's line: dead before / after, matched, reached beyond.
pub fn line(corpus: &str, m: &Moved) -> String {
    let beyond: Vec<&String> = m.reach_dead.difference(&m.matched_dead).collect();
    format!(
        "{corpus}: dead {} -> {}; matched {} files ({} dead); reached beyond matched {}{}",
        m.before.len(),
        m.after.len(),
        m.matched.len(),
        m.matched_dead.len(),
        beyond.len(),
        if beyond.is_empty() {
            String::new()
        } else {
            format!(" {beyond:?}")
        }
    )
}

/// The producer on a synthetic passing set (phase 2B part 1 c): files
/// named `<CE_DEADFILE_PROOF_NAME>` (default `package-info.java`, a
/// `basename` rule read by the candidate table's own matcher) on the
/// chosen corpora.
///   CE_DEADFILE_ROOT=<clones> CE_DEADFILE_COPIES=<scratch dir> CE_DEADFILE_CORPORA=gson,jsoup
///   cargo test --test it -- --ignored eval_deadfile_parts::audit::population::deadfile_still_dead_proof --nocapture
#[test]
#[ignore = "copies the corpus clones and runs ce deadcode on the copies"]
fn deadfile_still_dead_proof() {
    let name = std::env::var("CE_DEADFILE_PROOF_NAME").unwrap_or("package-info.java".into());
    let rule = json!({"kind": "basename", "names": [name]});
    for corpus in chosen() {
        let m = population_b(corpus, &|p| super::conventions::matches(&name, &rule, p));
        println!("{}", line(corpus, &m));
    }
}
