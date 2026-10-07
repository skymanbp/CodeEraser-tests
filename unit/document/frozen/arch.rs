//! Frozen W7 oracle (plan v2.33 W7; test-only): arch: cli/src/arch/face.rs, copied byte for
//! byte from 1324c927 (spans cli/src/arch/face.rs 109-113, 115-120, 122-132, 134-142) when the core took over spelling the
//! strings. Only this header and the imports above the copy are new. Never
//! edited: the differential (cli/tests/unit/document/spelled/, mounted
//! here so it reads the copied items' private fields) holds the core to it.
#![allow(dead_code)]

use super::binder::{self as document, Resolve, Why};
use crate::structure::oracle::tables::Tables;

#[path = "../spelled/arch.rs"]
mod spelled;

/// `[dir, bytes, chars]` for every directory (the root is 0 bytes).
fn widths(dirs: &[String]) -> Vec<[usize; 3]> {
    let width = |(i, d): (usize, &String)| [i, d.len(), d.chars().count()];
    dirs.iter().enumerate().map(width).collect()
}

/// The arch document's strings: the paths, the directories, and the
/// reason the judgment did not happen.
struct Names<'a> {
    t: &'a Tables,
    why: Why,
}

impl Resolve for Names<'_> {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        match class {
            "path" => document::at(&self.t.paths, ints),
            "dir" => document::at(&self.t.dir_paths, ints),
            "slashed" => document::at(&self.t.dir_paths, ints).map(|d| slashed(&d)),
            "why" => self.why.at(ints),
            _ => None,
        }
    }
}

/// A directory as an arc end: its path with a trailing slash, the
/// root as `./`.
pub fn slashed(dir: &str) -> String {
    if dir.is_empty() {
        "./".into()
    } else {
        format!("{dir}/")
    }
}
