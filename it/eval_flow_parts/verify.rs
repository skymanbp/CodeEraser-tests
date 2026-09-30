//! The frozen-sample verifier of the v2.31 flow exams (the gate in
//! eval_flow.rs calls it; the tamper battery runs it on forged copies):
//! a sample re-derived from its frozen slices, row by row — the
//! envelope, the allocation re-run from the slices' pools, every row's
//! hashes, identity, cell and unit, the per-cell counts, the audit
//! order.

use super::draw::{cell_of, text};
use super::{
    DOMAINS, FlowExam, MIN_PER_STRATUM, SCHEMAS, SLICES, assert_keys, fields, sample_constants,
    source_row, strata,
};
use crate::eval_lang_parts::Generated;
use crate::eval_support::assert_ranked;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// (corpus, path, unit nth): a frozen unit's address.
type UnitKey = (String, String, u64);

/// The frozen slices of one exam, in corpus order.
fn slices(exam: &FlowExam) -> Vec<(&'static str, Value)> {
    exam.corpora
        .iter()
        .map(|(name, _)| (*name, SLICES.load(exam, &exam.key(name))))
        .collect()
}

/// Every frozen unit row, by address.
fn frozen_units(slices: &[(&'static str, Value)]) -> BTreeMap<UnitKey, Value> {
    let mut units = BTreeMap::new();
    for (name, slice) in slices {
        for f in slice["files"].as_array().expect("files") {
            let path = text(f, "path").to_string();
            for u in f["units"].as_array().expect("units") {
                let nth = u["nth"].as_u64().expect("nth");
                units.insert((name.to_string(), path.clone(), nth), u.clone());
            }
        }
    }
    units
}

/// The allocation re-run from the slices' summaries: per cell
/// min(MIN_PER_STRATUM, the pool over every corpus).
fn allocation(slices: &[(&'static str, Value)]) -> BTreeMap<String, u64> {
    let mut pool: BTreeMap<String, u64> = BTreeMap::new();
    for (_, slice) in slices {
        for (cell, n) in slice["summary"]["pools"].as_object().expect("pools") {
            *pool.entry(cell.clone()).or_insert(0) += n.as_u64().expect("n");
        }
    }
    pool.into_iter()
        .map(|(c, n)| (c, n.min(MIN_PER_STRATUM)))
        .collect()
}

/// One frozen sample against its frozen slices.
pub fn verify_sample(exam: &FlowExam, doc: &Value) {
    let lang = exam.lang;
    let slices = slices(exam);
    let sources: Vec<Value> = slices.iter().map(|(n, s)| source_row(n, s)).collect();
    let want = json!({
        "schema": SCHEMAS.1, "lang": lang, "constants": sample_constants(),
        "sources": sources, "allocation": allocation(&slices),
    });
    assert_keys(lang, doc, &want);
    let units = frozen_units(&slices);
    let rows = doc["rows"].as_array().expect("rows");
    let (mut seen, mut drawn, mut per_cell) = (BTreeSet::new(), BTreeMap::new(), BTreeMap::new());
    for row in rows {
        check_row(exam, &units, row, &mut seen, &mut drawn);
        *per_cell.entry(cell_of(row)).or_insert(0u64) += 1;
    }
    for (cell, n) in want["allocation"].as_object().expect("allocation") {
        let have = per_cell.get(cell).copied().unwrap_or(0);
        assert_eq!(have, n.as_u64().expect("n"), "{lang}/{cell}: rows drawn");
    }
    assert!(
        rows.is_sorted_by_key(|r| text(r, "audit")),
        "{lang}: rows not in audit order"
    );
}

/// One row: hashes and rank (assert_ranked), the exam's identity, a
/// legal (kind, stratum), a frozen unit it echoes, its line inside the
/// unit, and never more rows of a cell than the unit's pool holds.
fn check_row(
    exam: &FlowExam,
    units: &BTreeMap<UnitKey, Value>,
    row: &Value,
    seen: &mut BTreeSet<String>,
    drawn: &mut BTreeMap<(UnitKey, String), u64>,
) {
    let rank = assert_ranked(row, DOMAINS, &fields(), seen);
    let corpus = text(row, "corpus");
    let tip = exam
        .tip(corpus)
        .unwrap_or_else(|| panic!("{rank}: {corpus} is no corpus of the exam"));
    assert_eq!(row["commit"], json!(tip), "{rank}: not the pinned commit");
    let (kind, stratum) = (row["kind"].as_u64().expect("kind"), text(row, "stratum"));
    let legal = stratum.len() == 1 && strata(kind).contains(&stratum.chars().next().expect("c"));
    assert!(legal, "{rank}: kind {kind} has no stratum {stratum}");
    let key = (
        corpus.to_string(),
        text(row, "path").to_string(),
        row["unit"].as_u64().expect("unit"),
    );
    let unit = units
        .get(&key)
        .unwrap_or_else(|| panic!("{rank}: no frozen unit at {key:?}"));
    assert_eq!(row["unit_name"], unit["name"], "{rank}: unit name");
    assert_eq!(row["unit_lines"], unit["lines"], "{rank}: unit lines");
    let dynamic = unit["dynamic"].as_bool() != Some(false);
    assert!(
        !dynamic,
        "{rank}: a dynamic unit drawn (the core judges none of it)"
    );
    let (line, nth) = (row["line"].as_u64().expect("line"), row["nth"].as_u64());
    let inside = unit["lines"][0].as_u64() <= Some(line) && Some(line) <= unit["lines"][1].as_u64();
    assert!(
        inside && nth >= Some(1),
        "{rank}: line {line} / nth {nth:?} outside the unit"
    );
    let cell = cell_of(row);
    let n = drawn.entry((key, cell.clone())).or_insert(0);
    *n += 1;
    assert!(
        unit["pools"][&cell].as_u64().is_some_and(|have| *n <= have),
        "{rank}: more {cell} rows drawn than the unit's pool holds"
    );
}
