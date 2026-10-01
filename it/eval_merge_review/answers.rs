//! The clone-merge audit's review doc (plan v2.31 step 7, design
//! booklet §13 item 36): the four judges' answer files read against the
//! batch plan (eval_merge_batches.rs) and assembled, in the sample's
//! order, into the exam generation's `merge-review-v<n>.json`; and the
//! verifier the gate and
//! the tamper battery run on the filed doc. One row check serves both,
//! so an answer line refused while assembling is the row the verifier
//! refuses once filed. Every refusal is named by batch and line and none
//! stops the read: a batch is re-dispatched once, not once per defect.

use crate::eval_merge_batches::{BATCHES, READINGS, plan};
use crate::eval_merge_parts::sample::SAMPLE;
use crate::eval_support::MIN_WHY;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const REVIEW_SCHEMA: &str = "ce.eval-merge-review/1.0.0";
/// The six reason names an audited row answers (merge/face.rs).
pub const REASONS: [&str; 6] = codeeraser::merge::face::REASONS;
/// A note's length bound, in characters.
pub const NOTE_MAX: usize = 200;
/// The fields of an answer line and of a review row, exactly.
const FIELDS: [&str; 5] = ["id", "feasible", "reason", "params", "note"];
/// The envelope's keys, exactly.
const ENVELOPE: [&str; 5] = ["schema", "generated_from", "auditor", "batches", "rows"];

fn keys(v: &Value) -> BTreeSet<&str> {
    v.as_object()
        .map(|o| o.keys().map(String::as_str).collect())
        .unwrap_or_default()
}

/// Why a row is not an answer, if it is not: exactly the five fields,
/// `feasible` a boolean, `reason` one of the six names and `ok` exactly
/// when feasible, `params` a non-negative integer, `note` a non-empty
/// sentence of at most NOTE_MAX characters. The first failing check
/// names it.
pub fn refusal(row: &Value) -> Option<String> {
    if keys(row) != BTreeSet::from(FIELDS) {
        return Some(format!("fields {:?}, not exactly {FIELDS:?}", keys(row)));
    }
    let reason = row["reason"].as_str().unwrap_or_default();
    let note = row["note"].as_str().unwrap_or_default();
    let checks = [
        (row["feasible"].is_boolean(), "feasible is not a boolean"),
        (
            REASONS.contains(&reason),
            "reason is not one of the six names",
        ),
        (
            row["feasible"].as_bool() == Some(reason == "ok"),
            "feasible is not exactly `reason == ok`",
        ),
        (
            row["params"].is_u64(),
            "params is not a non-negative integer",
        ),
        (
            !note.trim().is_empty() && note.chars().count() <= NOTE_MAX,
            "note empty or past 200 characters",
        ),
    ];
    let failed = checks.iter().find(|(holds, _)| !holds);
    failed.map(|(_, why)| (*why).to_string())
}

/// One batch's rows, read verbatim: as many lines as questions, each a
/// row `refusal` passes, the ids the batch's in its order.
fn read_batch(n: usize, ids: &[&str], text: Option<&str>, errs: &mut Vec<String>) -> Vec<Value> {
    let Some(text) = text else {
        errs.push(format!("batch {n}: no answers file"));
        return Vec::new();
    };
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() != ids.len() {
        let (l, q) = (lines.len(), ids.len());
        errs.push(format!("batch {n}: {l} lines for {q} questions"));
    }
    let mut rows = Vec::new();
    for (k, line) in lines.iter().enumerate() {
        let at = format!("batch {n} line {}", k + 1);
        match serde_json::from_str::<Value>(line) {
            Err(e) => errs.push(format!("{at}: not JSON ({e})")),
            Ok(row) => {
                if let Some(why) = refusal(&row) {
                    errs.push(format!("{at}: {why}"));
                }
                rows.push(row);
            }
        }
    }
    let seen: Vec<&str> = rows.iter().filter_map(|r| r["id"].as_str()).collect();
    if seen != ids {
        errs.push(format!(
            "batch {n}: the ids are not the batch's, in its order"
        ));
    }
    rows
}

