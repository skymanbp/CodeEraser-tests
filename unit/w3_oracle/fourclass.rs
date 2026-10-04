//! FROZEN ORACLE — the fourclass L1 judgment as the measuring side
//! computed it at 093aede4, copied statement for statement from
//! cli/src/fourclass/model.rs (`classify`, `hash_lines`,
//! `sig_contents`, `moved_line`, `relocated`), decls.rs (`tally`,
//! `one_sided`, `tables`), stacking.rs (`dup_spans`, `top_level_spans`)
//! and batch.rs (`side_runs`, `MAX_BRIDGE`). Plan v2.33 W3 moved the
//! judgment into the core (CE.FourClass.Moves); this copy stays as the
//! differential oracle the core road is checked against
//! (unit/fourclass/moves_differential.rs). Never edit the bodies: a
//! change here is a change to what "the same behaviour" means.

use crate::dedup::tokens::fnv1a;
use crate::fourclass::batch::Side;
use crate::fourclass::decls::Decl;
use crate::fourclass::diff;
use crate::fourclass::units::{self, Unit};
use crate::fourclass::{
    ChangedLines, Classification, FourClass, MovedLine, alnum_width, significant,
};
use crate::scan::lang::Lang;
use std::collections::{BTreeMap, HashMap, HashSet};

pub fn classify(before: &str, after: &str, lang: Lang) -> Classification {
    let a: Vec<&str> = before.lines().collect();
    let b: Vec<&str> = after.lines().collect();
    let d = diff::diff(&hash_lines(&a), &hash_lines(&b));

    let removed_sig: HashSet<&str> = sig_contents(&a, &d.removed);
    let added_sig: HashSet<&str> = sig_contents(&b, &d.added);
    let before_units = units::segments(before, lang);
    let after_units = units::segments(after, lang);

    let mut counts = FourClass::default();
    let mut moved = Vec::new();
    for &i in &d.removed {
        if significant(a[i]) && added_sig.contains(a[i].trim()) {
            counts.removed_moved += 1;
            moved.push(moved_line(i, true, &before_units));
        } else {
            counts.removed_deleted += 1;
        }
    }
    for &j in &d.added {
        if significant(b[j]) && removed_sig.contains(b[j].trim()) {
            counts.added_moved += 1;
            moved.push(moved_line(j, false, &after_units));
        } else {
            counts.added_novel += 1;
        }
    }
    let relocated_units = relocated(&moved, &before_units, &after_units, &d);
    let decls = tables(&before_units, &after_units);
    let changed = ChangedLines {
        removed: d.removed.iter().map(|&i| i + 1).collect(),
        added: d.added.iter().map(|&j| j + 1).collect(),
    };
    Classification {
        counts,
        moved,
        relocated_units,
        decls,
        changed,
        degraded: d.degraded,
    }
}

fn hash_lines(lines: &[&str]) -> Vec<u64> {
    use std::hash::{DefaultHasher, Hash, Hasher};
    lines
        .iter()
        .map(|l| {
            let mut h = DefaultHasher::new();
            l.hash(&mut h);
            h.finish()
        })
        .collect()
}

fn sig_contents<'s>(lines: &[&'s str], changed: &[usize]) -> HashSet<&'s str> {
    changed
        .iter()
        .map(|&i| lines[i].trim())
        .filter(|t| t.chars().any(char::is_alphanumeric))
        .collect()
}

fn moved_line(idx: usize, removed: bool, side_units: &[Unit]) -> MovedLine {
    MovedLine {
        line: idx + 1,
        removed,
        unit: units::owner(side_units, idx + 1).map(|u| u.key.clone()),
    }
}

fn relocated(
    moved: &[MovedLine],
    before_units: &[Unit],
    after_units: &[Unit],
    d: &diff::DiffLines,
) -> Vec<String> {
    let moved_of = |removed: bool, key: &str| {
        moved
            .iter()
            .filter(|m| m.removed == removed && m.unit.as_deref() == Some(key))
            .count()
    };
    let changed_in = |unit: &Unit, changed: &[usize]| {
        changed
            .iter()
            .filter(|&&i| unit.start_line <= i + 1 && i < unit.end_line)
            .count()
    };
    let mut out = Vec::new();
    for bu in before_units {
        let Some(au) = after_units.iter().find(|u| u.key == bu.key) else {
            continue;
        };
        let (rm, ad) = (changed_in(bu, &d.removed), changed_in(au, &d.added));
        if rm + ad > 0 && moved_of(true, &bu.key) == rm && moved_of(false, &au.key) == ad {
            out.push(bu.key.clone());
        }
    }
    out
}

