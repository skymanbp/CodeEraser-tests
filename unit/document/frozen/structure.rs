//! Frozen W7 oracle (plan v2.33 W7; test-only): structure: cli/src/structure/document.rs, copied byte for
//! byte from 1324c927 (spans cli/src/structure/document.rs 120-125, 127-140) when the core took over spelling the
//! strings. Only this header and the imports above the copy are new. Never
//! edited: the differential (cli/tests/unit/document/spelled/, mounted
//! here so it reads the copied items' private fields) holds the core to it.
#![allow(dead_code)]

use super::binder::{self as document, Resolve};
use crate::structure::seams::SeamFacts;

#[path = "../spelled/structure.rs"]
mod spelled;

/// The structure document's strings: the directory names, and the
/// advisory's file paths and unit names.
struct Names<'a> {
    dirs: Vec<String>,
    seams: Option<&'a SeamFacts>,
}

impl Resolve for Names<'_> {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        let file = |f: &i128| self.seams?.files.get(usize::try_from(*f).ok()?);
        match (class, ints) {
            ("dir", _) => document::at(&self.dirs, ints),
            ("path", [f]) => file(f).map(|f| f.0.clone()),
            ("unit", [f, u]) => {
                let units = self.seams?.unit_names.get(usize::try_from(*f).ok()?)?;
                units.get(usize::try_from(*u).ok()?).map(|u| u.0.clone())
            }
            _ => None,
        }
    }
}