/// The batch plan's ids, batch by batch.
fn planned(sample: &Value) -> Vec<Vec<&str>> {
    let mut out = Vec::new();
    for batch in plan(sample) {
        out.push(
            batch
                .into_iter()
                .map(|r| r["id"].as_str().expect("id"))
                .collect(),
        );
    }
    out
}

/// The review doc, or every refusal: the manifest must be the sample's
/// batch plan, the auditor a sentence, every batch's file its answers;
/// the rows go in the sample's order. `file(n)` is batch n's answer
/// text (None when absent), `stamp` the provenance stamp.
pub fn assemble(
    sample: &Value,
    manifest: &Value,
    file: &dyn Fn(usize) -> Option<String>,
    auditor: &str,
    stamp: Value,
) -> Result<Value, Vec<String>> {
    let batches = planned(sample);
    let mut errs = Vec::new();
    let listed: Vec<&Value> = manifest["batches"]
        .as_array()
        .into_iter()
        .flatten()
        .collect();
    let shown: Vec<Value> = listed.iter().map(|b| b["ids"].clone()).collect();
    let want: Vec<Value> = batches.iter().map(|b| json!(b)).collect();
    let head = json!({"sample": SAMPLE, "readings": READINGS});
    let given = json!({"sample": manifest["sample"], "readings": manifest["readings"]});
    if given != head || shown != want {
        errs.push("manifest: not the sample's batch plan".to_string());
    }
    if auditor.chars().count() < MIN_WHY {
        errs.push(format!("auditor: under {MIN_WHY} characters"));
    }
    let mut answered: BTreeMap<String, Value> = BTreeMap::new();
    for (i, ids) in batches.iter().enumerate() {
        for row in read_batch(i + 1, ids, file(i + 1).as_deref(), &mut errs) {
            answered.insert(row["id"].as_str().unwrap_or_default().to_string(), row);
        }
    }
    if !errs.is_empty() {
        return Err(errs);
    }
    let sampled = sample["rows"].as_array().expect("rows");
    let rows: Vec<&Value> = sampled
        .iter()
        .map(|s| &answered[s["id"].as_str().expect("id")])
        .collect();
    Ok(json!({
        "schema": REVIEW_SCHEMA, "generated_from": stamp, "auditor": auditor,
        "batches": BATCHES, "rows": rows,
    }))
}

/// The filed review against its sample: the envelope's five keys, the
/// schema, the batch count, an auditor sentence, a provenance stamp
/// naming a commit; then one row per sampled row, in the sample's
/// order, each a row `refusal` passes.
pub fn verify_review(sample: &Value, doc: &Value) {
    assert_eq!(keys(doc), BTreeSet::from(ENVELOPE), "review: envelope keys");
    let head = json!({"schema": doc["schema"], "batches": doc["batches"]});
    let want = json!({"schema": REVIEW_SCHEMA, "batches": BATCHES});
    assert_eq!(head, want, "review: schema and batch count");
    let auditor = doc["auditor"].as_str().map_or(0, |a| a.chars().count());
    assert!(auditor >= MIN_WHY, "review: no auditor sentence");
    let from = &doc["generated_from"];
    let commit = from["commit"].as_str().unwrap_or_default();
    let hex = commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit());
    assert!(
        hex && from["dirty"].is_boolean(),
        "review: no provenance stamp"
    );
    let (sampled, rows) = (
        sample["rows"].as_array().expect("rows"),
        doc["rows"].as_array().expect("rows"),
    );
    assert_eq!(rows.len(), sampled.len(), "one audited row per sampled row");
    for (row, s) in rows.iter().zip(sampled) {
        assert_eq!(
            row["id"], s["id"],
            "the review answers the sample in its order"
        );
        if let Some(why) = refusal(row) {
            panic!("review row {}: {why}", s["id"]);
        }
    }
}
