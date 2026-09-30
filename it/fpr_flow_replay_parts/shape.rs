//! The flow ledger's shared shapes — a leaf every module of the
//! directory reads and none reads back: the unit, finding and outcome
//! types (book.rs, lane.rs, side.rs), the frozen row's keys and the
//! rate (mod.rs, render.rs, the gate).

use crate::common::stats::rate_ppm;
use serde_json::Value;
use std::collections::BTreeMap;

/// A unit within its file: its name, its parameter count, and its
/// ordinal among the file's units of that name and count (overloads
/// and anonymous units of one arity would otherwise share one key).
pub type UnitKey = (String, u32, usize);
/// A finding within its unit: its kind and its anchor.
pub type Key = (u8, String);
/// 1-based lines, inclusive.
pub type Span = (u32, u32);
/// One reading's outcome of one kind, cell by lane::Fate's order.
pub type Outcome = [usize; 5];
/// Each kind's (strict, narrow) outcomes.
pub type Kinds = [[Outcome; 2]; 4];

/// What the core said of one unit version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Judgment {
    Judged(BTreeMap<Key, Span>),
    Dynamic,
    Unjudged,
}

/// One unit version: its token sequence (side.rs) and its judgment.
#[derive(Debug, Clone)]
pub struct View {
    pub tokens: String,
    pub judgment: Judgment,
}

/// One judged file version: every unit the lowering keyed, and the
/// names of the units it left out (no parameter count, so no key: a
/// unit of that name in the child is not read as gone).
#[derive(Debug, Default, Clone)]
pub struct Side {
    pub units: BTreeMap<UnitKey, View>,
    pub unkeyed: Vec<String>,
}

impl Side {
    /// The judged units and their findings.
    pub fn judged(&self) -> impl Iterator<Item = (&UnitKey, &BTreeMap<Key, Span>)> {
        self.units.iter().filter_map(|(u, v)| match &v.judgment {
            Judgment::Judged(found) => Some((u, found)),
            _ => None,
        })
    }
}

/// A row's scalar counts, in the frozen row's order.
pub const COUNTS: &str = "commits events units_judged dynamic_skipped unjudged";

/// One reading's counts (lane::Fate's order).
pub const OUTCOME: &str = "findings resolved_tp resolved_false removed undetermined";

/// The two readings, by their row keys (lane::STRICT, lane::NARROW).
pub const READINGS: [&str; 2] = ["strict", "narrow"];

/// The kinds a rate is read over — the three the precision gate
/// reads; kind 3 (unused parameter) is advisory, recorded apart.
pub const RATED: usize = 3;

/// One reading's rate, false over (true + false), per million.
pub fn rate(o: &Value) -> u64 {
    let n = |k: &str| o[k].as_u64().unwrap_or_default() as usize;
    let (tp, fp) = (n("resolved_tp"), n("resolved_false"));
    rate_ppm(fp, tp + fp)
}
