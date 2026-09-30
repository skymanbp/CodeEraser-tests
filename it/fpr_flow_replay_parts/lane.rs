//! One reading of the flow ledger (strict or narrow): the findings it
//! still holds open against each unit, the ones it closed false, and
//! how each settles. Both readings see every event; they differ only
//! in which edit settles a finding that stays. A finding is settled
//! at most once per reading (the coordinator's ruling, 2026-09-30): an
//! edit of its unit (token-level, side.rs) that drops it settles it
//! true in both; one that keeps it settles it false in the strict
//! reading, and in the narrow reading only when the edit's parent-side
//! lines meet the finding's — a non-meeting edit leaves it open there.
//! A finding closed false never re-enters that reading; one settled
//! true or removed re-enters as a new finding if it reappears.

use super::shape::{Key, Kinds, Side, Span, UnitKey};
use std::collections::{BTreeMap, BTreeSet};

type Id = (UnitKey, Key);

/// A finding's fate in one reading — an outcome's cell, in the order
/// of mod.rs's OUTCOME keys: `Found` once per entry, then exactly one
/// of the other four.
#[derive(Debug, Clone, Copy)]
pub enum Fate {
    Found,
    True,
    False,
    Removed,
    Open,
}

/// The two readings, as indices into a kind's pair of outcomes.
pub const STRICT: usize = 0;
pub const NARROW: usize = 1;

#[derive(Default)]
pub struct Lane {
    reading: usize,
    open: BTreeMap<String, BTreeSet<Id>>,
    closed: BTreeMap<String, BTreeSet<Id>>,
}

impl Lane {
    pub fn new(reading: usize) -> Lane {
        Lane {
            reading,
            ..Lane::default()
        }
    }

    fn bump(&self, kinds: &mut Kinds, key: &Key, fate: Fate) {
        kinds[usize::from(key.0)][self.reading][fate as usize] += 1;
    }

    /// A renamed file carries its findings to the new path.
    pub fn rename(&mut self, from: &str, to: &str) {
        for book in [&mut self.open, &mut self.closed] {
            if let Some(ids) = book.remove(from) {
                book.entry(to.to_string()).or_default().extend(ids);
            }
        }
    }

    /// Every finding of a version's judged units entered, once.
    pub fn register(&mut self, path: &str, side: &Side, kinds: &mut Kinds) {
        for (unit, found) in side.judged() {
            for key in found.keys() {
                let id = (unit.clone(), key.clone());
                let closed = self.closed.get(path).is_some_and(|c| c.contains(&id));
                let open = self.open.entry(path.to_string()).or_default();
                if !closed && open.insert(id) {
                    self.bump(kinds, key, Fate::Found);
                }
            }
        }
    }

    /// Every open finding of the path (all its units when `unit` is
    /// None) removed.
    pub fn remove(&mut self, path: &str, unit: Option<&UnitKey>, kinds: &mut Kinds) {
        let Some(open) = self.open.get_mut(path) else {
            return;
        };
        let mut gone = Vec::new();
        open.retain(|(u, key)| {
            let hit = unit.is_none_or(|w| w == u);
            if hit {
                gone.push(key.clone());
            }
            !hit
        });
        for key in gone {
            self.bump(kinds, &key, Fate::Removed);
        }
    }

    /// One token-level edit of a judged unit, against the findings its
    /// parent version carried: the keys this reading closed false.
    pub fn edit(
        &mut self,
        (path, unit): (&str, &UnitKey),
        (was, is): (&BTreeMap<Key, Span>, &BTreeMap<Key, Span>),
        hunks: &[Span],
        kinds: &mut Kinds,
    ) -> Vec<Key> {
        let mut fell = Vec::new();
        for (key, span) in was {
            let id = (unit.clone(), key.clone());
            let open = self.open.entry(path.to_string()).or_default();
            if !open.contains(&id) {
                continue;
            }
            if !is.contains_key(key) {
                open.remove(&id);
                self.bump(kinds, key, Fate::True);
            } else if self.reading == STRICT || meets(*span, hunks) {
                open.remove(&id);
                self.closed.entry(path.to_string()).or_default().insert(id);
                self.bump(kinds, key, Fate::False);
                fell.push(key.clone());
            }
        }
        fell
    }

    /// The window's end: every finding still open.
    pub fn finish(&self, kinds: &mut Kinds) {
        for (_, key) in self.open.values().flatten() {
            self.bump(kinds, key, Fate::Open);
        }
    }
}

/// Whether any hunk meets the span.
pub fn meets(span: Span, hunks: &[Span]) -> bool {
    hunks.iter().any(|&(a, b)| a <= span.1 && span.0 <= b)
}