fn tally(units: &[Unit]) -> BTreeMap<(&str, i64), usize> {
    let mut out: BTreeMap<(&str, i64), usize> = BTreeMap::new();
    for u in units {
        *out.entry((u.key.as_str(), u.kind)).or_default() += 1;
    }
    out
}

fn one_sided(side: &[Unit], other: &[Unit]) -> Vec<Decl> {
    let (mine, theirs) = (tally(side), tally(other));
    let mut out: Vec<Decl> = side
        .iter()
        .filter(|u| {
            let k = (u.key.as_str(), u.kind);
            mine[&k] == 1 && !theirs.contains_key(&k)
        })
        .map(|u| Decl {
            key: u.key.clone(),
            kind: u.kind,
            start: u.start_line,
            end: u.end_line,
        })
        .collect();
    out.sort_by(|a, b| (&a.key, a.kind).cmp(&(&b.key, b.kind)));
    out
}

pub fn tables(before: &[Unit], after: &[Unit]) -> (Vec<Decl>, Vec<Decl>) {
    (one_sided(before, after), one_sided(after, before))
}

pub fn dup_spans(before_text: &str, after_text: &str, lang: Lang) -> Vec<[u64; 3]> {
    let before = top_level_spans(before_text, lang);
    let mut out: Vec<[u64; 3]> = top_level_spans(after_text, lang)
        .into_iter()
        .filter(|(k, spans)| spans.len() >= 2 && spans.len() > before.get(k).map_or(0, Vec::len))
        .flat_map(|(k, spans)| {
            let hash = fnv1a(k.as_bytes());
            spans
                .into_iter()
                .map(move |(s, e)| [hash, s as u64, e as u64])
        })
        .collect();
    out.sort_unstable();
    out
}

fn top_level_spans(text: &str, lang: Lang) -> HashMap<String, Vec<(usize, usize)>> {
    let all = units::segments(text, lang);
    let top_level = |u: &units::Unit| {
        !all.iter().any(|v| {
            (v.start_line < u.start_line && u.end_line <= v.end_line)
                || (v.start_line <= u.start_line && u.end_line < v.end_line)
        })
    };
    all.iter()
        .filter(|u| {
            top_level(u) && !u.key.starts_with("(anonymous)") && !u.key.starts_with("impl ")
        })
        .fold(
            HashMap::new(),
            |mut m: HashMap<String, Vec<(usize, usize)>>, u| {
                m.entry(u.key.clone())
                    .or_default()
                    .push((u.start_line, u.end_line));
                m
            },
        )
}

/// batch.rs `leftovers` for one pair: the side runs of both sides.
pub fn leftovers(before: &str, after: &str, c: &Classification) -> (Side, Side) {
    let moved = |removed: bool| -> Vec<usize> {
        c.moved
            .iter()
            .filter(|m| m.removed == removed)
            .map(|m| m.line)
            .collect()
    };
    (
        side_runs(before, &c.changed.removed, &moved(true)),
        side_runs(after, &c.changed.added, &moved(false)),
    )
}

const MAX_BRIDGE: usize = 7;

fn side_runs(text: &str, changed: &[usize], moved: &[usize]) -> Side {
    let lines: Vec<&str> = text.lines().collect();
    let mut runs: Side = Vec::new();
    let mut open = false;
    let mut prev = 0usize;
    let mut last_kept = 0usize;
    for &l in changed {
        if l != prev + 1 {
            open = false; // an unchanged gap breaks the run
        }
        prev = l;
        let t = lines[l - 1];
        if !significant(t) {
            continue; // blank/punctuation changed line bridges
        }
        if moved.contains(&l) {
            open = false; // a within-moved line breaks the run
            continue;
        }
        if open && l - last_kept - 1 > MAX_BRIDGE {
            open = false; // an over-long bridge is not adjacency
        }
        let entry = (l, fnv1a(t.trim().as_bytes()), alnum_width(t));
        match runs.last_mut() {
            Some(run) if open => run.push(entry),
            _ => runs.push(vec![entry]),
        }
        open = true;
        last_kept = l;
    }
    runs
}
