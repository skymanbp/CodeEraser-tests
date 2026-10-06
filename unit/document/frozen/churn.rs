//! Frozen W7 oracle (plan v2.33 W7; test-only): churn: cli/src/churn/report.rs, copied byte for
//! byte from 1324c927 (spans cli/src/churn/report.rs 82-86, 88-96) when the core took over spelling the
//! strings. Only this header and the imports above the copy are new. Never
//! edited: the differential (cli/tests/unit/document/spelled/, mounted
//! here so it reads the copied items' private fields) holds the core to it.
#![allow(dead_code)]

use super::binder::{self as document, Resolve};

#[path = "../spelled/churn.rs"]
mod spelled;

/// The churn document's strings: the pairs' paths and the submodules.
struct Names<'a> {
    paths: Vec<String>,
    submodules: &'a [String],
}

impl Resolve for Names<'_> {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        match class {
            "path" => document::at(&self.paths, ints),
            "submodule" => document::at(self.submodules, ints),
            _ => None,
        }
    }
}
