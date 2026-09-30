//! One batch's answer file, read against the batch it answers
//! (batches.rs): one JSON object per line, exactly `id`, `truth` and
//! `reason`, all strings — the id one of the batch's, answered once,
//! the truth one of its kind's words or `cannot_tell`, the reason
//! within REASON characters. Every refusal is named by batch and line
//! (or by id, for a question left unanswered) and none stops the read:
//! the assembler reports the whole list, so a batch is re-dispatched
//! once, not once per defect.

use super::batches::{Batch, legal};
use super::draw::text;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// A reason's length in characters, inclusive both ends.
pub const REASON: (usize, usize) = (20, 240);

/// The keys of an answer line, exactly.
const KEYS: [&str; 3] = ["id", "truth", "reason"];

/// One accepted answer.
pub struct Answer {
    pub truth: String,
    pub reason: String,
}

/// Whether a reason's length is inside REASON.
pub fn reason_fits(reason: &str) -> bool {
    (REASON.0..=REASON.1).contains(&reason.chars().count())
}

/// A line as its three strings, or why it is not an answer line.
fn fields(line: &str) -> Result<[String; 3], String> {
    let value: Value = serde_json::from_str(line).map_err(|_| "not a JSON object".to_string())?;
    let object = value
        .as_object()
        .ok_or_else(|| "not a JSON object".to_string())?;
    let keys: BTreeSet<&str> = object.keys().map(String::as_str).collect();
    if keys != BTreeSet::from(KEYS) {
        return Err(format!("fields {keys:?}, not exactly {KEYS:?}"));
    }
    let string = |k: &str| {
        object[k]
            .as_str()
            .map(str::to_string)
            .ok_or(format!("{k} is not a string"))
    };
    Ok([string("id")?, string("truth")?, string("reason")?])
}

/// Why an answer to a kind-`kind` question is refused, if it is.
fn refusal(kind: u64, truth: &str, reason: &str) -> Option<String> {
    if !legal(kind, truth) {
        return Some(format!(
            "truth `{truth}` is not an answer to a kind-{kind} question"
        ));
    }
    let n = reason.chars().count();
    (!reason_fits(reason))
        .then(|| format!("reason of {n} characters, not {}..={}", REASON.0, REASON.1))
}

/// The batch's accepted answers by id; every refusal pushed onto
/// `errs`. `file` is None when the batch has no answer file.
pub fn read(batch: &Batch, file: Option<&str>, errs: &mut Vec<String>) -> BTreeMap<String, Answer> {
    let n = batch.n;
    let Some(file) = file else {
        errs.push(format!("batch {n}: no answers file"));
        return BTreeMap::new();
    };
    let mut kinds = BTreeMap::new();
    for r in &batch.rows {
        kinds.insert(text(r, "audit"), r["kind"].as_u64().expect("kind"));
    }
    let (mut seen, mut out) = (BTreeSet::new(), BTreeMap::new());
    for (i, line) in file.lines().enumerate() {
        let at = format!("batch {n} line {}", i + 1);
        let [id, truth, reason] = match fields(line) {
            Ok(f) => f,
            Err(why) => {
                errs.push(format!("{at}: {why}"));
                continue;
            }
        };
        let Some(&kind) = kinds.get(id.as_str()) else {
            errs.push(format!("{at}: id {id} is not in this batch"));
            continue;
        };
        if !seen.insert(id.clone()) {
            errs.push(format!("{at}: id {id} answered twice"));
        } else if let Some(why) = refusal(kind, &truth, &reason) {
            errs.push(format!("{at}: id {id}: {why}"));
        } else {
            out.insert(id, Answer { truth, reason });
        }
    }
    for id in kinds.keys().filter(|id| !seen.contains(**id)) {
        errs.push(format!("batch {n}: id {id} unanswered"));
    }
    out
}
