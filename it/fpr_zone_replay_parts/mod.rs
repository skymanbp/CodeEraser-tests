//! The ledger half of the graded-zone FPR replay
//! (fpr_zone_replay.rs): what one in-zone landing records, the exact
//! arithmetic the ledger quotes — the per-million false-ask rate and
//! the Clopper–Pearson 95 % upper bound — and the table
//! docs/FPR-REPLAY.md carries. Split from the walk so both files stay
//! inside the repository's own file budget, the fpr_replay_parts
//! shape.

pub use crate::common::stats::{cp_upper_ppm, rate_ppm};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

pub const DOC: &str = "contracts/eval/fpr-zone-v1.json";
pub const SCHEMA: &str = "ce.eval-fpr-zone/1.0.0";
/// The plan §4.2 / §6 M4 admission line as parts per million of
/// events: one false intercept per hundred real normal edits.
pub const GATE_PPM: u64 = 10_000;

/// One in-zone landing: the write, the two lines it was measured
/// against, its position and the tier the map gave it. `shadowed` =
/// the write also crossed H, so guard.rs never reaches `zone_assess`
/// at all — the hard-budget class decides and the zone's ask is moot.
#[derive(serde::Serialize)]
pub struct Landed {
    pub sha: String,
    #[serde(rename = "file")]
    pub rel: String,
    pub lines: usize,
    pub soft: usize,
    pub hard: usize,
    pub permille: usize,
    pub tier: &'static str,
    pub shadowed: bool,
}

/// One corpus as the walk finished it.
pub struct Corpus {
    pub name: String,
    pub tip: String,
    pub commits: usize,
    pub events: usize,
    pub events_shared: usize,
    pub unreadable: usize,
    pub softs: BTreeMap<usize, usize>,
    pub rows: Vec<Landed>,
}

/// How many landings carry `tier`.
pub fn count(rows: &[Landed], tier: &str) -> usize {
    rows.iter().filter(|r| r.tier == tier).count()
}

/// Asks the hard budget already refuses — the zone decides nothing
/// there, so they are neither true nor false intercepts OF THIS RULE.
pub fn shadowed(rows: &[Landed]) -> usize {
    rows.iter()
        .filter(|r| r.tier == "ask" && r.shadowed)
        .count()
}

/// False asks: both corpora are all-normal by review, so every ask is
/// false except a shadowed one.
pub fn false_asks(rows: &[Landed]) -> usize {
    count(rows, "ask") - shadowed(rows)
}

impl Corpus {
    /// The frozen row. Every derived number is recomputed by the gate
    /// from the raw ones, so a hand-edited doc reddens.
    pub fn json(&self) -> Value {
        let k = false_asks(&self.rows);
        let softs: Map<String, Value> = self
            .softs
            .iter()
            .map(|(s, n)| (s.to_string(), json!(n)))
            .collect();
        json!({
            "name": self.name,
            "tip": self.tip,
            "commits": self.commits,
            "events": self.events,
            "events_shared": self.events_shared,
            "config_unreadable_commits": self.unreadable,
            "in_zone": self.rows.len(),
            "observe": count(&self.rows, "observe"),
            "warn": count(&self.rows, "warn"),
            "ask": count(&self.rows, "ask"),
            "ask_shadowed": shadowed(&self.rows),
            "false_asks": k,
            "rate_ppm": rate_ppm(k, self.events),
            "cp_upper_ppm": cp_upper_ppm(k, self.events),
            "soft_lines": softs,
            "intercepts": self
                .rows
                .iter()
                .filter(|r| r.tier == "ask")
                .collect::<Vec<_>>(),
        })
    }
}

/// A ppm as a percentage with four decimals, integer arithmetic only.
pub fn pct(ppm: u64) -> String {
    format!("{}.{:04} %", ppm / 10_000, ppm % 10_000)
}

/// The ledger table docs/FPR-REPLAY.md carries — one row per corpus,
/// printed by the instrument for the maintainer to paste (the page is
/// hand-maintained prose, not a generated block).
pub fn table(corpora: &[Value]) -> String {
    let mut s = String::from(
        "| 语料 | 提交 | 事件 | 同分母事件 | 落区 | observe | warn | ask | ask 被硬线遮蔽 | 误拦 ask | 率 | CP 95 % 上界 |\n|---|---|---|---|---|---|---|---|---|---|---|---|\n",
    );
    for c in corpora {
        let n = |k: &str| c[k].as_u64().unwrap_or_default();
        s += &format!(
            "| {} @ {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            c["name"].as_str().unwrap_or("?"),
            c["tip"].as_str().unwrap_or("?"),
            n("commits"),
            n("events"),
            n("events_shared"),
            n("in_zone"),
            n("observe"),
            n("warn"),
            n("ask"),
            n("ask_shadowed"),
            n("false_asks"),
            pct(n("rate_ppm")),
            pct(n("cp_upper_ppm")),
        );
    }
    s
}
