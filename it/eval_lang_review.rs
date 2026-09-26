//! The v2.30 language exams' audit-table gates (registry
//! docs/EVAL-SET-LANGS.md): per audited corpus a blind ground-truth
//! table (`lang-review-<corpus>-v<g>.json`, g the exam's generation)
//! verified against its exam's frozen sample and universe
//! (eval_lang_parts/review.rs), and the forgeries the verifier must
//! refuse. Split from eval_lang.rs, the universes' and samples' gates,
//! as eval_lang_precision.rs holds the precision docs'. No git, no
//! corpus clone.

use crate::eval_lang_parts::review::{self, PACKAGE_KINDS, class_of, verify_review};
use crate::eval_lang_parts::tamper::tamper_battery;
use crate::eval_lang_parts::tree::{targets, universe};
use crate::eval_lang_parts::{AUDIT_TABLES, EXAMS, Exam, Reach, SLICES};
use crate::eval_support::{TRUTH_KEYWORDS, assert_corpus_set, doc_refused};
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Every frozen audit verifies against its exam's sample: one judged
/// row per primary, truths bound to the frozen universe (review.rs).
/// The frozen tables are exactly the audited exams' corpora (G10): an
/// exam whose audit is pending has none, by its flag, and a retired
/// generation's table left behind reads as its corpus twice.
#[test]
fn lang_reviews_verify() {
    let tables = audited_tables();
    for (exam, corpus, table, sample) in &tables {
        verify_review(exam, corpus, table, sample);
    }
    let mut corpora: Vec<Option<String>> = tables
        .iter()
        .map(|(_, c, ..)| Some(c.to_string()))
        .collect();
    corpora.sort();
    assert_corpus_set(AUDIT_TABLES.0, &corpora);
}

/// Every audited table with its exam and the exam's sample, in EXAMS
/// order — the walk the audit gates share.
fn audited_tables() -> Vec<(&'static Exam, &'static str, Value, Value)> {
    let mut tables = Vec::new();
    for exam in EXAMS.iter().filter(|e| review::audited(e)) {
        let sample = exam.sample();
        for (corpus, _) in exam.corpora {
            tables.push((
                exam,
                *corpus,
                AUDIT_TABLES.load(exam, corpus),
                sample.clone(),
            ));
        }
    }
    tables
}

/// G9 for the audit tables: a truth off the frozen universe or out
/// of the vocabulary, a drifted echo, a why under the floor, a
/// dropped row, a swapped pair (the shared frame, tamper.rs) and a
/// cooked summary — each refuses.
#[test]
fn a_tampered_review_is_refused() {
    let (pristine, refused) = tamper_battery(
        |exam, corpus| AUDIT_TABLES.load(exam, corpus),
        verify_review,
        &[
            ("truth", "forged/Path.java", "a truth off the universe"),
            ("truth", "elsewhere", "a truth out of the vocabulary"),
            ("why", "a label, not a reason", "a why under the floor"),
        ],
    );
    let mut cooked = pristine.clone();
    cooked["summary"]["external"] = json!(0);
    assert!(refused(&cooked), "a cooked summary must refuse");
}

/// A package truth answers a package-level site only, its code where
/// the language keeps it (review.rs PACKAGE_KINDS): moved onto a row
/// of another kind it refuses, and so does the directory that holds
/// a package's code standing in for the package (R's `R/`). Every
/// audited table that holds a package truth aims one.
#[test]
fn a_misplaced_package_truth_is_refused() {
    let mut aimed = Vec::new();
    for (exam, corpus, pristine, sample) in audited_tables() {
        let reach = reach_of(exam, corpus);
        let rows = pristine["rows"].as_array().expect("rows");
        let Some(at) = rows.iter().position(|r| package_truth(r, &reach)) else {
            continue;
        };
        let check = |doc: &Value| verify_review(exam, corpus, doc, &sample);
        assert!(
            !doc_refused(&pristine, &check),
            "{corpus}: pristine table must pass"
        );
        for forged in misplaced(&pristine, at) {
            assert!(
                doc_refused(&forged, &check),
                "{corpus}: a misplaced package truth must refuse"
            );
        }
        aimed.push(corpus);
    }
    assert_eq!(
        aimed,
        ["jsoup", "covid19model"],
        "the tables holding a package truth"
    );
}

