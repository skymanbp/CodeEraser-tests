//! The README demo, replayed (user directive 2026-08-29: a real demo
//! project in the tree, run once without CodeEraser and once with
//! it). `demo/run.js --check` re-runs both trees against THIS build
//! of `ce` and the resolved core and fails if any committed output —
//! the transcripts, the SVGs, the summary JSON, or the three embedded
//! tables — would change: a verdict whose wording moved fails CI
//! rather than leaving a stale picture in the README.

use crate::common::{core_bin, repo_root};

#[test]
fn the_committed_demo_outputs_are_what_this_build_produces() {
    let out = std::process::Command::new("node")
        .arg("demo/run.js")
        .arg("--check")
        .env("CE_BIN", env!("CARGO_BIN_EXE_ce"))
        .env("CE_CORE_BIN", core_bin())
        .env("CE_UPDATE_CHECK", "0")
        .current_dir(repo_root())
        .output()
        .expect("node is on PATH (the demo driver needs no packages)");
    assert!(
        out.status.success(),
        "demo drifted:\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
