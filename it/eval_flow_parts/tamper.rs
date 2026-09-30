//! The flow sample's tamper battery (G9): the pristine doc passes;
//! a forged path, stratum, rank or audit hash and a dropped row refuse
//! through the shared field-mutation frame, and a forged unit, a
//! forged kind, a swapped pair and a row past its cell's pool refuse
//! through the forgery frame — the last three re-ranked after the
//! edit, so the refusal is the semantic check's, not the hash's. A row
//! moved onto a dynamic unit of the C universe refuses by that reason
//! (the unit's empty pool would refuse it too; the message tells which
//! check spoke).

use super::{SLICES, draw, exam, verify::verify_sample};
use crate::eval_support::{Forgery, assert_forgeries_refused, assert_tampering_refused};
use serde_json::{Value, json};
use std::panic::{AssertUnwindSafe, catch_unwind};

/// The field mutations of row 0, `field=value` each: the frame labels
/// one "a forged <field>".
const MUTATIONS: &str = "path=forged/path.py stratum=Z rank=0000 audit=0000";

fn rows(doc: &mut Value) -> &mut Vec<Value> {
    doc["rows"].as_array_mut().expect("rows")
}

/// Row 0 with one field replaced and both hashes re-derived.
fn forge(doc: &mut Value, key: &str, value: Value) {
    let row = &mut doc["rows"][0];
    row[key] = value;
    *row = draw::ranked(row);
}

/// A copy of row 0 anchored a thousand lines later, re-ranked: one
/// more row of its cell than the allocation grants.
fn past_pool(doc: &mut Value) {
    let mut extra = doc["rows"][0].clone();
    extra["nth"] = json!(extra["nth"].as_u64().unwrap_or(0) + 1000);
    rows(doc).push(draw::ranked(&extra));
}

/// Row 0 moved onto the first dynamic unit of the C universe, its
/// line inside that unit, re-ranked.
fn onto_dynamic(doc: &mut Value) {
    let exam = exam("c");
    let corpus = exam.corpora[0].0;
    let slice = SLICES.load(exam, &exam.key(corpus));
    let files = slice["files"].as_array().expect("files");
    let (file, unit) = files
        .iter()
        .flat_map(|f| {
            f["units"]
                .as_array()
                .expect("units")
                .iter()
                .map(move |u| (f, u))
        })
        .find(|(_, u)| u["dynamic"] == json!(true))
        .expect("a dynamic unit in the C universe");
    let row = &mut doc["rows"][0];
    row["corpus"] = json!(corpus);
    row["path"] = file["path"].clone();
    row["unit"] = unit["nth"].clone();
    row["unit_name"] = unit["name"].clone();
    row["unit_lines"] = unit["lines"].clone();
    row["line"] = unit["lines"][0].clone();
    *row = draw::ranked(row);
}

/// A sample row on a dynamic unit refuses, and by the dynamic check.
fn assert_dynamic_refused() {
    let exam = exam("c");
    let mut forged = exam.sample();
    onto_dynamic(&mut forged);
    let err = catch_unwind(AssertUnwindSafe(|| verify_sample(exam, &forged)))
        .expect_err("a dynamic unit drawn must refuse");
    let said = err.downcast_ref::<String>().cloned().unwrap_or_default();
    assert!(
        said.contains("a dynamic unit drawn"),
        "refused for another reason: {said}"
    );
}

pub fn assert_flow_sample_tampering() {
    assert_dynamic_refused();
    let exam = exam("python");
    let pristine = exam.sample();
    let check = |doc: &Value| verify_sample(exam, doc);
    let labels: Vec<(String, String)> = MUTATIONS
        .split_whitespace()
        .map(|m| m.split_once('=').expect("field=value"))
        .map(|(f, v)| (format!("a forged {f}"), v.to_string()))
        .collect();
    let mutations: Vec<(&str, &str, &str)> = MUTATIONS
        .split_whitespace()
        .zip(&labels)
        .map(|(m, (label, v))| {
            (
                m.split_once('=').expect("field").0,
                v.as_str(),
                label.as_str(),
            )
        })
        .collect();
    assert_tampering_refused(&pristine, &mutations, &check);
    let forgeries: [Forgery; 4] = [
        (&|d| forge(d, "unit", json!(999_999)), "a forged unit"),
        (&|d| forge(d, "kind", json!(7)), "a forged kind"),
        (&|d| rows(d).swap(0, 1), "a swapped pair"),
        (&past_pool, "a row past its pool"),
    ];
    assert_forgeries_refused(&pristine, &forgeries, &check);
}
