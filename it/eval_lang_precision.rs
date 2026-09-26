//! The v2.30 language exams' scoring gates (booklet
//! language-expansion.md §11; registry docs/EVAL-SET-LANGS.md): per
//! scored corpus a precision doc (`lang-precision-<corpus>-v<g>.json`) —
//! the frozen sample resolved by the shipped ladder and judged against
//! the frozen blind audit, generated after both (lang_provenance.rs).
//! No git, no corpus clone: every number re-derives from the frozen
//! docs (eval_lang_parts/precision.rs, through score.rs).

use crate::eval_lang_parts::precision::verify_precision;
use crate::eval_lang_parts::score::{GATE, scored};
use crate::eval_lang_parts::tamper::{oracle_precision, tamper_battery};
use crate::eval_lang_parts::{EXAMS, Exam, PRECISION_DOCS, SLICES, Stage, exam_of_corpus, tree};
use crate::eval_support::{
    TRUTH_KEYWORDS, assert_corpus_precision, assert_corpus_set, correct_wrong, doc_refused,
};
use serde_json::{Map, Value, json};

/// Every scored exam: the frozen set is exactly the scored exams'
/// corpora (G10) — and a language is scored only once audited — each
/// doc verifies (score.rs), and the M5-2 G2 contract holds per
/// language: precision >= 0.90 overall, and per corpus where the
/// in-corpus truths reach 5.
#[test]
fn lang_precision_contract() {
    let mut corpora = Vec::new();
    for exam in EXAMS.iter().filter(|e| scored(e)) {
        assert!(
            exam.stage >= Stage::Audited,
            "{}: scored before its audit (G13)",
            exam.lang
        );
        corpora.extend(exam.corpora.iter().map(|(c, _)| Some(c.to_string())));
    }
    corpora.sort();
    assert_corpus_set(PRECISION_DOCS.0, &corpora);
    for exam in EXAMS.iter().filter(|e| e.stage == Stage::Scored) {
        let sample = exam.sample();
        let (mut c, mut w) = (0, 0);
        for (corpus, _) in exam.corpora {
            let doc = PRECISION_DOCS.load(exam, corpus);
            verify_precision(exam, corpus, &doc, &sample);
            let (dc, dw) = correct_wrong(&doc);
            let truths = doc["summary"]["in_corpus_truths"]
                .as_u64()
                .expect("in_corpus_truths");
            assert_corpus_precision(corpus, (dc, dw), truths, GATE, "G2");
            (c, w) = (c + dc, w + dw);
        }
        let precision = c as f64 / (c + w) as f64;
        assert!(
            precision >= GATE,
            "{}: precision {precision:.3} < {GATE} (G2)",
            exam.lang
        );
    }
}

/// G9: a drifted identity or truth, a forged verdict, a dropped or
/// swapped row (the shared frame, tamper.rs), a flag that is no
/// boolean, a flipped answer, a cooked summary, ledger or rate, and a
/// moved site gap each refuse; and the RG1 trigger both ways — an R1
/// share past it refuses until a written disposition rides the doc.
/// The pristine doc is the oracle's on jsoup's frozen inputs
/// (tamper.rs: it holds all three site kinds), so the battery runs
/// whether or not a language is scored at the moment.
#[test]
fn a_tampered_precision_doc_is_refused() {
    let (pristine, refused) = tamper_battery(
        |exam, corpus| oracle_precision(exam, corpus, &|_| false),
        verify_precision,
        &[
            (
                "truth",
                "forged/Truth.java",
                "a truth that is not the audit's",
            ),
            ("verdict", "forged", "a forged verdict"),
        ],
    );
    let mut flag = pristine.clone();
    flag["rows"][0]["external"] = json!(1);
    let mut flipped = pristine.clone();
    flipped["rows"][0]["answered"] = json!("not/the/Truth.java");
    flipped["rows"][0]["verdict"] = json!("correct");
    let mut summary = pristine.clone();
    summary["summary"]["verdicts"]["wrong"] = json!(99);
    let ledger = bumped(&pristine, "universe", "resolution_by");
    let mut rate = pristine.clone();
    rate["universe"]["resolution_rate"] = json!(1.0);
    let mut gap = pristine.clone();
    gap["site_gaps"][0]["line"] =
        json!(pristine["site_gaps"][0]["line"].as_u64().expect("line") + 1);
    for (label, doc) in [
        ("an external flag that is no boolean", &flag),
        ("a flipped answer", &flipped),
        ("a cooked summary", &summary),
        ("a cooked ledger", &ledger),
        ("a cooked rate", &rate),
        ("a moved site gap", &gap),
    ] {
        assert!(refused(doc), "{label} must refuse");
    }
    assert_rg1_both_ways(&pristine, &refused);
}

