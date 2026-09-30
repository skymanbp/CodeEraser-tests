//! One file version judged for the flow ledger: lowered by the
//! product (flow::lower::lower_file) and judged whole by the real core
//! over the caller's link (flow::wire::judge — the refusal-driven
//! exclusion included), then read into the book's terms. A unit's key
//! is (name, parameter count, ordinal among its namesakes); a
//! finding's is (kind, anchor): kind 0 the first unreachable
//! statement's text, kind 1 the variable's name and the write's text,
//! kinds 2 / 3 the variable's name — every text with its whitespace
//! removed, so a re-indent or re-spacing is not an edit of the finding.
//! The text is the legend's (`stmt_text`, the statement's first line);
//! a finding's lines are its statements' first lines, or the
//! declaration's.
//!
//! A unit's version is read as its TOKEN sequence (the coordinator's
//! ruling, 2026-09-30): the text of every tree-sitter leaf whose start
//! line lies in the unit's lines, comment subtrees skipped, whitespace
//! never a leaf — so a commit that only reformats or re-comments a
//! unit does not edit it. Not the dedup token stream: that one folds
//! every identifier to ID and every literal to LIT, so a renamed
//! variable or a changed constant would read as no edit.

use super::shape::{Judgment, Key, Side, Span, View};
use crate::flow_precision::flagged::judge_file;
use codeeraser::corelink::Link;
use codeeraser::flow::lower::{Unit, lower_file};
use codeeraser::flow::wire::Finding;
use codeeraser::scan::ast::{children, parse_lang};
use codeeraser::scan::lang::Lang;
use std::collections::BTreeMap;

/// Whitespace removed.
pub fn fold(text: &str) -> String {
    text.split_whitespace().collect()
}

/// Every leaf's start line and text, in source order, comment subtrees
/// skipped (the language's scan table names its comment kinds).
fn leaves(text: &str, lang: Lang) -> Vec<(u32, String)> {
    let Some(tree) = parse_lang(text, lang) else {
        return Vec::new();
    };
    let comments = codeeraser::scan::spec::spec(lang).comment_kinds;
    let mut out = Vec::new();
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        if comments.contains(&node.kind()) {
            continue;
        }
        if node.child_count() > 0 {
            stack.extend(children(node).into_iter().rev());
            continue;
        }
        let bytes = &text.as_bytes()[node.byte_range()];
        let line = node.start_position().row as u32 + 1;
        out.push((line, String::from_utf8_lossy(bytes).into_owned()));
    }
    out
}

/// A finding's anchor and its lines, read off its unit's legend.
fn anchor(unit: &Unit, f: &Finding) -> (Key, Span) {
    let legend = &unit.legend;
    let stmt = |s: i64| legend.stmt_at[s as usize].0;
    let text = |s: i64| fold(&legend.stmt_text[s as usize]);
    let name = |v: i64| legend.var_name[v as usize].clone();
    let (anchor, span) = match f.kind {
        0 => {
            let lines = (f.seq..=f.seq_end).map(stmt);
            let span = (lines.clone().min(), lines.max());
            (text(f.seq), (span.0.unwrap_or(0), span.1.unwrap_or(0)))
        }
        1 => (
            format!("{} {}", name(f.v), text(f.seq)),
            (stmt(f.seq), stmt(f.seq)),
        ),
        _ => {
            let line = legend.var_at[f.v as usize].0;
            (name(f.v), (line, line))
        }
    };
    ((f.kind, anchor), span)
}

/// A unit's findings by key; two findings on one key (two dead stores
/// of one text) are one, over the union of their lines.
fn keys(unit: &Unit, found: &[Finding]) -> BTreeMap<Key, Span> {
    let mut out: BTreeMap<Key, Span> = BTreeMap::new();
    for f in found {
        let (key, (lo, hi)) = anchor(unit, f);
        let span = out.entry(key).or_insert((lo, hi));
        *span = (span.0.min(lo), span.1.max(hi));
    }
    out
}

/// A unit's token sequence: the leaves starting on its lines, joined
/// by a separator no leaf holds.
fn tokens(leaves: &[(u32, String)], unit: &Unit) -> String {
    let from = leaves.partition_point(|(l, _)| *l < unit.start_line);
    let to = leaves.partition_point(|(l, _)| *l <= unit.end_line);
    let texts: Vec<&str> = leaves[from..to.max(from)]
        .iter()
        .map(|(_, t)| t.as_str())
        .collect();
    texts.join("\u{1f}")
}

/// One version of a file, judged. A language with no flow table (none
/// here: the ledger reads the exams' languages) reads as no units.
pub fn side(link: &mut Link, text: &str, lang: Lang, at: &str) -> Side {
    let Some(done) = lower_file(text, lang) else {
        return Side::default();
    };
    let judged = judge_file(link, done, at);
    let (done, found) = (&judged.done, &judged.findings);
    let leaves = leaves(text, lang);
    let mut seen: BTreeMap<(String, u32), usize> = BTreeMap::new();
    let mut out = Side::default();
    for left_out in &done.unlowered {
        out.unkeyed.push(left_out.name.clone());
    }
    for unit in &done.units {
        let ord = seen.entry((unit.name.clone(), unit.params)).or_default();
        let key = (unit.name.clone(), unit.params, *ord);
        *ord += 1;
        let judgment = if judged.refused.contains_key(&unit.nth) {
            Judgment::Unjudged
        } else if unit.dynamic {
            Judgment::Dynamic
        } else {
            Judgment::Judged(keys(unit, found.get(&unit.nth).map_or(&[], Vec::as_slice)))
        };
        let tokens = tokens(&leaves, unit);
        out.units.insert(key, View { tokens, judgment });
    }
    out
}
