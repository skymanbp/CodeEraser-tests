//! The language exams' generators — `#[ignore]`, because they read the
//! pinned corpus clone under `.ce-eval/corpora/<name>` (EXAMS pins the
//! tip; corpus::pinned_root refuses any other tree). Each writes one
//! frozen doc and refuses to overwrite it: a re-freeze is a new tip and
//! a named entry in docs/EVAL-SET-LANGS.md, never a silent re-run.
//!   CE_LANG_CORPUS=gson cargo test --test it -- --ignored eval_lang_parts::generate::lang_slice --nocapture
//!   CE_LANG=java cargo test --test it -- --ignored eval_lang_parts::generate::lang_sample --nocapture

use super::{Exam, SAMPLE_SCHEMA, SLICE_SCHEMA};
use crate::eval_support::{
    eval_doc, generated_from, git_in, lang_of, load, pinned_root, site_row, site_summary,
};
use codeeraser::graph::sites::detect;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const SLICE_METHOD: &str = "site universe of one pinned corpus for one v2.30 language: every \
    file whose extension is in scope, inventoried with the sha256 of the text the detector saw \
    and its resolution-free per-kind site counts (graph::sites — grammar kind tables and \
    file-local facts only, no path ever consulted). Frozen before the language's ladder exists; \
    the falsification constants are pre-registered here, before any measurement.";

const SAMPLE_METHOD: &str = "hash-ranked stratified draw over the language's frozen site \
    universes, reconstructed file by file against their frozen rows. rank id = \
    sha256(domain|corpus|commit|path|line|nth|kind|spec), the M5-2 payload. Primaries: per \
    kind min(min_per_kind, pool), the seats left to total by largest remainder over each \
    kind's remaining pool, top of the site-domain rank per kind; rows stored in audit-domain \
    order. Backups: per kind, the audit-domain rank over the unpicked rest — the audit walks \
    them only for an unanswerable primary of the same kind.";

fn env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{key} names what to freeze"))
}

fn corpus_repo(name: &str, tip: &str) -> String {
    pinned_root(name, tip).to_string_lossy().into_owned()
}

fn blob(repo: &str, tip: &str, path: &str) -> String {
    git_in(Some(repo), &["show", &format!("{tip}:{path}")])
}

/// Write a frozen doc once.
fn freeze(stem: &str, doc: &Value) {
    let path = eval_doc(stem);
    assert!(
        !std::path::Path::new(&path).exists(),
        "{path} is frozen: a re-freeze is a new tip and a named ledger entry"
    );
    let text = serde_json::to_string_pretty(doc).expect("json") + "\n";
    std::fs::write(&path, text).expect(&path);
    println!("{path} written");
}

/// The pinned tree's in-scope files as universe rows, plus the tally
/// of what the scope left out.
fn walk(exam: &Exam, repo: &str, tip: &str) -> (Vec<Value>, BTreeMap<&'static str, u64>) {
    // -z: unquoted non-ASCII paths; --full-tree: root-relative paths
    // whatever the cwd (the M5-2 walker's two lessons)
    let listing = git_in(
        Some(repo),
        &["ls-tree", "-r", "--full-tree", "--name-only", "-z", tip],
    );
    let (mut files, mut excluded) = (Vec::new(), BTreeMap::new());
    for path in listing.split('\0').filter(|p| !p.is_empty()) {
        let ext = path.rsplit_once('.').map_or("", |(_, e)| e);
        if exam.exts.contains(&ext) {
            files.push(site_row(path, exam.lang, &blob(repo, tip, path)));
        } else {
            *excluded.entry("other_extension").or_insert(0) += 1;
        }
    }
    files.sort_by(|a: &Value, b: &Value| a["path"].as_str().cmp(&b["path"].as_str()));
    (files, excluded)
}

#[test]
#[ignore = "reads the pinned corpus clone"]
fn lang_slice() {
    let name = env("CE_LANG_CORPUS");
    let (exam, tip) = super::exam_of_corpus(&name);
    let (files, excluded) = walk(exam, &corpus_repo(&name, tip), tip);
    freeze(
        &format!("lang-slice-{name}"),
        &json!({
            "schema": SLICE_SCHEMA,
            "corpus": {"name": name, "tip": tip, "lang": exam.lang},
            "scope": super::scope(exam),
            "constants": super::slice_constants(),
            "generated_from": generated_from(),
            "method": SLICE_METHOD,
            "summary": site_summary(&files),
            "excluded": excluded,
            "files": files,
        }),
    );
}

/// Every site of one frozen universe, re-detected at the pinned tip;
/// each file must reproduce its frozen row first — the pool equals the
/// frozen universe by checked reconstruction, not by trust.
fn corpus_pool(exam: &Exam, name: &str, tip: &str, slice: &Value) -> Vec<Value> {
    let repo = corpus_repo(name, tip);
    let mut pool = Vec::new();
    for row in slice["files"].as_array().expect("files") {
        let path = row["path"].as_str().expect("path");
        let text = blob(&repo, tip, path);
        assert_eq!(
            &site_row(path, exam.lang, &text),
            row,
            "{name}/{path}: not the frozen row"
        );
        for s in detect(&text, lang_of(exam.lang)) {
            pool.push(json!({
                "corpus": name, "commit": tip, "path": path, "line": s.line,
                "nth": s.nth, "kind": s.kind, "lang": exam.lang, "spec": s.spec,
            }));
        }
    }
    pool
}

#[test]
#[ignore = "reads every pinned corpus clone of the language"]
fn lang_sample() {
    let exam = super::exam(&env("CE_LANG"));
    let (mut pool, mut sources) = (Vec::new(), Vec::new());
    for (name, tip) in exam.corpora {
        let slice = load(&eval_doc(&format!("lang-slice-{name}")));
        pool.extend(corpus_pool(exam, name, tip, &slice));
        sources.push(super::source_row(name, &slice));
    }
    let draw = super::draw(&pool);
    freeze(
        &format!("lang-sample-{}", exam.lang),
        &json!({
            "schema": SAMPLE_SCHEMA,
            "lang": exam.lang,
            "constants": super::sample_constants(),
            "generated_from": generated_from(),
            "method": SAMPLE_METHOD,
            "sources": sources,
            "allocation": draw.allocation,
            "rows": draw.primary,
            "backups": draw.backups,
        }),
    );
}
