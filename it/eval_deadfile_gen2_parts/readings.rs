//! BARS-GEN2 sections 3 and 4 as code: the two readings of an event's
//! fate, the tallies per state, kind and corpus, the Clopper-Pearson 95 %
//! upper bound, the pooled share with each corpus left out, and the bars.
//! Shares compare as integers (`100 * false <= events` is "at most
//! 1.0 %"), so no float decides a bar.

use super::model::{Event, Fate, Kind, State};
use crate::common::stats::cp_upper_ppm;
use std::collections::BTreeMap;

/// Section 4's judgment constants.
pub const MIN_EVENTS: usize = 100;
pub const MIN_CORPUS_EVENTS: usize = 30;
/// The pooled bar, 1.0 %, and the single-corpus bar, 5.0 %: a share
/// `k / n` passes when `k * DIV <= n`.
pub const POOLED_DIV: usize = 100;
pub const CORPUS_DIV: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    A,
    B,
}

/// One event's standing under a reading: `Some(true)` counts it false,
/// `Some(false)` right, `None` leaves it out of the denominator.
pub fn counts(r: Reading, f: Fate) -> Option<bool> {
    match (r, f) {
        (_, Fate::ReReferred) => Some(true),
        (Reading::A, Fate::KeptUp { .. }) | (Reading::A, Fate::Undetermined) => Some(true),
        (Reading::B, Fate::KeptUp { focused }) => focused.then_some(true),
        (Reading::B, Fate::Undetermined) => None,
        (_, Fate::RemovedUntouched) => Some(false),
    }
}

/// `(false, denominator)` of a set of fates under a reading.
pub fn share(r: Reading, fates: &[Fate]) -> (usize, usize) {
    fates
        .iter()
        .filter_map(|f| counts(r, *f))
        .fold((0, 0), |(k, n), bad| (k + usize::from(bad), n + 1))
}

/// Whether `(k, n)` is at most `1 / div` (an empty denominator is zero).
pub fn within(k: usize, n: usize, div: usize) -> bool {
    k * div <= n
}

/// One corpus's events of one state: the fates, by kind.
pub type ByKind = BTreeMap<Kind, Vec<Fate>>;
/// Per corpus (name), per state: the fates by kind.
pub type Table = BTreeMap<String, BTreeMap<State, ByKind>>;

/// The table of one corpus's replay.
pub fn tally(corpus: &str, events: &[(Event, Fate)], table: &mut Table) {
    let row = table.entry(corpus.to_string()).or_default();
    for (e, f) in events {
        row.entry(e.state)
            .or_default()
            .entry(e.kind)
            .or_default()
            .push(*f);
    }
}

/// A state's fates on one corpus, of one kind or of every kind.
pub fn fates(table: &Table, corpus: &str, state: State, kind: Option<Kind>) -> Vec<Fate> {
    let Some(by_kind) = table.get(corpus).and_then(|r| r.get(&state)) else {
        return Vec::new();
    };
    by_kind
        .iter()
        .filter(|(k, _)| kind.is_none_or(|want| **k == want))
        .flat_map(|(_, f)| f.iter().copied())
        .collect()
}

/// A state's fates pooled over the corpora `keep` accepts.
pub fn pooled(
    table: &Table,
    state: State,
    kind: Option<Kind>,
    keep: &dyn Fn(&str) -> bool,
) -> Vec<Fate> {
    table
        .keys()
        .filter(|c| keep(c))
        .flat_map(|c| fates(table, c, state, kind))
        .collect()
}

/// Conditions 2 and 3 for one kind (or every kind): the pooled share at
/// most 1.0 %, and no corpus with at least MIN_CORPUS_EVENTS events of
/// it above 5.0 %.
fn holds(table: &Table, state: State, kind: Option<Kind>, r: Reading) -> bool {
    let (k, n) = share(r, &pooled(table, state, kind, &|_| true));
    within(k, n, POOLED_DIV)
        && table.keys().all(|c| {
            let f = fates(table, c, state, kind);
            let (k, n) = share(r, &f);
            f.len() < MIN_CORPUS_EVENTS || within(k, n, CORPUS_DIV)
        })
}

/// Section 4: `None` when the state has fewer than MIN_EVENTS events
/// (not judged), else whether it BLOCKS under the reading — conditions
/// 2 and 3, and both again for each kind with MIN_EVENTS events of its
/// own.
pub fn blocks(table: &Table, state: State, r: Reading) -> Option<bool> {
    if pooled(table, state, None, &|_| true).len() < MIN_EVENTS {
        return None;
    }
    let kinds = [Kind::Code, Kind::Document]
        .into_iter()
        .filter(|k| pooled(table, state, Some(*k), &|_| true).len() >= MIN_EVENTS);
    Some(holds(table, state, None, r) && kinds.into_iter().all(|k| holds(table, state, Some(k), r)))
}

/// The outcome of a state, one of four section 4 names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    FailsByDefault,
    Listed,
    OwnerQuestion,
    NotJudged,
}

pub fn outcome(table: &Table, state: State) -> Outcome {
    match (
        blocks(table, state, Reading::A),
        blocks(table, state, Reading::B),
    ) {
        (None, _) | (_, None) => Outcome::NotJudged,
        (Some(true), Some(true)) => Outcome::FailsByDefault,
        (Some(false), Some(false)) => Outcome::Listed,
        _ => Outcome::OwnerQuestion,
    }
}

/// The sentence the page prints for an outcome — one per outcome
/// section 4 can produce, so none is said for another (N14).
pub fn sentence(state: &str, o: Outcome) -> String {
    match o {
        Outcome::FailsByDefault => {
            format!("{state} blocks under readings A and B: it fails the gate by default.")
        }
        Outcome::Listed => format!(
            "{state} misses the bars under readings A and B: it is listed, never fails the gate by default, and carries its measured share."
        ),
        Outcome::OwnerQuestion => format!(
            "{state} blocks under one reading and not the other: the owner is asked, with the two shares and reading C."
        ),
        Outcome::NotJudged => {
            format!("{state} has fewer than {MIN_EVENTS} events: not judged, it does not block.")
        }
    }
}

/// The Clopper-Pearson 95 % upper bound of a share, in ppm.
pub fn upper_ppm(k: usize, n: usize) -> u64 {
    cp_upper_ppm(k, n)
}

/// The pooled share with each corpus left out in turn: `(corpus, k, n)`.
pub fn leave_one_out(table: &Table, state: State, r: Reading) -> Vec<(String, usize, usize)> {
    table
        .keys()
        .map(|out| {
            let (k, n) = share(r, &pooled(table, state, None, &|c| c != out));
            (out.clone(), k, n)
        })
        .collect()
}
