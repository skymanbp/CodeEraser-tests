//! Frozen W7 oracle (plan v2.33 W7; test-only): join: cli/src/join/document.rs, copied byte for
//! byte from 1324c927 (spans cli/src/join/document.rs 208-213, 215-230) when the core took over spelling the
//! strings. Only this header and the imports above the copy are new. Never
//! edited: the differential (cli/tests/unit/document/spelled/, mounted
//! here so it reads the copied items' private fields) holds the core to it.
#![allow(dead_code)]

use super::binder::{self as document, Resolve, Why};

#[path = "../spelled/join.rs"]
mod spelled;

/// The join document's strings: the paths and each unit row's keys.
struct Names {
    paths: Vec<String>,
    keys: Vec<[String; 2]>,
    why: Why,
}

impl Resolve for Names {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        match class {
            "path" => document::at(&self.paths, ints),
            "key" => match ints {
                [k, side] => {
                    let row = self.keys.get(usize::try_from(*k).ok()?)?;
                    row.get(usize::try_from(*side).ok()?).cloned()
                }
                _ => None,
            },
            "why" => self.why.at(ints),
            _ => None,
        }
    }
}
