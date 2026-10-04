//! The differential gate of plan v2.33 wave W2a (user instruction
//! 2026-10-03): every site a tree holds for the four ladders the core
//! now runs (Python, Lua, Go, C / C++) is resolved twice — by the core
//! over the real resolve/1 wire (`ladder::resolve_all`) and by the
//! frozen a8db74a9 rungs (unit/graph/ladder/oracle/, mounted as
//! `ladder::frozen` in cli/src/graph/ladder/mod.rs) — and the two
//! outcomes must be equal, site by site; the C forced-include arcs
//! likewise. Legs are `#[ignore]`d instruments (a core must be named):
//! `real` walks the trees `CE_LADDER_DIFF_TREES` names, the four random
//! legs build seeded trees (`CE_LADDER_DIFF_SEED`, `CE_LADDER_DIFF_N`
//! trees of 30 sites each). Each leg prints its tally verbatim and
//! fails on any mismatch, printing the first ones whole. What the legs
//! share lives in leaves (common.rs, rng.rs, the word tables in
//! tables.rs): this file only declares them.

mod c;
mod common;
mod go;
mod lua;
mod py;
mod real;
mod rng;
mod tables;
