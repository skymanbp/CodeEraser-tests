//! The bin-root facts of one Cargo package as cli/src/graph/mounts.rs
//! held them at 1324c927, the last commit before plan v2.33 W2-text stage
//! F moved the Cargo.toml reading into the core (the mounts table's bit 1
//! asks the core's `private` since). Not a whole-file copy: `RustTargets`
//! and its `impl` are byte copies of that file's items; the differential
//! gate (unit/dedup/ladder_diff/) holds the core's answer against them,
//! reading the frozen Cargo reader (oracle_cfg/cargo.rs) at
//! `crate::graph::cargo`.

use crate::graph::cargo;
use std::collections::BTreeSet;

/// The bin-root facts of one Cargo package, computed ONCE per manifest
/// (the `Declared::gather` discipline — a per-file recomputation
/// rescans the walked set for every Rust file, the shape the criterion
/// itself rules out where nothing bounds the rescanned set, W9-F6).
pub(crate) struct RustTargets {
    /// `[package]` present — a virtual workspace manifest is not a
    /// package and keeps nothing.
    is_package: bool,
    has_lib: bool,
    bins: BTreeSet<String>,
}

impl RustTargets {
    pub(crate) fn of(pkg: Option<cargo::Package>, files: &BTreeSet<String>) -> Self {
        match pkg {
            Some(p) => RustTargets {
                is_package: p.name.is_some(),
                has_lib: p.lib_root(files).is_some(),
                bins: p.bin_roots(files),
            },
            None => RustTargets {
                is_package: false,
                has_lib: false,
                bins: BTreeSet::new(),
            },
        }
    }

    /// The Rust arm, symmetric with the cabal one (§4, L3-F15): a
    /// package without a lib target keeps every file (nothing outside
    /// can `use` it); one with a lib target keeps its bin roots alone
    /// — tests, benches, examples and build.rs are test-side facts,
    /// and a file below a bin root is the mount table's business, not
    /// this bit's.
    pub(crate) fn keeps(&self, path: &str) -> bool {
        self.is_package && (!self.has_lib || self.bins.contains(path))
    }
}
