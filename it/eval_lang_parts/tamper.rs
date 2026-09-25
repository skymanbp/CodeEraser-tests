//! The exam docs' tamper frame (G9), one throat for the three frozen
//! families that keep their rows in sample order and echo the sampled
//! identity — the sample (verify.rs), the audit tables (review.rs) and
//! the precision docs (precision.rs). Each family's gate adds only its
//! bespoke cases; the batteries grew the same prologue three times
//! before the clone gate paired them.

use super::{Docs, EXAMS, Exam};
use crate::eval_support::{assert_tampering_refused, doc_refused, eval_doc, load};
use serde_json::Value;

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
    docs: &Docs,
    verify: fn(&Exam, &str, &Value, &Value),
    extra: &[(&str, &str, &str)],
) -> (Value, impl Fn(&Value) -> bool) {
    let exam = &EXAMS[0];
    let (corpus, _) = exam.corpora[1];
    let sample = load(&eval_doc(&format!("lang-sample-{}", exam.lang)));
    let pristine = docs.load(corpus);
    let check = move |doc: &Value| verify(exam, corpus, doc, &sample);
    assert_exam_tampering(&pristine, extra, &check);
    (pristine, move |doc: &Value| doc_refused(doc, &check))
}
