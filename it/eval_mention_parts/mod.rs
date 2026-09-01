//! The booklet's self census row, written by the run that measures it.
//!
//! Methodology booklet 13 states the rule in prose — the self row "is
//! re-taken on the commit that ships the text … and moves with the
//! tree by design". Nothing executed it. The row was taken at 32b23c2
//! and shipped unchanged through v1.3.0, v1.3.1, v1.3.2, v1.4.0 and
//! v1.4.1 — 63 commits and 25 changed source files later — while the
//! restated chips under the table read that same stale table and were
//! byte-consistent with it. Every gate in reach compared the page
//! with its own generator; none of them read the tree.
//!
//! The measurement this needs is one CI already takes (the self
//! corpus leg is not `--ignored`), so executing the rule costs
//! nothing: the leg renders the rows from that run and compares,
//! `CE_BLESS=1` rewrites them. Only digits move, which is the fixed
//! point the prose claims — a re-bless changes no line of the tree
//! and therefore no reading of it.

use crate::mention_universe::Formula;
use serde_json::Value;

const BOOKLET: &str = "docs/reference/methodology/13-unmentioned-declaration-advisory.md";
/// The block's first cell, and the shape of a language continuation
/// row (empty corpus, empty universe).
const HEAD: &str = "| self @ this commit |";
const CONT: &str = "| | |";

/// A percentage to one decimal. A rate over an empty population is
/// 0.0 rather than NaN — the survival column takes one whenever a
/// language declares nothing this run.
fn pct(n: usize, d: usize) -> String {
    if d == 0 {
        "0.0".to_string()
    } else {
        format!("{:.1}", n as f64 * 100.0 / d as f64)
    }
}

fn num(r: &Value, a: &str, b: &str) -> usize {
    r[a][b].as_u64().unwrap_or(0) as usize
}

/// The universe cell: U, then the listing and every subtraction that
/// fired, in the order the walk asks its rules. A term that took
/// nothing is not spelled — naming a zero would invite the reader to
/// read it as a rule that ran and found something.
fn universe_cell(f: &Formula) -> String {
    let t = &f.terms;
    let subs: String = [
        (t.named_cut, "named-cut"),
        (t.nested, "nested"),
        (t.pattern_ignored, "pattern-ignored"),
        (t.excluded, "excluded"),
        (t.absent, "absent"),
        (t.oversize, "oversize"),
        (t.binary, "early-NUL"),
    ]
    .iter()
    .filter(|(n, _)| *n > 0)
    .map(|(n, what)| format!(" \u{2212} {n} {what}"))
    .collect();
    format!("{} ({}{subs})", f.universe(), f.listed)
}

/// One language's five columns: the domain and its exported half,
/// what survived every veto and its exported half, the survival rate,
/// the collision-saved share of the survivors, and the same count
/// over the by-other vetoes it partitions.
fn lang_cells(r: &Value) -> Vec<String> {
    let (decl, decl_x) = (num(r, "declared", "all"), num(r, "declared", "exported"));
    let (unm, unm_x) = (
        num(r, "unmentioned", "all"),
        num(r, "unmentioned", "exported"),
    );
    let (saved, other) = (
        num(r, "vetoed", "collision_saved"),
        num(r, "vetoed", "other"),
    );
    // no survivors, no rate: `0 / 0 = 0.0 %` reads as a measured share
    let saved_cell = if unm == 0 {
        format!("{saved} / {unm}")
    } else {
        format!("{saved} / {unm} = {} %", pct(saved, unm))
    };
    vec![
        format!("{decl} ({decl_x})"),
        format!("{unm} ({unm_x})"),
        format!("{} %", pct(unm, decl)),
        saved_cell,
        format!("{saved} / {other}"),
    ]
}

/// One table row, padded the way the page writes it: an empty cell is
/// a single space, never two.
fn row(cells: &[String]) -> String {
    let inner: Vec<String> = cells
        .iter()
        .map(|c| {
            if c.is_empty() {
                " ".to_string()
            } else {
                format!(" {c} ")
            }
        })
        .collect();
    format!("|{}|", inner.join("|"))
}

/// The self rows as this tree reads, largest domain first so the page
/// opens on the language that carries the corpus; the corpus and
/// universe cells ride the first row only, as the table's other
/// multi-language corpora write them.
pub fn self_rows(f: &Formula, rates: &Value) -> Vec<String> {
    let mut langs: Vec<(&String, &Value)> = rates
        .as_object()
        .expect("the census is an object keyed by language")
        .iter()
        .collect();
    langs.sort_by_key(|(name, r)| {
        (
            std::cmp::Reverse(num(r, "declared", "all")),
            (*name).clone(),
        )
    });
    langs
        .iter()
        .enumerate()
        .map(|(i, (name, r))| {
            let head = match i {
                0 => vec!["self @ this commit".to_string(), universe_cell(f)],
                _ => vec![String::new(), String::new()],
            };
            let mut cells = head;
            cells.push((*name).clone());
            cells.extend(lang_cells(r));
            row(&cells)
        })
        .collect()
}

/// The booklet's self block against this run. The block is found by
/// its first cell and runs to the last continuation row, so a
/// language entering or leaving the corpus moves this table and
/// nothing else.
pub fn check_booklet(f: &Formula, rates: &Value) {
    let path = crate::common::repo_root().join(BOOKLET);
    let text = std::fs::read_to_string(&path).expect("the booklet");
    let lines: Vec<&str> = text.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.starts_with(HEAD))
        .unwrap_or_else(|| panic!("{BOOKLET} has no {HEAD:?} row"));
    let n = 1 + lines[at + 1..]
        .iter()
        .take_while(|l| l.starts_with(CONT))
        .count();
    let (have, want) = (lines[at..at + n].join("\n"), self_rows(f, rates).join("\n"));
    if have == want {
        return;
    }
    assert!(
        crate::facts::blessing(),
        "the booklet's self census row is not this tree's reading — the \
         page says it is re-taken on the commit that ships it, so \
         CE_BLESS=1 and commit the rewrite (the restated chips below \
         the table read this block, so re-bless facts_ after)\n\
         page:\n{have}\ntree:\n{want}"
    );
    std::fs::write(&path, text.replacen(&have, &want, 1)).expect("bless the booklet");
}
