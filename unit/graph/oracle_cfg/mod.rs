//! The frozen configuration readers (plan v2.33 W2-text): cmdline.rs,
//! compdb.rs, compdb_flags.rs and gomod.rs are byte copies of
//! cli/src/graph/ at 92e728b1, the last commit before resolve/1 began
//! carrying the texts and the core began reading them; the trailing
//! unit-test mount of each copy reads `../x.rs`, the only edit.
//! graph/mod.rs mounts this module for tests only and puts the four names
//! back at their old paths, where the frozen ladders (ladder::frozen) read
//! them; the differential gate (unit/graph/resolve/) holds the core's
//! readers against them. pyproject.rs is mounted from roots.rs.
//!
//! The glob gives the copies their old parent's names (`roots`, each
//! other), as ladder/frozen.rs does for the frozen rungs.

use super::*;

pub(crate) mod cmdline;
pub(crate) mod compdb;
pub(crate) mod compdb_flags;
pub(crate) mod gomod;
