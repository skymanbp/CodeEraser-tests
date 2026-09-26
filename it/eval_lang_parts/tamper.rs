//! The exam docs' tamper frame (G9), one throat for the three frozen
//! families that keep their rows in sample order and echo the sampled
//! identity — the sample (verify.rs), the audit tables (review.rs) and
//! the precision docs (precision.rs). Each family's gate adds only its
//! bespoke cases; the batteries grew the same prologue three times
//! before the clone gate paired them. The precision family's pristine
//! doc is the one an oracle ladder scores (oracle_precision): the
//! real docs retire whenever a ladder change moves their answers.

use super::score::{PRECISION_SCHEMA, answer, judged, ledger, summary};
use super::walk::{Walk, universe_files, walk_record};
use super::{AUDIT_TABLES, EXAMS, Exam, SLICES};
use crate::eval_support::{
    TRUTH_KEYWORDS, assert_tampering_refused, doc_refused, of_corpus, sum_obj_into,
};
use codeeraser::graph::ladder::{Outcome, Reason};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// The pristine doc passes; a forged path or spec on row 0, each of the
/// family's `extra` row-0 mutations, a dropped row and a swapped pair
/// refuse.
pub fn assert_exam_tampering(
    pristine: &Value,
    extra: &[(&str, &str, &str)],
    check: &dyn Fn(&Value),
) {
    let mut mutations = vec![
        ("path", "forged/Path.java", "a forged path"),
        ("spec", "forged.Spec", "a forged spec"),
    ];
    mutations.extend_from_slice(extra);
    assert_tampering_refused(pristine, &mutations, check);
    let mut swapped = pristine.clone();
    swapped["rows"].as_array_mut().expect("rows").swap(0, 1);
    assert!(doc_refused(&swapped, check), "a swapped pair must refuse");
}

/// The frame on a per-corpus family's doc of the first exam's second
/// corpus (jsoup: every site kind, every truth class), checked against
/// the exam's frozen sample; returns the pristine doc and the refusal
/// probe for the family's bespoke cases.
pub fn tamper_battery(
    pristine_of: fn(&Exam, &str) -> Value,
    verify: fn(&Exam, &str, &Value, &Value),
    extra: &[(&str, &str, &str)],
) -> (Value, impl Fn(&Value) -> bool) {
    let exam = &EXAMS[0];
    let (corpus, _) = exam.corpora[1];
    let sample = exam.sample();
    let pristine = pristine_of(exam, corpus);
    let check = move |doc: &Value| verify(exam, corpus, doc, &sample);
    assert_exam_tampering(&pristine, extra, &check);
    (pristine, move |doc: &Value| doc_refused(doc, &check))
}

/// The precision doc an oracle ladder scores from the frozen inputs
/// alone — no clone, no ladder: each sampled row answered with its
/// truth as the walk scores it, the walked universe's sites half
/// resolved at rung 2 and half refused (an R1 share of 0, under the RG1
/// trigger), each audit site gap with no detected site. The verifier
/// holds a doc to its frozen inputs and never to a ladder, so the
/// oracle's doc passes it: the pristine doc the precision tamper gates
/// mutate, whether or not a language is scored at the moment.
/// `refuses` picks the universe files its walk refuses.
pub fn oracle_precision(exam: &Exam, corpus: &str, refuses: &dyn Fn(&str) -> bool) -> Value {
    let (slice, review) = (SLICES.load(exam, corpus), AUDIT_TABLES.load(exam, corpus));
    let universe = universe_files(&slice);
    let refused: BTreeSet<String> = universe
        .iter()
        .filter(|p| refuses(p))
        .map(|p| p.to_string())
        .collect();
    let walk = Walk::new(universe, refused.clone());
    let record = walk_record(&slice, &refused);
    let sample = exam.sample();
    let rows: Vec<Value> = of_corpus(sample["rows"].as_array().expect("rows"), corpus)
        .into_iter()
        .zip(review["rows"].as_array().expect("rows"))
        .map(|(s, a)| {
            let said = a["truth"].as_str().expect("truth");
            judged(s, said, &walk, answer(&oracle(walk.scored(said))))
        })
        .collect();
    let gaps: Vec<Value> = review["site_gaps"]
        .as_array()
        .expect("site_gaps")
        .iter()
        .map(|g| json!({"path": g["path"], "line": g["line"], "sites": []}))
        .collect();
    json!({
        "schema": PRECISION_SCHEMA,
        "corpus": {"name": corpus, "tip": exam.tip(corpus).expect("tip"), "lang": exam.lang},
        "generated_from": {"commit": "oracle"},
        "universe": oracle_ledger(&slice, &record),
        "walk": record,
        "summary": summary(&rows),
        "site_gaps": gaps,
        "rows": rows,
    })
}

/// The oracle's answer to a truth: an in-corpus truth resolved as
/// written, "external" by External, a keyword by a refusal.
fn oracle(truth: &str) -> Outcome {
    match truth {
        "external" => Outcome::External { rung: 2 },
        t if TRUTH_KEYWORDS.contains(&t) => Outcome::Unresolved(Reason::OutOfScope),
        t => Outcome::Resolved {
            path: t.to_string(),
            rung: 2,
        },
    }
}

/// The oracle's ledger: each walked cell of the universe, its sites
/// less the refused files' own, half resolved at rung 2 (rounded up)
/// and half refused — so the resolution rate is a number a forged rate
/// can move, not the 1.0 of a doc with no refusal.
fn oracle_ledger(slice: &Value, record: &Value) -> Value {
    let (mut sites, mut gone) = (BTreeMap::new(), BTreeMap::new());
    sum_obj_into(&slice["summary"]["sites_by"], &mut sites);
    sum_obj_into(&record["refused_sites_by"], &mut gone);
    let (mut by, mut refused) = (BTreeMap::new(), BTreeMap::new());
    for (cell, n) in sites {
        let walked = n - gone.get(&cell).copied().unwrap_or(0);
        for (tally, key, count) in [
            (&mut by, format!("{cell}/r2"), walked - walked / 2),
            (&mut refused, format!("{cell}/out_of_scope"), walked / 2),
        ] {
            if count > 0 {
                tally.insert(key, count);
            }
        }
    }
    ledger(by, refused)
}
