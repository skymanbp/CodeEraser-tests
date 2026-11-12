//! The replay doc's words (G4): one table per word, written by doc.rs
//! and read back by the report. A word no table holds panics by name, so
//! a misspelt doc is never read as the other value.

use super::model::{Fate, Kind, State};

pub const FATES: [(&str, Fate); 5] = [
    ("re-referred", Fate::ReReferred),
    ("kept up focused", Fate::KeptUp { focused: true }),
    ("kept up sweep", Fate::KeptUp { focused: false }),
    ("removed untouched", Fate::RemovedUntouched),
    ("undetermined", Fate::Undetermined),
];

pub const STATES: [(&str, State); 2] = [("orphaned", State::Orphaned), ("born", State::Born)];

pub const KINDS: [(&str, Kind); 2] = [("code", Kind::Code), ("document", Kind::Document)];

/// The word a table gives a value.
pub fn word<T: PartialEq + Copy>(table: &[(&'static str, T)], x: T) -> &'static str {
    table
        .iter()
        .find(|(_, v)| *v == x)
        .map(|(w, _)| *w)
        .expect("every value has a word")
}

/// The value a table gives a word; `what` names the table in the panic.
pub fn value<T: Copy>(table: &[(&'static str, T)], what: &str, name: &str) -> T {
    table
        .iter()
        .find(|(w, _)| *w == name)
        .map(|(_, v)| *v)
        .unwrap_or_else(|| panic!("unknown {what} {name:?}"))
}

pub fn fate_name(f: Fate) -> &'static str {
    word(&FATES, f)
}

pub fn fate_of(name: &str) -> Fate {
    value(&FATES, "fate", name)
}
