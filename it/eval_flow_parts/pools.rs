//! The candidate pools of one lowered unit (booklet §5.5): pure
//! functions over its four tables and legend, never a core verdict, so
//! a universe freezes before any judgment exists. Each item carries the
//! question's anchor — the line, the nth same-shaped anchor on it (a
//! statement in column order, a write of the name, a declaration of
//! the name) and the variable name — which is what an auditor reads a
//! question by — and the statement and variable it stands for, which is
//! how a precision doc maps a finding back to it (precision.rs reads
//! the item, never a second derivation of the anchor).

use super::cell;
use codeeraser::flow::lower::Unit;
use std::collections::BTreeMap;

/// One pool item: a question before it is sampled, with the rows it
/// stands for — `seq` the statement (kind 0: the question; kind 1: the
/// write; kinds 2 / 3: −1) and `v` the variable (kind 0: −1).
pub struct Item {
    pub kind: u64,
    pub stratum: char,
    pub line: u32,
    pub nth: u64,
    pub name: String,
    pub seq: i64,
    pub v: i64,
}

/// The terminal leaves: return, throw, break, continue, goto,
/// noreturn-call.
const TERMINAL: [i64; 6] = [9, 10, 11, 12, 13, 15];
/// The structure statements a leaf may hide in: if, loop, switch, try.
const STRUCTURE: [i64; 4] = [2, 3, 4, 6];
/// A loop whose head is always true (stmts flag bit 1).
const INFINITE: i64 = 1 << 1;
/// A variable the core never judges: captured, ignored, address-taken
/// (vars flag bits 1, 2, 3).
const EXEMPT: i64 = 0b1110;
const PARAM: i64 = 1;

/// Every item of the unit, kind by kind; none of a dynamic unit —
/// the core judges none of it, so each of its questions could only
/// be answered unjudged (booklet §13 item 22).
pub fn items(u: &Unit) -> Vec<Item> {
    if u.dynamic {
        return Vec::new();
    }
    let mut out = unreachable(u);
    out.extend(dead_stores(u));
    out.extend(unused(u, false));
    out.extend(unused(u, true));
    out
}

/// The unit's pool per cell, every cell present.
pub fn counts(u: &Unit) -> BTreeMap<String, u64> {
    let mut pools: BTreeMap<String, u64> = super::cells().into_iter().map(|c| (c, 0)).collect();
    for item in items(u) {
        *pools.entry(cell(item.kind, item.stratum)).or_insert(0) += 1;
    }
    pools
}

fn synthetic(u: &Unit, seq: i64) -> bool {
    u.legend.stmt_text[seq as usize].starts_with("<synthetic:")
}

/// Kind 0: a statement whose previous sibling is a terminal leaf or an
/// infinite loop (A), a structure statement holding a terminal leaf
/// (B) or another structure statement (C). A synthetic statement is
/// never the question.
fn unreachable(u: &Unit) -> Vec<Item> {
    let mut out = Vec::new();
    for &[seq, parent, ..] in &u.stmts {
        if synthetic(u, seq) {
            continue;
        }
        let prev = u
            .stmts
            .iter()
            .filter(|r| r[1] == parent && r[0] < seq)
            .map(|r| r[0])
            .max();
        if let Some(stratum) = prev.and_then(|p| after(u, p)) {
            let (line, nth) = stmt_anchor(u, seq);
            out.push(Item {
                kind: 0,
                stratum,
                line,
                nth,
                name: String::new(),
                seq,
                v: -1,
            });
        }
    }
    out
}

/// The stratum a statement after `prev` falls in, if any.
fn after(u: &Unit, prev: i64) -> Option<char> {
    let [_, _, kind, flags, _] = u.stmts[prev as usize];
    if TERMINAL.contains(&kind) || kind == 3 && flags & INFINITE != 0 {
        Some('A')
    } else if STRUCTURE.contains(&kind) {
        Some(if holds_terminal(u, prev) { 'B' } else { 'C' })
    } else {
        None
    }
}

/// Whether a terminal leaf sits anywhere under `root`.
fn holds_terminal(u: &Unit, root: i64) -> bool {
    u.stmts.iter().any(|&[seq, _, kind, ..]| {
        TERMINAL.contains(&kind) && {
            let mut s = seq;
            while s > root {
                s = u.stmts[s as usize][1];
            }
            s == root
        }
    })
}

/// A statement's line and its place among the source statements of
/// that line, in (column, seq) order, 1-based.
fn stmt_anchor(u: &Unit, seq: i64) -> (u32, u64) {
    let nth = nth_on_line(&u.legend.stmt_at, seq as usize, |t| !synthetic(u, t as i64));
    (u.legend.stmt_at[seq as usize].0, nth)
}

/// The 1-based place of `at` among the places on its line that
/// `counts`, in (column, index) order — the anchor's nth.
fn nth_on_line(places: &[(u32, u32)], at: usize, counts: impl Fn(usize) -> bool) -> u64 {
    let (line, col) = places[at];
    let before = places
        .iter()
        .enumerate()
        .filter(|&(i, &(l, c))| l == line && (c, i) < (col, at) && counts(i))
        .count();
    before as u64 + 1
}

/// Kind 1: each write (mode 1 / 2) of a non-exempt variable that is
/// read somewhere — its next access a pure write (A: mode 1; a
/// readwrite reads the store first), none (B), a read or readwrite (C).
fn dead_stores(u: &Unit) -> Vec<Item> {
    let mut out = Vec::new();
    for &[v, _, flags] in &u.vars {
        if flags & EXEMPT != 0 {
            continue;
        }
        let acc: Vec<(i64, i64)> = u
            .uses
            .iter()
            .filter(|r| r[1] == v)
            .map(|r| (r[0], r[2]))
            .collect();
        if !acc.iter().any(|&(_, m)| m != 1) {
            continue;
        }
        for (i, &(seq, mode)) in acc.iter().enumerate() {
            if mode == 0 {
                continue;
            }
            let stratum = match acc.get(i + 1) {
                Some(&(_, 1)) => 'A',
                None => 'B',
                Some(_) => 'C',
            };
            let line = u.legend.stmt_at[seq as usize].0;
            let nth = acc[..i]
                .iter()
                .filter(|&&(s, m)| m != 0 && u.legend.stmt_at[s as usize].0 == line)
                .count() as u64
                + 1;
            out.push(Item {
                kind: 1,
                stratum,
                line,
                nth,
                name: u.legend.var_name[v as usize].clone(),
                seq,
                v,
            });
        }
    }
    out
}

/// Kinds 2 / 3: each non-exempt local (or parameter) with no read
/// (A) or some read (B), anchored at its declaration — the nth
/// declaration of that name on its line, in (column, v) order.
fn unused(u: &Unit, params: bool) -> Vec<Item> {
    let mut out = Vec::new();
    for &[v, _, flags] in &u.vars {
        if (flags & PARAM == PARAM) != params || flags & EXEMPT != 0 {
            continue;
        }
        let read = u.uses.iter().any(|r| r[1] == v && r[2] != 1);
        let name = &u.legend.var_name[v as usize];
        let same = |w: usize| u.legend.var_name[w] == *name;
        out.push(Item {
            kind: 2 + params as u64,
            stratum: if read { 'B' } else { 'A' },
            line: u.legend.var_at[v as usize].0,
            nth: nth_on_line(&u.legend.var_at, v as usize, same),
            name: name.clone(),
            seq: -1,
            v,
        });
    }
    out
}
