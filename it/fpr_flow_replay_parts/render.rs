//! The three tables docs/FPR-REPLAY.md quotes of the flow ledger — the
//! ledger table (rated kinds 0 / 1 / 2, both readings and their rates),
//! the per-kind table of the rated kinds, and the advisory kind 3's
//! own table (no rate) — each rendered from the frozen rows, the rates
//! derived here and never read from the doc.

use super::shape::{COUNTS, OUTCOME, RATED, READINGS, rate};
use crate::common::ledger;
use serde_json::{Value, json};

const STRICT_COLS: &str = "| 严·发现 | 严·真阳 | 严·误拦 | 严·移除 | 严·未定 |";
const NARROW_COLS: &str = "| 窄·发现 | 窄·真阳 | 窄·误拦 | 窄·移除 | 窄·未定 |";

/// Both readings of `src` written flat into `dst` (`strict_findings`
/// …), with each reading's rate as `<reading>_ppm`.
fn flatten(dst: &mut Value, src: &Value) {
    for r in READINGS {
        for k in OUTCOME.split_whitespace() {
            dst[format!("{r}_{k}")] = src[r][k].clone();
        }
        dst[format!("{r}_ppm")] = json!(rate(&src[r]));
    }
}

/// The flat column spec of both readings, `%rate` after each.
fn reading_spec(rates: bool) -> String {
    let cols = |r: &str| {
        let mut c: Vec<String> = OUTCOME
            .split_whitespace()
            .map(|k| format!("{r}_{k}"))
            .collect();
        if rates {
            c.push(format!("%{r}_ppm"));
        }
        c.join(" ")
    };
    READINGS.map(cols).join(" ")
}

/// A header line of `lead` columns, the two readings' columns (with a
/// rate column after each when `rates`) and `tail`, and its rule.
fn header(lead: &str, rates: bool, tail: &str) -> String {
    let (s, n) = if rates {
        (
            format!("{STRICT_COLS} 严格率 |"),
            format!("{NARROW_COLS} 窄率 |"),
        )
    } else {
        (STRICT_COLS.to_string(), NARROW_COLS.to_string())
    };
    let line = format!("{lead}{}{}{tail}", &s[1..], &n[1..]);
    let rule = "|---".repeat(line.matches('|').count() - 1) + "|";
    format!("{line}\n{rule}\n")
}

fn lang(c: &Value) -> &str {
    c["lang"].as_str().unwrap_or("?")
}

/// The ledger table: one row per language, over the rated kinds, with
/// kind 3's findings in the last column.
pub fn table(rows: &[Value]) -> String {
    let flat: Vec<Value> = rows
        .iter()
        .map(|c| {
            let mut d = c.clone();
            flatten(&mut d, c);
            d["advisory_findings"] = c["kinds"][RATED]["strict"]["findings"].clone();
            d
        })
        .collect();
    let lead = "| 语言 | 语料 | 提交 | 事件 | 判过的单元版本 | dynamic | 未判 |";
    ledger::table(
        &header(lead, true, " 类 3 发现 |"),
        |c| {
            let tip = c["tip"].as_str().unwrap_or("?");
            let corpus = c["corpus"].as_str().unwrap_or("?");
            format!("{} | {corpus} @ {}", lang(c), tip.get(..8).unwrap_or(tip))
        },
        &format!("{COUNTS} {} advisory_findings", reading_spec(true)),
        &flat,
    )
}

/// One row per language and kind, over `kinds`.
fn per_kind(rows: &[Value], kinds: std::ops::Range<usize>, rates: bool, lead: &str) -> String {
    let cells: Vec<Value> = rows
        .iter()
        .flat_map(|c| {
            kinds.clone().map(move |k| {
                let mut d = json!({"lang": c["lang"], "kind": k});
                flatten(&mut d, &c["kinds"][k]);
                d
            })
        })
        .collect();
    ledger::table(
        &header(lead, rates, ""),
        |d| format!("{} | {}", lang(d), d["kind"]),
        &reading_spec(rates),
        &cells,
    )
}

/// The rated kinds, one row per language and kind.
pub fn kind_table(rows: &[Value]) -> String {
    per_kind(rows, 0..RATED, true, "| 语言 | 类 |")
}

/// The advisory kind 3, one row per language, no rate.
pub fn advisory_table(rows: &[Value]) -> String {
    per_kind(rows, RATED..RATED + 1, false, "| 语言 | 类 |")
}
