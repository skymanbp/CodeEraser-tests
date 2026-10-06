//! Frozen W7 oracle (plan v2.33 W7; test-only): deadcode: cli/src/graph/deadcode/document.rs, copied byte for
//! byte from 1324c927 (spans cli/src/graph/deadcode/document.rs 166-172, 174-194) when the core took over spelling the
//! strings. Only this header and the imports above the copy are new. Never
//! edited: the differential (cli/tests/unit/document/spelled/, mounted
//! here so it reads the copied items' private fields) holds the core to it.
#![allow(dead_code)]

use super::binder::{self as document, Resolve, Why};
use crate::graph::nodes::Node;

#[path = "../spelled/deadcode.rs"]
mod spelled;

/// The deadcode document's strings: the node paths, a section's
/// `path#unit` label, the advisory's names.
pub(crate) struct Names<'a> {
    nodes: &'a [Node],
    symbols: Vec<String>,
    why: Why,
}

impl Resolve for Names<'_> {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        let node = || {
            let [i] = ints else { return None };
            usize::try_from(*i).ok().and_then(|i| self.nodes.get(i))
        };
        match class {
            "path" => node().map(|n| n.path.clone()),
            "node_name" => node().map(|n| {
                if n.unit.is_empty() {
                    n.path.clone()
                } else {
                    format!("{}#{}", n.path, n.unit)
                }
            }),
            "symbol" => document::at(&self.symbols, ints),
            "why" => self.why.at(ints),
            _ => None,
        }
    }
}
