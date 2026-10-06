//! The differential gate of plan v2.33 wave W2a (user instruction
//! 2026-10-03): every site a tree holds for the ladders the core now
//! runs (Python, Lua, Go, C / C++; R since W2-text stage B, Java since
//! stage C, Haskell since stage D, TypeScript / TSX since stage E, Rust
//! since stage F) is
//! resolved
//! twice — by the core over the real resolve/1 wire
//! (`ladder::resolve_all`) and by the frozen rungs
//! (unit/graph/ladder/oracle/, mounted as `ladder::frozen` in
//! cli/src/graph/ladder/mod.rs) — and the two outcomes must be equal,
//! site by site; the C forced-include arcs, the R package code, the
//! cabal mains, the Cargo crate roots and the cabal and Cargo privacy
//! likewise. Legs are `#[ignore]`d
//! instruments (a core must be named):
//! `real` walks the trees `CE_LADDER_DIFF_TREES` names, the random legs
//! build seeded trees (`CE_LADDER_DIFF_SEED`, `CE_LADDER_DIFF_N` trees of
//! 30 sites each). Each leg prints its tally verbatim and fails on any
//! mismatch, printing the first ones whole. What the legs share lives in
//! leaves (common.rs, beside.rs, rng.rs, the word tables in tables.rs):
//! this file only declares them.

mod beside;
mod c;
mod common;
mod go;
mod hs;
mod java;
mod java_gen;
mod lua;
mod py;
mod r;
mod real;
mod rng;
mod rs;
mod rs_gen;
mod tables;
mod ts;
mod ts_gen;
