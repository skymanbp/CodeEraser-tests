//! The pinned tree of an exam corpus whose truths reach any tracked
//! file (Reach::Tree — HTML: a page fetches whatever the site serves,
//! not only pages): every path `git ls-tree` lists at the pinned tip,
//! frozen as `lang-tree-<corpus>-v<g>.json` beside the universe. It
//! binds such an audit's truths and scope gaps to real files without a
//! clone (review.rs), and the gate holds it to the frozen universe:
//! the universe's files are tree paths, and the slice's own tallies of
//! what it left out — another extension, a walk refusal — count the
//! rest, class by class. The precision generator, the one reader with
//! the clone in hand, re-derives it before scoring (assert_frozen_tree).
//!   CE_LANG_CORPUS=learning-area cargo test --test it -- --ignored eval_lang_parts::tree::lang_tree --nocapture

use super::generate::{corpus_repo, env, freeze, tree_paths};
use super::walk::in_scope;
use super::{Docs, Exam, Reach, SLICES};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub const TREES: Docs = Docs("lang-tree");
pub const TREE_SCHEMA: &str = "ce.eval-lang-tree/1.0.0";

const TREE_METHOD: &str = "every path git ls-tree -r --full-tree lists at the pinned tip, \
    in byte order: the files an audit truth may name when the exam's references reach any \
    tracked file (a stylesheet, an image, a script, a page). A function of the tip alone - \
    no product code, no judgment.";

/// A frozen universe's files.
pub fn universe(slice: &Value) -> BTreeSet<String> {
    slice["files"]
        .as_array()
        .expect("files")
        .iter()
        .map(|f| f["path"].as_str().expect("path").to_string())
        .collect()
}

/// A frozen tree's paths, in its order.
fn paths(doc: &Value) -> Vec<&str> {
    doc["paths"]
        .as_array()
        .expect("paths")
        .iter()
        .map(|p| p.as_str().expect("path"))
        .collect()
}

/// The paths a truth of the exam may name in one corpus: the frozen
/// universe's files, or — for a Reach::Tree exam — its pinned tree.
pub fn targets(exam: &Exam, corpus: &str, universe: &BTreeSet<String>) -> BTreeSet<String> {
    match exam.reach {
        Reach::Universe => universe.clone(),
        Reach::Tree => paths(&TREES.load(exam, corpus))
            .into_iter()
            .map(str::to_string)
            .collect(),
    }
}

/// One frozen tree against its exam and its frozen universe: the
/// envelope, the paths strictly ascending, every universe file among
/// them, and the rest counted by the slice's own tallies — a path of
/// another extension is `other_extension`, one of the exam's own that
/// the universe does not hold is `walk_refused`.
pub fn verify_tree(exam: &Exam, corpus: &str, doc: &Value) {
    exam.assert_envelope(corpus, doc, TREE_SCHEMA);
    assert_eq!(doc["method"], json!(TREE_METHOD), "{corpus}: method");
    let listed = paths(doc);
    assert!(
        listed.windows(2).all(|w| w[0] < w[1]),
        "{corpus}: tree paths not strictly ascending"
    );
    let slice = SLICES.load(exam, corpus);
    let files = universe(&slice);
    let tree: BTreeSet<&str> = listed.iter().copied().collect();
    let missing = files.iter().find(|f| !tree.contains(f.as_str()));
    assert!(
        missing.is_none(),
        "{corpus}: universe file {missing:?} not in the tree"
    );
    let other = listed.iter().filter(|p| !in_scope(exam, p)).count();
    let refused = listed.len() - other - files.len();
    let tally = |k: &str| slice["excluded"][k].as_u64().unwrap_or(0);
    assert_eq!(
        (other as u64, refused as u64),
        (tally("other_extension"), tally("walk_refused")),
        "{corpus}: the tree is not the one the slice tallied"
    );
}

/// The pinned tree, sorted — what the doc freezes.
fn pinned_tree(repo: &str, tip: &str) -> Vec<String> {
    let mut listed = tree_paths(repo, tip);
    listed.sort();
    listed
}

/// The frozen tree re-derived at the pinned clone, for a Reach::Tree
/// exam: a tree that drifted never scores (generate.rs scored_doc).
pub(super) fn assert_frozen_tree(exam: &Exam, corpus: &str, repo: &str, tip: &str) {
    if exam.reach == Reach::Tree {
        assert_eq!(
            TREES.load(exam, corpus)["paths"],
            json!(pinned_tree(repo, tip)),
            "{corpus}: the frozen tree is not the pinned tip's"
        );
    }
}

#[test]
#[ignore = "reads the pinned corpus clone"]
fn lang_tree() {
    let name = env("CE_LANG_CORPUS");
    let (exam, tip) = super::exam_of_corpus(&name);
    assert_eq!(
        exam.reach,
        Reach::Tree,
        "{name}: its truths name its universe"
    );
    let listed = pinned_tree(&corpus_repo(&name, tip), tip);
    let doc = json!({
        "schema": TREE_SCHEMA,
        "corpus": {"name": name, "tip": tip, "lang": exam.lang},
        "method": TREE_METHOD,
        "paths": listed,
    });
    verify_tree(exam, &name, &doc);
    freeze(&TREES.file(exam, &name), &doc);
}
