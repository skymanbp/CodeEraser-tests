//! The v2.30 language exams' scoring gates (booklet
//! language-expansion.md §11; registry docs/EVAL-SET-LANGS.md): per
//! scored corpus a precision doc (`lang-precision-<corpus>-v1.json`) —
//! the frozen sample resolved by the shipped ladder and judged against
//! the frozen blind audit, generated after both (lang_provenance.rs).
//! No git, no corpus clone: every number re-derives from the frozen
//! docs (eval_lang_parts/precision.rs, through score.rs).

use crate::eval_lang_parts::precision::verify_precision;
use crate::eval_lang_parts::score::{GATE, scored};
use crate::eval_lang_parts::tamper::tamper_battery;
use crate::eval_lang_parts::{EXAMS, PRECISION_DOCS};
use crate::eval_support::{
    assert_corpus_precision, assert_corpus_set, correct_wrong, eval_doc, load,
};
use serde_json::{Map, Value, json};

fn sample_of(lang: &str) -> Value {
    load(&eval_doc(&format!("lang-sample-{lang}")))
}

/// Every scored exam: the frozen set is exactly the scored exams'
/// corpora (G10) — and a language is scored only once audited — each
/// doc verifies (score.rs), and the M5-2 G2 contract holds per
/// language: precision >= 0.90 overall, and per corpus where the
/// in-corpus truths reach 5.
#[test]
fn lang_precision_contract() {
    let mut corpora = Vec::new();
    for exam in EXAMS.iter().filter(|e| scored(e)) {
        assert!(exam.audited, "{}: scored before its audit (G13)", exam.lang);
        corpora.extend(exam.corpora.iter().map(|(c, _)| Some(c.to_string())));
    }
    corpora.sort();
    assert_corpus_set(PRECISION_DOCS.0, &corpora);
    for exam in EXAMS.iter().filter(|e| e.scored) {
        let sample = sample_of(exam.lang);
        let (mut c, mut w) = (0, 0);
        for (corpus, _) in exam.corpora {
            let doc = PRECISION_DOCS.load(corpus);
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
/// jsoup's doc holds all three site kinds.
#[test]
fn a_tampered_precision_doc_is_refused() {
    let (pristine, refused) = tamper_battery(
        &PRECISION_DOCS,
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
    let mut ledger = pristine.clone();
    let (cell, n) = first(&pristine["universe"]["resolution_by"]);
    ledger["universe"]["resolution_by"][&cell] = json!(n + 1);
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

/// A tally's first cell and its count.
fn first(tally: &Value) -> (String, u64) {
    let (cell, n) = tally
        .as_object()
        .expect("tally")
        .iter()
        .next()
        .expect("a cell");
    (cell.clone(), n.as_u64().expect("count"))
}

/// A ledger true to the frozen universe kind by kind, with every site
/// answered at R1 — the RG1 "cowardly precision" shape, consistent in
/// every other respect, so only the trigger can refuse it.
fn all_first_rung(corpus: &str) -> Value {
    let slice = load(&eval_doc(&format!("lang-slice-{corpus}")));
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
