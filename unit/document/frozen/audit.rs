//! Frozen W7 oracle (plan v2.33 W7; test-only): audit (Stop / precommit / commitmsg): cli/src/audit/speech.rs, copied byte for
//! byte from 1324c927 (spans cli/src/audit/speech.rs 163-171, 252-268) when the core took over spelling the
//! strings. Only this header and the imports above the copy are new. Never
//! edited: the differential (cli/tests/unit/document/spelled/, mounted
//! here so it reads the copied items' private fields) holds the core to it.
#![allow(dead_code)]

use super::binder::{Resolve, at};

#[path = "../spelled/audit.rs"]
mod spelled;

/// The strings the request's references name: each shown block's two
/// paths, each shown site's path, and the three single strings.
struct Strings {
    blocks: Vec<(String, String)>,
    places: Vec<String>,
    error: Option<String>,
    mount: Option<String>,
    message: Option<String>,
}

impl Resolve for Strings {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        let one = |s: &Option<String>| s.clone().filter(|_| ints.is_empty());
        match class {
            "block_a" | "block_b" => {
                let [k] = ints else { return None };
                let (a, b) = self.blocks.get(usize::try_from(*k).ok()?)?;
                Some(if class == "block_a" { a } else { b }.clone())
            }
            "place_file" => at(&self.places, ints),
            "error" => self.error.clone().filter(|_| ints == [0]),
            "mount" => one(&self.mount),
            "message" => one(&self.message),
            _ => None,
        }
    }
}
