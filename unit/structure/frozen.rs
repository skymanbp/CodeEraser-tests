//! The frozen structure and arch tables (plan v2.33 W1 item 3): tree.rs,
//! edges.rs and rows.rs in oracle/ are byte copies of cli/src/structure/
//! at c2abca5d, the last commit before structure/1, arch/1 and query/1
//! began carrying the paths and the core began building the directory
//! tree and the tables over it (CE.Structure.Tree, CE.Structure.Raw,
//! CE.Arch.Tables, CE.Query.Tree); ../arch/oracle/tables.rs is the byte
//! copy of cli/src/arch/tables.rs there. The edits: the trailing unit-test
//! mount of tree.rs, edges.rs and tables.rs reads `../x.rs`, so the unit
//! tests of the tree, the edge counts and the arch tables keep testing the
//! code they were written for; tables.rs reads its two old siblings
//! (`structure::tree`, `structure::rows`) here. structure/mod.rs mounts
//! this module for tests only; the differential legs hold the core's
//! built tables against these copies.
//!
//! The copies keep every item they had, the ones no leg calls too
//! (`dead_code` allowed on each mount — a byte copy is not trimmed).
//!
//! Each copy is mounted here by `#[path]` (oracle/ holds no mod.rs: a
//! parent there would turn the copies' `super::` references into edges
//! back to it, a cycle — graph/frozen_cfg.rs, same reason). The copies
//! find each other as siblings under this module, as they did under
//! structure/.

#[allow(dead_code)]
#[path = "oracle/edges.rs"]
pub mod edges;
#[allow(dead_code)]
#[path = "oracle/rows.rs"]
pub mod rows;
#[allow(dead_code)]
#[path = "../arch/oracle/tables.rs"]
pub mod tables;
#[allow(dead_code)]
#[path = "oracle/tree.rs"]
pub mod tree;

// The differential legs that hold the core to the copies, and their cases.
#[path = "diff_arch.rs"]
pub mod diff_arch;
#[path = "diff_gen.rs"]
pub mod diff_gen;
#[path = "diff_hold.rs"]
pub mod diff_hold;
#[path = "diff_real.rs"]
mod diff_real;
#[path = "diff_rows.rs"]
pub mod diff_rows;
#[path = "diff_stale.rs"]
mod diff_stale;
