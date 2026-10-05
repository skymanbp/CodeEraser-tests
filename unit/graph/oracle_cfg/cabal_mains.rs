//! The cabal main-is expansion as it stood in cli/src/graph/deadcode/
//! targets.rs at fa83a48d, the last commit before resolve/1 began
//! carrying each .cabal file's text and the core began reading it (plan
//! v2.33 W2-text stage D). Not a whole-file copy: `main_targets` is a
//! byte copy of that file's function, and `mains` is the cabal branch of
//! `Declared::gather`'s manifest loop, the cabal manifests given — the
//! part of the declared-target pass the core now answers. The
//! differential gate (unit/dedup/ladder_diff/) holds the core's `mains`
//! reply against it.

use super::cabal;
use crate::graph::roots;
use std::collections::BTreeSet;
use std::path::Path;

/// Declared executable/test mains (2.28.0): each stanza's main-is
/// joined onto each of its source roots, kept only where the file is
/// in the walked set — a main-is naming a missing file declares
/// nothing.
fn main_targets(c: &cabal::Cabal, files: &BTreeSet<String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for s in &c.stanzas {
        let Some(main) = &s.main_is else { continue };
        for r in &s.roots {
            if let Some(cand) = roots::join_rel(r, main)
                && files.contains(&cand)
            {
                out.insert(cand);
            }
        }
    }
    out
}

/// The declared mains over the cabal paths `gather` found.
pub(crate) fn mains(
    root: &Path,
    manifests: &BTreeSet<String>,
    files: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for m in manifests {
        if let Some(c) = cabal::parse(root, m) {
            out.extend(main_targets(&c, files));
        }
    }
    out
}
