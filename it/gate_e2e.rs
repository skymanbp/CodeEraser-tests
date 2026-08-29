//! Gate e2e batteries through the real binaries — ONE home for the
//! --check/--core gates (review C16 + the eleventh ratchet bite:
//! dedup_check.rs and scan_check.rs re-grew the harness skeleton as
//! sibling files the moment the second gate landed, so the gates
//! live together and walk common::gate_red_green).
//! dedup budget: M2 review R12, comparison in the core since P2;
//! scan grading: ADR-008 P3, levels and exit semantics in the core.

use std::path::Path;
use std::process::Output;

use crate::common;
use crate::common::{core_bin, gate_red_green, rust_fn, tmp};

fn run_dedup_check(dir: &Path) -> Output {
    common::run_ce(dir, &["dedup", ".", "--check", "--core", &core_bin()])
}

fn run_scan(dir: &Path) -> Output {
    common::run_ce(dir, &["scan", ".", "--core", &core_bin()])
}

#[test]
fn dedup_check_fails_over_budget_and_passes_at_budget() {
    let dir = tmp("gate-dedup");
    common::seed_clone_pair(&dir);
    common::seed_budget(&dir, 0);
    gate_red_green(&dir, &run_dedup_check, "ratchet", true, &|| {
        common::seed_budget(&dir, 1);
    });
}

/// A tree with no clone block at all passes `--check` on budget 0:
/// the core answers `dedupBlocks: null` when no distinct row rode,
/// and null is zero admitted, not a drift (step #12 found the first
/// such tree: a superproject whose only clones sit in a submodule).
#[test]
fn dedup_check_passes_on_a_tree_without_clones() {
    let dir = tmp("gate-dedup-clean");
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    common::seed_budget(&dir, 0);
    let out = run_dedup_check(&dir);
    assert!(out.status.success(), "no blocks, budget 0: {out:?}");
}

#[test]
fn dedup_check_without_budget_is_an_error() {
    let dir = tmp("gate-dedup-nocfg");
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    let out = run_dedup_check(&dir);
    assert!(!out.status.success(), "--check without budget must error");
}

/// `--check` accepts the calibrated operating point or a tighter one
/// only — a looser filter empties the budget with no clone repaid
/// (k4 fence attack, item O41). The refusal precedes any core
/// contact (a nonexistent core proves it, and the message, not the
/// exit code, is what the old binary could not produce); tightening
/// rows reach the core and stay red on the same seeded pair.
#[test]
fn dedup_check_refuses_loosening_overrides_before_the_core() {
    let dir = tmp("gate-dedup-filters");
    common::seed_clone_pair(&dir);
    common::seed_budget(&dir, 0);
    let loosening = [
        ("--min-tokens", "1000"),
        ("--min-distinct", "8"),
        ("--min-distinct", "0"),
    ];
    for (flag, value) in loosening {
        let args = [
            "dedup",
            ".",
            "--check",
            flag,
            value,
            "--core",
            "ce-core-that-does-not-exist",
        ];
        let out = common::run_ce(&dir, &args);
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(2),
            "{flag} {value}: refused as usage: {out:?}"
        );
        assert!(
            err.contains("default or tighter only") && err.contains(flag),
            "{flag} {value}: the refusal names the flag and the rule: {err}"
        );
    }
    for (flag, value) in [("--min-tokens", "25"), ("--min-distinct", "1")] {
        let out = common::run_ce(
            &dir,
            &["dedup", ".", "--check", flag, value, "--core", &core_bin()],
        );
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(1),
            "{flag} {value}: reaches the core, stays red: {out:?}"
        );
        assert!(
            err.contains("ratchet"),
            "{flag} {value}: the budget gate names itself: {err}"
        );
    }
}

#[test]
fn scan_fails_past_the_hard_line_and_passes_clean() {
    let dir = tmp("gate-scan");
    std::fs::write(dir.join("a.rs"), "// filler\n".repeat(800)).expect("a.rs");
    gate_red_green(&dir, &run_scan, "file-lines", false, &|| {
        std::fs::write(dir.join("a.rs"), "fn tiny() -> i64 { 1 }\n").expect("shrink");
    });
}
