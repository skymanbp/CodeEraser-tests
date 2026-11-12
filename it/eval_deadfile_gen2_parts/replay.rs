//! The replay (BARS-GEN2 section 2): one corpus's first-parent history
//! read into the model — every tree from the window's first parent to
//! the tip judged by this build's `ce deadcode`, every step read off
//! `git diff -M -l0 --name-status -z` — on a CLONE of the registered
//! clone (the registered clone is read-only), checked out commit by
//! commit. The corpus is the copy's directory name. Nothing here counts;
//! model.rs reads the events and fates, readings.rs the shares.

use super::gitz::{Listing, case_clash, checkout, clone_copy, ls_tree, step};
use super::judge::settle;
use super::model::{Dead, FUTURE, History, Step, Tree, WINDOW};
use crate::common::first_parent;
use crate::common::gitio::git_ok;
use codeeraser::scan::lang::Lang;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Every chain index divisible by this, and the tip, is judged a second
/// time on a cold index (G13).
pub const COLD_EVERY: usize = 100;

/// The product's word for a documentation file: a file of a document
/// language (Markdown, HTML) or plain prose.
pub fn document(path: &str) -> bool {
    matches!(
        Lang::from_path(Path::new(path)),
        Some(Lang::Markdown | Lang::Html | Lang::Text)
    )
}

/// The dead map of one `ce deadcode --format json` document: a root is
/// an `unref_*` verdict (no reference from any file of the tree).
pub fn dead_of(doc: &Value) -> BTreeMap<String, Dead> {
    doc["dead"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|r| {
            let path = r["name"].as_str().expect("name").to_string();
            let root = r["verdict"]
                .as_str()
                .is_some_and(|v| v.starts_with("unref"));
            let d = Dead {
                root,
                document: document(&path),
            };
            (path, d)
        })
        .collect()
}

/// G11: every dead row names a file of the tree (only file nodes enter
/// `dead`; a mismatch is an instrument defect, never an event).
pub fn check_in_files(
    corpus: &str,
    sha: &str,
    dead: &BTreeMap<String, Dead>,
    files: &BTreeSet<String>,
) {
    if let Some(p) = dead.keys().find(|p| !files.contains(*p)) {
        panic!("{corpus} {sha}: dead row {p:?} names no file of the tree");
    }
}

/// The tree a judged document gives over a listing: the symbolic links'
/// rows dropped and counted (G16), every other row a file (G11).
pub fn judged_tree(corpus: &str, sha: &str, doc: &Value, l: &Listing) -> Tree {
    let (links, dead): (BTreeMap<_, _>, BTreeMap<_, _>) = dead_of(doc)
        .into_iter()
        .partition(|(p, _)| l.symlinks.contains(p));
    check_in_files(corpus, sha, &dead, &l.files);
    Tree {
        files: l.files.clone(),
        dead,
        judged: true,
        symlink_rows: links.len(),
        ..Tree::default()
    }
}

/// The checked-out tree judged by this build, a refusal asked again (R2).
fn judge_here(copy: &Path, corpus: &str, sha: &str, l: &Listing) -> Tree {
    let s = settle(copy)
        .unwrap_or_else(|run| panic!("{corpus} {sha}: not a statement about the tree: {run}"));
    let t = match s.read {
        Ok(doc) => judged_tree(corpus, sha, &doc, l),
        Err(why) => unjudged(l, why),
    };
    Tree {
        retried: s.retried,
        ..t
    }
}

fn unjudged(l: &Listing, why: String) -> Tree {
    Tree {
        files: l.files.clone(),
        why: Some(why),
        ..Tree::default()
    }
}

/// One commit's tree: checked out in the copy, judged by this build; a
/// tree two of whose paths differ only by case (R1c), or that git cannot
/// check out here (G17), is unjudged.
fn tree(copy: &Path, corpus: &str, sha: &str) -> Tree {
    let l = ls_tree(copy, sha);
    if let Some(why) = case_clash(&l) {
        return unjudged(&l, why);
    }
    if let Err(why) = checkout(copy, sha) {
        return unjudged(&l, why);
    }
    git_ok(copy, &["clean", "-q", "-ffdx", "-e", ".ce"]);
    judge_here(copy, corpus, sha, &l)
}

/// G13: the tree judged again after `.ce` is removed must read the same;
/// else both maps are written beside the copy and the run stops.
fn cold_check(copy: &Path, corpus: &str, sha: &str, warm: &Tree) {
    let index = copy.join(".ce");
    if index.exists() {
        std::fs::remove_dir_all(&index).expect("remove the warm index");
    }
    let cold = judge_here(copy, corpus, sha, &ls_tree(copy, sha));
    if (&cold.dead, cold.judged) == (&warm.dead, warm.judged) {
        return;
    }
    let map = |t: &Tree| -> Value {
        json!({"judged": t.judged, "why": t.why,
               "dead": t.dead.iter().map(|(p, d)| json!([p, d.root, d.document])).collect::<Vec<_>>()})
    };
    let out = copy.with_file_name(format!("{corpus}-warm-cold-{sha}.json"));
    let doc = json!({"corpus": corpus, "commit": sha, "warm": map(warm), "cold": map(&cold)});
    std::fs::write(&out, serde_json::to_string_pretty(&doc).expect("doc")).expect("write");
    panic!(
        "{corpus} {sha}: the warm index and a cold one judge differently ({})",
        out.display()
    );
}

/// The first-parent chain the replay reads, oldest first: from the
/// window's first parent to the tip.
pub fn chain(repo: &Path, tip: &str) -> Vec<String> {
    let mut all = first_parent(repo, tip);
    all.reverse();
    let take = (WINDOW + FUTURE + 1).min(all.len());
    all.split_off(all.len() - take)
}

/// The corpus's history: a fresh copy of the registered clone, every
/// tree of the chain and every step between.
pub fn history(clone: &Path, copy: &Path, tip: &str) -> History {
    let corpus = copy
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .expect("the copy is named for its corpus");
    clone_copy(clone, copy);
    let shas = chain(copy, tip);
    let mut h = History {
        shas: shas.clone(),
        ..History::default()
    };
    for (i, sha) in shas.iter().enumerate() {
        h.steps.push(if i == 0 {
            Step::default()
        } else {
            step(copy, &shas[i - 1], sha)
        });
        let t = tree(copy, &corpus, sha);
        if t.why
            .as_deref()
            .is_none_or(|w| !w.starts_with("checkout: "))
            && (i % COLD_EVERY == 0 || i + 1 == shas.len())
        {
            cold_check(copy, &corpus, sha, &t);
        }
        h.trees.push(t);
    }
    h
}
