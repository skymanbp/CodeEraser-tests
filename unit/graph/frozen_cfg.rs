//! The frozen configuration readers (plan v2.33 W2-text): cmdline.rs,
//! compdb.rs, compdb_flags.rs and gomod.rs in oracle_cfg/ are byte copies
//! of cli/src/graph/ at 92e728b1, the last commit before resolve/1 began
//! carrying the texts and the core began reading them; the trailing
//! unit-test mount of each copy reads `../x.rs`, the only edit.
//! graph/mod.rs mounts this module for tests only and puts the four names
//! back at their old paths, where the frozen ladders (ladder::frozen) read
//! them; the differential gate (unit/graph/resolve/) holds the core's
//! readers against them. pyproject.rs is mounted from roots.rs.
//! cabal.rs and cabal_parse.rs are byte copies of cli/src/graph/ at
//! fa83a48d, before stage D of W2-text moved the cabal reading into the
//! core (cabal.rs's one edit: its unit-test mount reads `../`); the
//! `roots` below gives them back `beside`, the opening only they read,
//! as it stood there. cabal_mains.rs is the main-is expansion of
//! deadcode/targets.rs at fa83a48d. roots_ts.rs and jsonc.rs are byte
//! copies of cli/src/graph/ at dd0eec61, before stage E moved the
//! tsconfig chain and the JSONC reading into the core (jsonc.rs's one
//! edit: its unit-test mount reads `../`); ts_package.rs, the
//! package.json half of roots.rs there, is mounted from roots.rs (its
//! `read_jsonc` reads this jsonc.rs, the frozen chain reads it back
//! through `roots` below). cargo.rs is a byte copy of cli/src/graph/ at
//! 1324c927, before stage F moved the Cargo.toml reading into the core;
//! it reads the frozen TOML walk (oracle_cfg/toml_walk.rs, mounted from
//! roots.rs) through `roots` below. rust_targets.rs holds mounts.rs's
//! `RustTargets` as it stood there, the bin-root facts the mounts
//! table's bit 1 read before it asked the core.
//!
//! Each copy is mounted here by `#[path]` (oracle_cfg/ holds no mod.rs: a
//! parent there would turn the copies' references to their old parent
//! into edges back to it, a cycle — ladder/frozen.rs, same reason). The
//! copies find their old parent's names here: `roots` below, each other.

#[path = "oracle_cfg/cargo.rs"]
pub(crate) mod cargo;
#[path = "oracle_cfg/jsonc.rs"]
pub(crate) mod jsonc;
#[path = "oracle_cfg/roots_ts.rs"]
pub(crate) mod roots_ts;

/// The live path steps, and `beside` as it stood at fa83a48d.
mod roots {
    pub(crate) use crate::graph::roots::*;
    use std::path::Path;

    /// A config file's text with the directory it speaks for: the opening
    /// every reader of a per-directory file shares (a cabal file, a
    /// `compile_flags.txt`) - one throat, or the two openings read as
    /// clones of each other.
    pub(crate) fn beside(root: &Path, rel: &str) -> Option<(String, String)> {
        let text = std::fs::read_to_string(root.join(rel)).ok()?;
        Some((text, parent_dir(rel)))
    }
}

#[path = "oracle_cfg/cabal.rs"]
pub(crate) mod cabal;
#[path = "oracle_cfg/cabal_mains.rs"]
pub(crate) mod cabal_mains;

#[path = "oracle_cfg/cmdline.rs"]
pub(crate) mod cmdline;
#[path = "oracle_cfg/compdb.rs"]
pub(crate) mod compdb;
#[path = "oracle_cfg/compdb_flags.rs"]
pub(crate) mod compdb_flags;
#[path = "oracle_cfg/gomod.rs"]
pub(crate) mod gomod;
#[path = "oracle_cfg/rust_targets.rs"]
pub(crate) mod rust_targets;
