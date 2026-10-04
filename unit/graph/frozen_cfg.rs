//! The frozen configuration readers (plan v2.33 W2-text): cmdline.rs,
//! compdb.rs, compdb_flags.rs and gomod.rs in oracle_cfg/ are byte copies
//! of cli/src/graph/ at 92e728b1, the last commit before resolve/1 began
//! carrying the texts and the core began reading them; the trailing
//! unit-test mount of each copy reads `../x.rs`, the only edit.
//! graph/mod.rs mounts this module for tests only and puts the four names
//! back at their old paths, where the frozen ladders (ladder::frozen) read
//! them; the differential gate (unit/graph/resolve/) holds the core's
//! readers against them. pyproject.rs is mounted from roots.rs.
//!
//! Each copy is mounted here by `#[path]` (oracle_cfg/ holds no mod.rs: a
//! parent there would turn the copies' references to their old parent
//! into edges back to it, a cycle — ladder/frozen.rs, same reason). The
//! glob gives the copies their old parent's names (`roots`, each other).

use super::*;

#[path = "oracle_cfg/cmdline.rs"]
pub(crate) mod cmdline;
#[path = "oracle_cfg/compdb.rs"]
pub(crate) mod compdb;
#[path = "oracle_cfg/compdb_flags.rs"]
pub(crate) mod compdb_flags;
#[path = "oracle_cfg/gomod.rs"]
pub(crate) mod gomod;
