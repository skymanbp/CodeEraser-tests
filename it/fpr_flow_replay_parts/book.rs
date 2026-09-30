//! The flow ledger's book (design booklet analysis-track.md §5.5
//! 「回放台账」): the unit versions each event brings and the two
//! readings (lane.rs) they are told to. Pure over judged file versions
//! — the core is asked in side.rs and git is read in walk.rs, so the
//! counting rule has one owner and the synthetic history in
//! fpr_flow_gate.rs walks the same code the corpora do.
//!
//! A finding enters when a judged version of its unit carries it (the
//! parent's version when the file is an event, the child's after). An
//! event EDITS a unit when both versions are judged and the unit's
//! token sequence differs (side.rs: a reformat is no edit); each
//! reading settles the unit's open findings as lane.rs says. A unit
//! gone from the child (the file deleted, the unit deleted or renamed)
//! takes its open findings out as `removed`; what is open at the
//! window's end is `undetermined`. Per reading and per kind,
//! `findings = tp + false + removed + undetermined` closes by
//! construction.

use super::lane::{Lane, NARROW, STRICT};
use super::shape::{Judgment, Key, Kinds, Side, Span, UnitKey};

/// The ledger's counts: the events and unit versions, and each kind's
/// two outcomes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Tally {
    pub events: usize,
    pub units_judged: usize,
    pub dynamic_skipped: usize,
    pub unjudged: usize,
    pub kinds: Kinds,
    /// Token-identical units whose finding keys differ between the two
    /// versions (anchor text drift): printed, never frozen.
    pub drift: usize,
}

/// One event: the file's parent version (path, judged side) if it had
/// one, the child's if it has one, and the parent-side lines the edit
/// touched.
pub struct Event<'a> {
    pub old: Option<(&'a str, &'a Side)>,
    pub new: Option<(&'a str, &'a Side)>,
    pub hunks: &'a [Span],
}

pub struct Book {
    lanes: [Lane; 2],
    pub tally: Tally,
    /// The narrow falses of the last event, `(path, unit, finding)`,
    /// for the walk to print and clear.
    pub fell: Vec<(String, UnitKey, Key)>,
}

impl Default for Book {
    fn default() -> Book {
        Book {
            lanes: [Lane::new(STRICT), Lane::new(NARROW)],
            tally: Tally::default(),
            fell: Vec::new(),
        }
    }
}

impl Book {
    pub fn event(&mut self, e: &Event) {
        self.tally.events += 1;
        if let (Some((from, _)), Some((to, _))) = (e.old, e.new) {
            self.lanes.iter_mut().for_each(|l| l.rename(from, to));
        }
        let Some(path) = e.new.or(e.old).map(|(p, _)| p) else {
            return;
        };
        for (_, side) in e.old.iter().chain(e.new.iter()) {
            self.count(side);
        }
        let kinds = &mut self.tally.kinds;
        if let Some((_, parent)) = e.old {
            self.lanes
                .iter_mut()
                .for_each(|l| l.register(path, parent, kinds));
            match e.new {
                Some((_, child)) => self.resolve(path, parent, child, e.hunks),
                None => self
                    .lanes
                    .iter_mut()
                    .for_each(|l| l.remove(path, None, kinds)),
            }
        }
        if let Some((_, child)) = e.new {
            let kinds = &mut self.tally.kinds;
            self.lanes
                .iter_mut()
                .for_each(|l| l.register(path, child, kinds));
        }
    }

    /// The window's end: every finding still open, in each reading.
    pub fn finish(mut self) -> Tally {
        let kinds = &mut self.tally.kinds;
        self.lanes.iter().for_each(|l| l.finish(kinds));
        self.tally
    }

    fn count(&mut self, side: &Side) {
        for view in side.units.values() {
            match view.judgment {
                Judgment::Judged(_) => self.tally.units_judged += 1,
                Judgment::Dynamic => self.tally.dynamic_skipped += 1,
                Judgment::Unjudged => self.tally.unjudged += 1,
            }
        }
        self.tally.unjudged += side.unkeyed.len();
    }

    fn resolve(&mut self, path: &str, parent: &Side, child: &Side, hunks: &[Span]) {
        let kinds = &mut self.tally.kinds;
        for (unit, before) in &parent.units {
            let Some(after) = child.units.get(unit) else {
                if !child.unkeyed.contains(&unit.0) {
                    self.lanes
                        .iter_mut()
                        .for_each(|l| l.remove(path, Some(unit), kinds));
                }
                continue;
            };
            let (Judgment::Judged(was), Judgment::Judged(is)) = (&before.judgment, &after.judgment)
            else {
                continue;
            };
            if before.tokens == after.tokens {
                self.tally.drift += usize::from(was.keys().ne(is.keys()));
                continue;
            }
            let [strict, narrow] = &mut self.lanes;
            strict.edit((path, unit), (was, is), hunks, kinds);
            for key in narrow.edit((path, unit), (was, is), hunks, kinds) {
                self.fell.push((path.to_string(), unit.clone(), key));
            }
        }
    }
}
