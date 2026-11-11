//! The ledger half of the hard-budget FPR replay (fpr_budget_replay.rs):
//! what one write past the line records, the two readings the ledger
//! quotes with their Clopper–Pearson bounds, and the table
//! docs/FPR-REPLAY.md carries — the fpr_zone_replay_parts shape, shared
//! with the gate so the frozen row and its recomputation cannot drift.
//!
//! The two readings (REVIEW-1 Q1). `all`: every refusal counted —
//! crossing + growth over events. `tp`: a refusal is a true positive
//! when the write takes the file ACROSS the line (the wall `ce scan`
//! goes from pass to fail on that file because of this write), and a
//! growth of a file the parent already failed on is counted on the
//! false side. The second reading is a DEFINITION, not a measurement
//! error: the rule is exact arithmetic, so "false" there means only
//! that the wall's verdict on that file did not change.

use crate::common::ledger::{self, Ledger, corpus_at};
use crate::common::stats::{cp_upper_ppm, rate_ppm};
use serde_json::{Value, json};

pub const DOC: &str = "contracts/eval/fpr-budget-v1.json";
pub const SCHEMA: &str = "ce.eval-fpr-budget/1.0.0";
pub use crate::fpr_zone_replay_parts::GATE_PPM;

pub const LEDGER: Ledger =
    Ledger::gated(DOC, SCHEMA, "cli/tests/it/fpr_budget_replay.rs", GATE_PPM);

/// One write past its line: the file's size before and after, the line,
/// and what the hook's rule (`guard::zone::budget_kind`) made of it.
#[derive(serde::Serialize)]
pub struct Over {
    pub sha: String,
    #[serde(rename = "file")]
    pub rel: String,
    pub before: usize,
    pub after: usize,
    pub cap: usize,
    pub kind: &'static str,
}

/// One corpus as the walk finished it.
pub struct Corpus {
    pub name: String,
    pub tip: String,
    pub commits: usize,
    pub events: usize,
    pub unreadable: usize,
    pub rows: Vec<Over>,
}

/// The rows of one kind.
pub fn kinds(rows: &[Value], kind: &str) -> usize {
    rows.iter().filter(|r| r["kind"] == kind).count()
}

/// The derived numbers of a row, from the raw counts — one owner, read
/// by the instrument when it writes the row and by the gate when it
/// re-derives it.
pub fn derived(events: usize, rows: &[Value]) -> Vec<(&'static str, u64)> {
    let (cross, grow) = (kinds(rows, "crossing"), kinds(rows, "growth"));
    let refused = cross + grow;
    vec![
        ("over", rows.len() as u64),
        ("crossing", cross as u64),
        ("growth", grow as u64),
        ("carried", kinds(rows, "carried") as u64),
        ("old_ppm", rate_ppm(rows.len(), events)),
        ("all_ppm", rate_ppm(refused, events)),
        ("all_cp_upper_ppm", cp_upper_ppm(refused, events)),
        ("tp_false_ppm", rate_ppm(grow, events)),
        ("tp_cp_upper_ppm", cp_upper_ppm(grow, events)),
    ]
}

impl Corpus {
    /// The frozen row: the raw counts, every write past the line, and the
    /// derived numbers the gate recomputes from those rows.
    pub fn json(&self) -> Value {
        let rows: Vec<Value> = self
            .rows
            .iter()
            .map(|r| serde_json::to_value(r).expect("a row"))
            .collect();
        let mut row = json!({
            "name": self.name,
            "tip": self.tip,
            "commits": self.commits,
            "events": self.events,
            "config_unreadable_commits": self.unreadable,
        });
        for (k, v) in derived(self.events, &rows) {
            row[k] = json!(v);
        }
        row["rows"] = json!(rows);
        row
    }
}

/// The ledger table docs/FPR-REPLAY.md carries — one row per corpus,
/// printed by the instrument for the maintainer to paste.
pub fn table(corpora: &[Value]) -> String {
    ledger::table(
        "| 语料 | 提交 | 事件 | 写后超线（旧规则全拒） | 越线 | 增长 | 承载 | 旧规则率 | 率（每次拒绝） | CP 95 % 上界 | 率（越线为真阳） | CP 95 % 上界 |\n|---|---|---|---|---|---|---|---|---|---|---|---|\n",
        corpus_at,
        "commits events over crossing growth carried %old_ppm %all_ppm %all_cp_upper_ppm %tp_false_ppm %tp_cp_upper_ppm",
        corpora,
    )
}