/// The files the exam's truths may name in one corpus (tree.rs).
fn reach_of(exam: &Exam, corpus: &str) -> BTreeSet<String> {
    targets(exam, corpus, &universe(&SLICES.load(exam, corpus)))
}

/// Whether a row's truth is a package directory, by the verifier's own
/// reading of it (review.rs class_of): a file the exam's truths may
/// name is no package, whatever its extension.
fn package_truth(row: &Value, reach: &BTreeSet<String>) -> bool {
    let truth = row["truth"].as_str().expect("truth");
    class_of(truth, row["kind"].as_str().expect("kind"), reach) == "package"
}

/// Where an exam's references reach any tracked file, a truth names a
/// path of the pinned tree beyond the universe (a page's stylesheet,
/// an image) and passes; one off the tree refuses. The first such
/// audited table aims it.
#[test]
fn a_truth_beyond_the_tree_is_refused() {
    let (exam, corpus, pristine, sample) = audited_tables()
        .into_iter()
        .find(|(e, ..)| e.reach == Reach::Tree)
        .expect("an audited exam reaching the tree");
    let files = universe(&SLICES.load(exam, corpus));
    let beyond = |r: &Value| {
        let truth = r["truth"].as_str().expect("truth");
        let path = truth.split('#').next().unwrap_or(truth);
        !TRUTH_KEYWORDS.contains(&truth) && !files.contains(path)
    };
    let rows = pristine["rows"].as_array().expect("rows");
    let at = rows
        .iter()
        .position(beyond)
        .expect("a truth beyond the universe");
    let check = |doc: &Value| verify_review(exam, corpus, doc, &sample);
    assert!(
        !doc_refused(&pristine, &check),
        "{corpus}: pristine table must pass"
    );
    let mut forged = pristine.clone();
    forged["rows"][at]["truth"] = json!("site/forged.png");
    assert!(
        doc_refused(&forged, &check),
        "{corpus}: a truth beyond the tree must refuse"
    );
}

/// The forgeries of row `at`'s package truth: the truth on the first
/// row of another kind, and — where the kind keeps its code in a
/// subdirectory — that code directory standing in for the package.
fn misplaced(pristine: &Value, at: usize) -> Vec<Value> {
    let row = &pristine["rows"][at];
    let (truth, kind) = (row["truth"].as_str().expect("truth"), &row["kind"]);
    let mut moved = pristine.clone();
    let other = moved["rows"]
        .as_array_mut()
        .expect("rows")
        .iter_mut()
        .find(|r| &r["kind"] != kind)
        .expect("a row of another kind");
    other["truth"] = json!(truth);
    let mut forged = vec![moved];
    let code = PACKAGE_KINDS
        .iter()
        .find(|(k, _)| json!(k) == *kind)
        .map_or("", |(_, c)| c.trim_end_matches('/'));
    if !code.is_empty() {
        let mut deeper = pristine.clone();
        deeper["rows"][at]["truth"] = json!(format!("{truth}/{code}"));
        forged.push(deeper);
    }
    forged
}

/// Gaps sit on the side of the frozen universe their list names
/// (review.rs check_gaps): a scope gap filed as a site gap refuses,
/// and so does a site gap filed as a scope gap. The first audited table
/// holding a site gap carries both, a scope gap on a file the universe
/// does not walk set beside it (luarocks' launcher `src/bin/luarocks`
/// is Lua without the extension).
#[test]
fn a_gap_on_the_wrong_side_is_refused() {
    let (exam, corpus, mut pristine, sample) = audited_tables()
        .into_iter()
        .find(|(_, _, table, _)| table["site_gaps"].as_array().is_some_and(|g| !g.is_empty()))
        .expect("an audited table holding a site gap");
    pristine["scope_gaps"] = json!([{"path": "bin/launcher", "line": 1, "note": "no extension"}]);
    let check = |doc: &Value| verify_review(exam, corpus, doc, &sample);
    assert!(
        !doc_refused(&pristine, &check),
        "the pristine table verifies"
    );
    for (from, to) in [("scope_gaps", "site_gaps"), ("site_gaps", "scope_gaps")] {
        let mut moved = pristine.clone();
        let gap = moved[from].as_array_mut().expect(from).remove(0);
        moved[to].as_array_mut().expect(to).push(gap);
        assert!(
            doc_refused(&moved, &check),
            "a {from} entry filed under {to} must refuse"
        );
    }
}
