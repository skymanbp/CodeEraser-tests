//! Shared integration-test helpers. Extracted after the FPR
//! self-replay arbitration flagged 33 copies of these very functions
//! across `cli/tests/*.rs` — the tool catching its author's own
//! stacking (docs/FPR-REPLAY.md).
//!
//! Each test binary compiles its own copy of this module and uses a
//! subset of it, so unused items here are expected — that is the why
//! for the allow below.
#![allow(dead_code)]
// The hooks re-export is likewise unused in binaries that never run
// hooks — same subset story as dead_code above.
#![allow(unused_imports)]

pub mod audit;
pub mod daemon;
pub mod fixtures;
pub mod gates;
pub mod gitio;
mod history;
pub mod hooks;
pub mod ladder;
pub mod mcp;
pub mod metric;
pub mod stats;
// One brace, not five `pub use` lines: the per-line form made this
// index a byte-shaped twin of eval_support/mod.rs the moment a fifth
// entry joined, and a module index is not something to clone.
pub use {
    audit::*, daemon::*, fixtures::*, gates::*, gitio::*, history::*, hooks::*, ladder::*, mcp::*,
    metric::*,
};

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// Run the library dedup pipeline on `dir`, assert the pairwise-block
/// and group counts every caller checks anyway, and return the result
/// for the distinctive per-test assertions. Counts live HERE so the
/// per-test stanzas stay below the winnowing guarantee t — the
/// ratchet caught three copies of that pattern (dedup_groups.rs).
pub fn analyze(dir: &Path, blocks: usize, groups: usize) -> codeeraser::dedup::pairs::Blocks {
    let (found, _) = codeeraser::dedup::analyze(dir, None, None, None).expect("analyze");
    assert_eq!(found.blocks.len(), blocks, "pairwise block count");
    assert_eq!(found.groups.len(), groups, "group count");
    found
}

/// The repository root — cli/'s parent. Every gate that reads a
/// tracked file starts here. It lived as seven private copies across
/// the test binaries until the clone gate pointed at one pair; the
/// class was all seven, so the definition moved here once.
pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent")
        .to_path_buf()
}

/// Every file under `dir`, recursively, whose extension is `ext` —
/// the one walk the fixtures' `_why` gate and the unit-mount gate
/// share (they were a clone row apart until the dedup gate said so).
pub fn files_with_ext(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    for p in std::fs::read_dir(dir)
        .expect("a directory to walk")
        .flatten()
        .map(|e| e.path())
    {
        if p.is_dir() {
            files_with_ext(&p, ext, out);
        } else if p.extension().is_some_and(|x| x == ext) {
            out.push(p);
        }
    }
}

/// Path of a golden fixture under contracts/fixtures.
pub fn golden_path(name: &str) -> PathBuf {
    repo_root().join("contracts/fixtures").join(name)
}

/// CE_BLESS=1 regenerates the golden; otherwise byte-compare
/// (CRLF-normalized). The switch is read by `facts::blessing()` alone.
pub fn assert_matches_golden(json: &str, path: &Path) {
    if crate::facts::blessing() {
        std::fs::create_dir_all(path.parent().expect("golden dir")).expect("mkdir");
        std::fs::write(path, format!("{json}\n")).expect("bless golden");
        return;
    }
    let golden = std::fs::read_to_string(path)
        .unwrap_or_else(|e| {
            panic!(
                "missing golden {} ({e}); CE_BLESS=1 to create",
                path.display()
            )
        })
        .replace("\r\n", "\n");
    assert_eq!(
        json.trim_end(),
        golden.trim_end(),
        "report shape drifted — bump the schema id and re-bless deliberately"
    );
}

/// Corrupt the project's dedup index so every deep check degrades —
/// the A9f test fixture (audit, precommit, and guard variants).
pub fn corrupt_index(dir: &Path) {
    std::fs::create_dir_all(dir.join(".ce")).expect(".ce");
    std::fs::write(dir.join(".ce/index.db"), b"not a sqlite database").expect("corrupt db");
}

/// Run the real `ce` binary with `args` in `dir`; the caller asserts
/// on success or failure (gate tests need both directions).
/// run_expect / write_all (the success-direction and multi-write
/// stanzas) live in gates.rs — mod.rs sits at its own 300-line gate.
pub fn run_ce(dir: &Path, args: &[&str]) -> std::process::Output {
    run_ce_env(dir, args, &[])
}

/// `run_ce` under named environment — the acts `ce baseline` reads
/// and the console language; the one process-construction throat.
/// The two acts and CE_LANG are cleared first: a developer shell that
/// exported one would otherwise turn every refusal case green, or
/// every English assertion red, by inheritance (the hooks helper and
/// cli_bare scrub the same variable; a row testing Chinese passes it
/// in `env`, which lands after). The SessionStart update notice is
/// off unless a leg arms it: no battery reaches the release index by
/// accident (update/notice.rs).
pub fn run_ce_env(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ce"))
        .args(args)
        .env_remove("CE_ACCEPT_BASELINE")
        .env_remove("CE_ACCEPT_FENCE")
        .env_remove("CE_LANG")
        .env("CE_UPDATE_CHECK", "0")
        .envs(env.iter().copied())
        .current_dir(dir)
        .output()
        .expect("run ce")
}

/// Build the project index by running the real `ce dedup .` in `dir`.
pub fn build_index(dir: &Path) {
    let out = run_ce(dir, &["dedup", "."]);
    assert!(out.status.success(), "seed dedup failed");
}

/// Open the built index and assemble the graph wire from it, on the
/// road asked — the prelude of every leg that reads the wire off a
/// real tree (export surface, mounts, the advisory report); its third
/// copy was the clone gate's.
pub fn graph_wire(
    dir: &Path,
    advisory: codeeraser::graph::deadcode::Advisory,
) -> (
    codeeraser::dedup::index::Index,
    codeeraser::graph::deadcode::GraphWire,
) {
    let db = dir.join(".ce/index.db");
    let idx = codeeraser::dedup::index::Index::open(&db, codeeraser::dedup::Params::default())
        .expect("open index");
    let w = codeeraser::graph::deadcode::wire_of(dir, &idx, &db, advisory).expect("graph wire");
    (idx, w)
}

/// The two named baseline acts as environments (plan v2.18 step #14).
pub const WHOLESALE: &[(&str, &str)] = &[("CE_ACCEPT_BASELINE", "1")];
pub const FENCE: &[(&str, &str)] = &[("CE_ACCEPT_FENCE", "1")];

/// Declare `dir`'s ce.toml whole.
pub fn declare(dir: &Path, toml: &str) {
    std::fs::write(dir.join("ce.toml"), toml).expect("ce.toml");
}

/// One `ce` run as (exit code, stdout, stderr) — the shape every
/// act-and-refusal leg reads (baseline_policy, fence_wire).
pub fn ce_triple(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> (Option<i32>, String, String) {
    let out = run_ce_env(dir, args, env);
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

// Daemon e2e machinery (spawn/raw-line/shutdown/wait) lives in
// daemon.rs — split at the 300-line dogfood wall.