/// The walk record and the truths it rewrites, on the oracle's luarocks
/// doc, whose walk refuses the vendored modules as the product's does
/// (`vendor/`, scan/walk.rs) — and two truths name one: a refused file
/// outside the universe, a refused file dropped (the sites no longer
/// add up), a cooked refused tally, an unreached path on an exam whose
/// truths stay in the universe, an unwalked truth scored as the
/// in-corpus file it names, and the audit's word beside a truth the
/// walk left alone each refuse.
#[test]
fn a_forged_walk_is_refused() {
    let exam = &EXAMS[1];
    let (corpus, _) = exam.corpora[0];
    let (pristine, check) = passing_oracle(exam, corpus, &|p| p.starts_with("vendor/"));
    let rows = pristine["rows"].as_array().expect("rows");
    let rewritten = rows
        .iter()
        .position(|r| r["audit_truth"].is_string())
        .expect("a truth the walk rewrote");
    let kept = rows
        .iter()
        .position(|r| r["audit_truth"].is_null())
        .expect("a truth the walk left alone");
    let mut outside = pristine.clone();
    outside["walk"]["refused"]
        .as_array_mut()
        .expect("refused")
        .push(json!("zz/no-universe-file.lua"));
    let mut dropped = pristine.clone();
    dropped["walk"]["refused"]
        .as_array_mut()
        .expect("refused")
        .pop();
    let tally = bumped(&pristine, "walk", "refused_sites_by");
    let mut unreached = pristine.clone();
    unreached["walk"]["unreached"]
        .as_array_mut()
        .expect("unreached")
        .push(json!("zz/beyond-the-universe.js"));
    let mut in_corpus = pristine.clone();
    let row = &mut in_corpus["rows"][rewritten];
    row["truth"] = row["audit_truth"].take();
    row["verdict"] = json!("missed");
    let mut beside = pristine.clone();
    beside["rows"][kept]["audit_truth"] = pristine["rows"][kept]["truth"].clone();
    for (label, doc) in [
        ("a refused file outside the universe", &outside),
        ("a refused file dropped", &dropped),
        ("a cooked refused tally", &tally),
        (
            "an unreached path where truths stay in the universe",
            &unreached,
        ),
        ("an unwalked truth scored in-corpus", &in_corpus),
        ("the audit's word beside a walked truth", &beside),
    ] {
        assert!(doc_refused(doc, &check), "{label} must refuse");
    }
}

/// The unreached paths of an exam whose truths reach the pinned tree
/// (HTML, learning-area), on the oracle's doc: a universe page listed
/// as unreached refuses, a path off the tree refuses, and a tree path
/// a truth names, listed as unreached, refuses because that truth is
/// then scored as outside the corpus while the doc still echoes the
/// audit's word — the record rewrites truths, so a forged one cannot
/// pass beside frozen rows.
#[test]
fn an_unreached_path_is_held_to_the_tree_and_rewrites_its_truths() {
    let exam = &EXAMS[3];
    let (corpus, _) = exam.corpora[2];
    let (pristine, check) = passing_oracle(exam, corpus, &|_| false);
    let universe = tree::universe(&SLICES.load(exam, corpus));
    let page = universe.iter().next().expect("a universe page").clone();
    let named = pristine["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|r| r["truth"].as_str().expect("truth"))
        .find(|t| {
            !TRUTH_KEYWORDS.contains(t) && !universe.contains(t.split('#').next().unwrap_or(t))
        })
        .expect("a truth naming a tree path outside the universe")
        .to_string();
    for (label, path) in [
        ("a universe page listed as unreached", page),
        ("a path off the tree", "zz/not-in-the-tree.png".to_string()),
        ("a tree path a truth names", named),
    ] {
        let mut doc = pristine.clone();
        doc["walk"]["unreached"]
            .as_array_mut()
            .expect("unreached")
            .push(json!(path));
        assert!(doc_refused(&doc, &check), "{label} must refuse");
    }
}

/// The oracle's doc under `refuse`, and the check that must pass it —
/// the prelude both walk legs share.
fn passing_oracle<'a>(
    exam: &'a Exam,
    corpus: &'a str,
    refuse: &dyn Fn(&str) -> bool,
) -> (Value, impl Fn(&Value) + 'a) {
    let sample = exam.sample();
    let pristine = oracle_precision(exam, corpus, refuse);
    let check = move |doc: &Value| verify_precision(exam, corpus, doc, &sample);
    assert!(!doc_refused(&pristine, &check), "the pristine doc passes");
    (pristine, check)
}

/// The RG1 trigger both ways: a ledger whose R1 share is past it
/// refuses until a written disposition rides the doc, and then passes.
fn assert_rg1_both_ways(pristine: &Value, refused: &dyn Fn(&Value) -> bool) {
    let mut cowardly = pristine.clone();
    cowardly["universe"] = all_first_rung(pristine["corpus"]["name"].as_str().expect("corpus"));
    assert!(
        refused(&cowardly),
        "an R1 share past the trigger must refuse"
    );
    cowardly["r0_disposition"] =
        json!("a written disposition, long enough to be a reason and not a label");
    assert!(
        !refused(&cowardly),
        "a written disposition clears the RG1 trigger"
    );
}

/// The doc with one tally's first cell counted one higher.
fn bumped(pristine: &Value, section: &str, tally: &str) -> Value {
    let (cell, n) = pristine[section][tally]
        .as_object()
        .expect("tally")
        .iter()
        .next()
        .expect("a cell");
    let mut doc = pristine.clone();
    doc[section][tally][cell] = json!(n.as_u64().expect("count") + 1);
    doc
}

/// A ledger true to the frozen universe kind by kind, with every site
/// answered at R1 — the RG1 "cowardly precision" shape, consistent in
/// every other respect, so only the trigger can refuse it.
fn all_first_rung(corpus: &str) -> Value {
    let slice = SLICES.load(exam_of_corpus(corpus).0, corpus);
    let sites = slice["summary"]["sites_by"].as_object().expect("sites_by");
    let by: Map<String, Value> = sites
        .iter()
        .map(|(cell, n)| (format!("{cell}/r1"), n.clone()))
        .collect();
    json!({
        "total_sites": slice["summary"]["total_sites"],
        "resolution_rate": 1.0,
        "r0_share": 1.0,
        "resolution_by": by,
        "unresolved_by": {},
    })
}
