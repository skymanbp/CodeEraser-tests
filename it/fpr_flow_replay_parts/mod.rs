//! The flow ledger's shared half (plan v2.31 step 4 commit D; design
//! booklet analysis-track.md §5.5 「回放台账」): the book (book.rs) and
//! its two readings (lane.rs), one judged file version (side.rs), the
//! walk (walk.rs), the tables docs/FPR-REPLAY.md quotes (render.rs),
//! and the frozen row with its derived rates — read alike by the
//! instrument (fpr_flow_replay.rs) and its executor (fpr_flow_gate.rs),
//! so the frozen row and the gate's recomputation cannot drift apart.

pub mod book;
pub mod lane;
pub mod render;
pub mod shape;
pub mod side;
pub mod walk;

use crate::common::ledger::{Ledger, WINDOW};
use crate::eval_flow_parts::{EXAMS, FlowExam};
use book::Tally;
use serde_json::{Map, Value, json};
pub use shape::{COUNTS, OUTCOME, Outcome, RATED, READINGS, rate};

pub const DOC: &str = "contracts/eval/fpr-flow-v1.json";
pub const SCHEMA: &str = "ce.eval-fpr-flow/1.0.0";

/// No admission line: a flow language is admitted by its precision
/// doc (booklet §13 item 6, 精度考题 ≥ 99 %，回放台账并记); this
/// ledger is the second reading, recorded beside it.
pub const LEDGER: Ledger = Ledger {
    rel: DOC,
    schema: SCHEMA,
    generated_from: "cli/tests/it/fpr_flow_replay.rs",
    key: "lang",
    gate_ppm: None,
};

/// One reading's outcome as JSON, under the OUTCOME keys.
fn reading(o: &Outcome) -> Value {
    let cells: Map<String, Value> = OUTCOME
        .split_whitespace()
        .zip(o)
        .map(|(k, v)| (k.to_string(), json!(v)))
        .collect();
    Value::Object(cells)
}

/// Both readings of an outcome pair, keyed by READINGS.
fn readings(pair: &[Outcome; 2]) -> Map<String, Value> {
    READINGS
        .iter()
        .zip(pair)
        .map(|(r, o)| (r.to_string(), reading(o)))
        .collect()
}

/// The exam a row is read for and its ledger corpus: the exam's first
/// corpus at its pinned tip (eval_flow_parts::EXAMS, the one source).
pub fn corpus_of(exam: &FlowExam) -> (&'static str, &'static str) {
    exam.corpora[0]
}

/// A row's rank: its exam's place in EXAMS.
pub fn rank(c: &Value) -> (usize, String) {
    let lang = c["lang"].as_str().unwrap_or("?");
    let at = EXAMS.iter().position(|e| e.lang == lang);
    let at = at.unwrap_or_else(|| panic!("{lang}: no flow exam"));
    (at, String::new())
}

/// The rated kinds summed, per reading.
pub fn rated(t: &Tally) -> [Outcome; 2] {
    let mut sum = [[0; 5]; 2];
    for pair in &t.kinds[..RATED] {
        for (s, o) in sum.iter_mut().zip(pair) {
            s.iter_mut().zip(o).for_each(|(a, b)| *a += b);
        }
    }
    sum
}

/// The frozen row: the scalar counts, both readings over the rated
/// kinds, every kind's two readings (`kinds`), and the harness. The
/// rates are never stored: the gate derives them.
pub fn row(exam: &FlowExam, commits: usize, t: &Tally, harness: Value, date: &str) -> Value {
    let (corpus, tip) = corpus_of(exam);
    let mut row = json!({
        "lang": exam.lang, "corpus": corpus, "tip": tip, "window": WINDOW,
        "commits": commits, "events": t.events, "units_judged": t.units_judged,
        "dynamic_skipped": t.dynamic_skipped, "unjudged": t.unjudged,
    });
    let fields = row.as_object_mut().expect("an object");
    fields.extend(readings(&rated(t)));
    let kinds = t.kinds.iter().enumerate().map(|(k, pair)| {
        let mut cell = readings(pair);
        cell.insert("kind".into(), json!(k));
        Value::Object(cell)
    });
    fields.insert("kinds".into(), Value::Array(kinds.collect()));
    fields.insert("harness".into(), harness);
    fields.insert("measured_at".into(), json!(date));
    row
}
