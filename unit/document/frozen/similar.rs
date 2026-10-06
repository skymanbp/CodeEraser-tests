//! Frozen W7 oracle (plan v2.33 W7; test-only): similar: cli/src/similar/document.rs, copied byte for
//! byte from 1324c927 (spans cli/src/similar/document.rs 71-77, 79-89) when the core took over spelling the
//! strings. Only this header and the imports above the copy are new. Never
//! edited: the differential (cli/tests/unit/document/spelled/, mounted
//! here so it reads the copied items' private fields) holds the core to it.
#![allow(dead_code)]

use super::binder::{self as document, Resolve, Why};

#[path = "../spelled/similar.rs"]
mod spelled;

/// The query's label, each seat's place and key, the reason texts.
struct Strings {
    label: String,
    at: Vec<String>,
    key: Vec<String>,
    why: Why,
}

impl Resolve for Strings {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        match (class, ints) {
            ("label", []) => Some(self.label.clone()),
            ("at", _) => document::at(&self.at, ints),
            ("key", _) => document::at(&self.key, ints),
            ("why", _) => self.why.at(ints),
            _ => None,
        }
    }
}
