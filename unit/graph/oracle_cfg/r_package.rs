//! The R package-code expansion as it stood in cli/src/graph/deadcode/
//! targets.rs at c96ab3f6, the last commit before resolve/1 began carrying
//! each DESCRIPTION's text and the core began reading it (plan v2.33
//! W2-text stage B). Not a whole-file copy: `package_code` and `is_r` are
//! byte copies of that file's functions, and `packages` is the R branch of
//! `Declared::gather`'s manifest loop, the R manifests given — the part of
//! the declared-target pass the core now answers. The differential gate
//! (unit/dedup/ladder_diff/) holds the core's `packages` reply against it.

use super::r::description::{self, Description};
use crate::graph::roots;
use crate::scan::lang::Lang;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// An R package's code: its `Collate` files under `R/` when it lists
/// them, else every walked R file directly in `R/` — what R loads with
/// the package, which nothing `source`s.
fn package_code(d: &Description, files: &BTreeSet<String>) -> BTreeSet<String> {
    let code = roots::join_dir(&d.dir, "R");
    if d.collate.is_empty() {
        files
            .iter()
            .filter(|f| is_r(f) && roots::parent_dir(f) == code)
            .cloned()
            .collect()
    } else {
        d.collate
            .iter()
            .filter_map(|n| roots::join_rel(&code, n))
            .filter(|p| files.contains(p))
            .collect()
    }
}

fn is_r(path: &str) -> bool {
    Lang::from_path(Path::new(path)) == Some(Lang::R)
}

/// Each R package's code by its root, over the DESCRIPTION paths
/// `gather` found (the manifests ending in `DESCRIPTION`).
pub(crate) fn packages(
    root: &Path,
    manifests: &BTreeSet<String>,
    files: &BTreeSet<String>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut out = BTreeMap::new();
    for m in manifests {
        if m.ends_with("DESCRIPTION")
            && let Some(d) = description::parse(root, m)
        {
            let code = package_code(&d, files);
            out.insert(d.dir, code);
        }
    }
    out
}
