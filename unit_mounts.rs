//! The root binary of the three-set gate (it/unit_mounts.rs, mounted
//! below; the legs keep their `unit_mounts::` names).
//!
//! Alone in its binary: `cargo package --list` asks git for the status
//! of the whole package root (cli/), cli/target included, while the
//! legs of the `it` crate create and remove scratch directories under
//! cli/target/tmp in parallel threads. A directory that goes away
//! between the walker listing it and reading it fails the listing:
//! run 43909586343 (Windows, 2026-11-16) stopped on "Failed to read the
//! directory at '...\cli\target\tmp\update-apply\assets'". Cargo runs
//! test binaries one after another, so in a root binary nothing else
//! of the suite runs while the listing walks the checkout.

// This binary compiles its own copy of common/ (the one crate's
// helper tree) and calls a slice of it: the rest is dead here only.
#[allow(dead_code, unused_imports)]
#[path = "it/common/mod.rs"]
mod common;
// common's golden comparator reads the bless switch through the one
// reader (plan v2.21 gate 1); this binary never blesses, hence the
// allow on the unused function.
#[allow(dead_code)]
#[path = "it/facts/mod.rs"]
mod facts;
#[path = "it/unit_mounts.rs"]
mod unit_mounts;
