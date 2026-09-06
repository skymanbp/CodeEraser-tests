//! Printed evidence and the frozen changeset ledger.

use super::GATE_MAX_PERCENT;
use super::Row;
use super::SCHEMA;
use super::tally::Tally;
use serde_json::{Value, json};

pub const HEADER: &str = "| 语料 | 事件 | 已审正常 | 未审 | 异常 | 拦截 | 误报（严格） | 漏 | 严格率 ‰ | 误报（宽） | 宽率 ‰ | 严格 CP 95 % | 宽 CP 95 % | 召回 ‰ |";

/// Every displayed count or interval comes from the same frozen row.
pub fn table_row(c: &Value) -> String {
    let name = c["corpus"].as_str().expect("corpus");
    let mut cells = vec![if name == "total" { "合计" } else { name }.to_string()];
    for key in [
        "events",
        "normal",
        "unreviewed",
        "abnormal",
        "intercepts",
        "false_strict",
        "misses",
        "strict_permille",
        "false_wide",
        "wide_permille",
    ] {
        cells.push(c[key].as_u64().expect(key).to_string());
    }
    for key in ["cp95_strict", "cp95_wide"] {
        let lo = c[key][0].as_f64().expect("lower bound");
        let hi = c[key][1].as_f64().expect("upper bound");
        cells.push(format!("{lo:.3}–{hi:.3} %"));
    }
    cells.push(
        c["recall_permille"]
            .as_u64()
            .map_or_else(|| "不适用".into(), |n| n.to_string()),
    );
    format!("| {} |", cells.join(" | "))
}

/// The ledger as the doc's own table, plus every intercept.
pub fn report(tallies: &[Tally], rows: &[Row]) {
    for r in rows {
        println!("  INTERCEPT {}", serde_json::to_string(r).expect("l2 row"));
    }
    println!("{HEADER}");
    println!("|{}", "---|".repeat(14));
    let all = Tally::fold("total", tallies);
    for t in tallies.iter().chain(std::iter::once(&all)) {
        println!("{}", table_row(&t.json()));
    }
    for t in tallies {
        println!("{} skipped {:?}", t.name, t.skipped);
    }
}

/// The frozen document. `promoted` is the §4.2 switch this batch does
/// NOT touch: false = no tier reads this class, so the gate enforces
/// the line as an implication rather than unconditionally.
pub fn document(tallies: &[Tally], rows: &[Row]) -> Value {
    let root = crate::common::repo_root();
    let (_, head) = crate::common::git_out(&root, &["rev-parse", "HEAD"]);
    let (_, status) = crate::common::git_out(&root, &["status", "--porcelain"]);
    json!({
        "schema": SCHEMA,
        "gate_max_percent": GATE_MAX_PERCENT,
        "promoted": false,
        "generated_from": {
            "ce": env!("CARGO_PKG_VERSION"),
            "commit": head.trim(),
            "dirty": !status.trim().is_empty(),
        },
        "method": "every first-parent commit of the three frozen commit slices replayed as the multi-file changeset the Stop audit sees: fourclass::session::commit_pairs over <sha>^..<sha> (renames as one pair, a copy record as an added file), both blob sides through tombstone::texts::load, judged by daemon::judge::Judge::judge_changeset — the seam Judge::classify itself calls, so the same classify_batch and the same fourclass/2 core link. A changeset that is not whole is not an event and is counted under a named skip.",
        "predicate": "intercept = the counterfactual deny: config::tier_of(.., \"observe\") == \"deny\" AND the core's suspicions non-empty (CE.FourClass.Verdict: novel lines inside a newly duplicated top-level unit span >= 20 AND deleted * 10 < novel). Cross-file relocations are evidence on the row, never the predicate: a clean refactor produces them by the hundred.",
        "ground_truth": "by sha off the frozen contracts: copy = a commit-slice row with a pair carrying copied:true (the only duplication-shaped positive the three corpora hold); normal = a reviewed commit-labels row and no copied pair; unreviewed = admitted by the slice, never individually reviewed. Strict rate = false / normal (calibration, below the 500 floor); wide rate = false_wide / (normal + unreviewed), the reading the plan's per-500 line takes. Recall is defined here — the ground truth has a positive, unlike the 600-sample gate.",
        "corpora": tallies.iter().map(Tally::json).collect::<Vec<_>>(),
        "totals": Tally::fold("total", tallies).json(),
        "rows": rows,
    })
}
