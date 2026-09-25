//! The frozen precision-doc verifier of the v2.30 language exams: one
//! corpus's scored sample (`lang-precision-<corpus>-v1.json`) against
//! the frozen sample it answers, the frozen audit whose truths it
//! echoes and the frozen universe its ledger must equal — every number
//! re-derived through the scorer's own functions (score.rs, G1). Split
//! from the gates like review.rs and verify.rs: the tamper gate runs
//! it on forged copies.

use super::score::{PRECISION_SCHEMA, ledger, shape_of, summary};
use crate::eval_graph_precision_parts::verdict_of;
use crate::eval_lang_parts::review::ECHO;
use crate::eval_lang_parts::{AUDIT_TABLES, Exam, slice_constants};
use crate::eval_support::{MIN_WHY, eval_doc, load, of_corpus, sum_obj_into, tally_add};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// One corpus's precision doc against its exam: the envelope; one row
/// per sampled primary in sample order, echoing its identity and the
/// audit's truth, in one of the three answer shapes, its verdict
/// re-derived (G6); the summary re-derived from the rows (G1); the
/// universe ledger re-derived from its tallies and equal, kind by kind,
/// to the frozen slice — with a written disposition when the R1 share
/// passes the pre-registered trigger; and the audit's site gaps, in
/// order, each site answered in one shape.
pub fn verify_precision(exam: &Exam, corpus: &str, doc: &Value, sample: &Value) {
    let tip = exam
        .tip(corpus)
        .unwrap_or_else(|| panic!("{corpus}: no corpus of the exam"));
    assert_eq!(doc["schema"], json!(PRECISION_SCHEMA), "{corpus}: schema");
    let identity = json!({"name": corpus, "tip": tip, "lang": exam.lang});
    assert_eq!(doc["corpus"], identity, "{corpus}: not the exam's corpus");
    assert!(
        doc["generated_from"]["commit"].is_string(),
        "{corpus}: generated_from"
    );
    let review = AUDIT_TABLES.load(corpus);
    let rows = doc["rows"].as_array().expect("rows");
    let sampled = of_corpus(sample["rows"].as_array().expect("rows"), corpus);
    assert_eq!(rows.len(), sampled.len(), "{corpus}: judged row count (G3)");
    let truths = review["rows"].as_array().expect("rows");
    for ((row, s), audit) in rows.iter().zip(sampled).zip(truths) {
        let rank = s["rank"].as_str().expect("rank");
        for field in ECHO.iter().chain(&["lang"]) {
            assert_eq!(
                row[*field], s[*field],
                "{corpus}/{rank}: {field} is not the sampled one"
            );
        }
        assert_eq!(
            row["truth"], audit["truth"],
            "{corpus}/{rank}: not the audit's truth"
        );
        assert!(shape_of(row).is_some(), "{corpus}/{rank}: no answer shape");
        let truth = row["truth"].as_str().expect("truth");
        let verdict = verdict_of(truth, row["answered"].as_str(), row["external"] == true);
        assert_eq!(
            row["verdict"],
            json!(verdict),
            "{corpus}/{rank}: verdict contradicts its row (G6)"
        );
    }
    assert_eq!(
        doc["summary"],
        summary(rows),
        "{corpus}: summary drifted from rows (G1)"
    );
    verify_universe(corpus, doc);
    verify_gaps(corpus, doc, &review);
}

/// The ledger's rates re-derived from its tallies, the tallies equal to
/// the frozen slice kind by kind, and the RG1 trigger honoured.
fn verify_universe(corpus: &str, doc: &Value) {
    let u = &doc["universe"];
    let (mut by, mut refused) = (BTreeMap::new(), BTreeMap::new());
    sum_obj_into(&u["resolution_by"], &mut by);
    sum_obj_into(&u["unresolved_by"], &mut refused);
    let mut per_kind = BTreeMap::new();
    for (key, n) in by.iter().chain(&refused) {
        let (cell, _) = key.rsplit_once('/').expect("lang/kind/outcome");
        tally_add(&mut per_kind, cell, *n);
    }
    let slice = load(&eval_doc(&format!("lang-slice-{corpus}")));
    assert_eq!(
        json!(per_kind),
        slice["summary"]["sites_by"],
        "{corpus}: the ledger is not the frozen universe"
    );
    assert_eq!(
        *u,
        ledger(by, refused),
        "{corpus}: ledger rates drifted (G1)"
    );
    let trigger = slice_constants()["r0_share_trigger"]
        .as_f64()
        .expect("trigger");
    if u["r0_share"].as_f64().is_some_and(|r| r > trigger) {
        let written = doc["r0_disposition"]
            .as_str()
            .is_some_and(|d| d.chars().count() >= MIN_WHY);
        assert!(
            written,
            "{corpus}: the R1 share passed {trigger} with no written disposition (RG1)"
        );
    }
}

/// The audit's site gaps, in order, each detected site in one shape.
fn verify_gaps(corpus: &str, doc: &Value, review: &Value) {
    let gaps = doc["site_gaps"].as_array().expect("site_gaps");
    let want = review["site_gaps"].as_array().expect("site_gaps");
    assert_eq!(gaps.len(), want.len(), "{corpus}: site gap count");
    for (g, w) in gaps.iter().zip(want) {
        assert!(
            g["path"] == w["path"] && g["line"] == w["line"],
            "{corpus}: not the audit's site gap"
        );
        for s in g["sites"].as_array().expect("sites") {
            let named = s["nth"].is_u64() && s["kind"].is_string() && s["spec"].is_string();
            assert!(
                named && shape_of(s).is_some(),
                "{corpus}: a gap site without its answer"
            );
        }
    }
}
