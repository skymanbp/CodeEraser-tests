//! The README demo, replayed (user directive 2026-08-29: a real demo
//! project in the tree, run once without CodeEraser and once with
//! it). `demo/run.js --check` re-runs both trees against THIS build
//! of `ce` and the resolved core and fails if any committed output —
//! the transcripts, the SVGs, the summary JSON, the close-up scenes,
//! or any marked README block — would change: a verdict whose wording
//! moved fails CI rather than leaving a stale picture in the README.
//! The block set is a table in run.js (`EMBEDS`, marker column), so a
//! new family of blocks is gated here without a line changing.

use crate::common::{core_bin, expect_ok, node};

#[test]
fn the_committed_demo_outputs_are_what_this_build_produces() {
    let core = core_bin();
    let env = [
        ("CE_BIN", env!("CARGO_BIN_EXE_ce")),
        ("CE_CORE_BIN", core.as_str()),
        ("CE_UPDATE_CHECK", "0"),
    ];
    expect_ok(&node(&["demo/run.js", "--check"], &env), "demo drifted");
}
