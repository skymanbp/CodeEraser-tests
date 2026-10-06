//! Frozen W7 oracle (plan v2.33 W7; test-only): erase: cli/src/erase/document.rs, copied byte for
//! byte from 1324c927 (spans cli/src/erase/document.rs 26-28, 95-101, 103-112) when the core took over spelling the
//! strings. Only this header and the imports above the copy are new. Never
//! edited: the differential (cli/tests/unit/document/spelled/, mounted
//! here so it reads the copied items' private fields) holds the core to it.
#![allow(dead_code)]

use super::binder::{self as document, Resolve};

#[path = "../spelled/erase.rs"]
mod spelled;

/// The plan's unified diff, one text per file: the place of the file's
/// first eraseable row in the plan, and its hunks.
pub struct Diffs(Vec<(usize, String)>);

/// The plan's strings: paths and each row's provenance by index, and
/// each file's diff (a `diff` reference names the file's first and last
/// row; the first is the key).
struct PlanStrings<'a> {
    lists: document::Lists,
    diffs: &'a Diffs,
}

impl Resolve for PlanStrings<'_> {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        if class != "diff" {
            return self.lists.resolve(class, ints);
        }
        let first = *ints.first()?;
        let found = self.diffs.0.iter().find(|(i, _)| *i as i128 == first);
        found.map(|(_, d)| d.clone())
    }
}
