//! The frozen W7 oracle (plan v2.33 W7; test-only, mounted from
//! cli/src/document.rs): the binder and every face resolver as they
//! stood at 1324c927, when this side still spelled the document's
//! strings (frozen/), each with its differential beside it (spelled/,
//! mounted inside the frozen module it reads so it reaches the copied
//! items' private fields). The kit (kit.rs) and the seeded stream
//! (rng.rs), beside this directory, are shared.
//! Every leg is an `#[ignore]`d instrument: it names a core
//! (`CE_CORE_BIN`), draws `CE_W7_DIFF_N` references (default 10,000)
//! from `CE_LADDER_DIFF_SEED`, prints its tally and fails on any
//! reference, document or line the core spells otherwise.

mod arch;
mod audit;
mod binder;
mod churn;
mod deadcode;
mod docdup;
mod erase;
mod flow;
mod join;
mod merge;
mod similar;
mod structure;

#[path = "../kit.rs"]
mod kit;
#[path = "../rng.rs"]
mod rng;

// the frozen lines.rs reached the binder as `super::bind`
use binder::bind;
