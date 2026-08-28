//! The seating contract of a declared submodule (plan v2.18: the suite
//! rides at `cli/tests` as the CodeEraser-tests submodule). A gitlink
//! is tree content a filesystem walk cannot see, so every judging
//! surface must either SEAT it or REFUSE by name — never a hollow
//! score: (1) trend seats a commit's gitlinks in nested worktrees and
//! the seated row equals the live checkout's own judgment; (2) an
//! unseated checkout refuses by name in the trend library, at the
//! measurement walk every family shares (`walk::collect`: scan, score,
//! dedup, graph), at the trend binary's exit code, and in `ce
//! baseline`, so a shrunken ratchet is never persisted; (3) a declared
//! path the exclusion model prunes judges nothing seated or not, and
//! is NOT refused (ruling: `vendor/` is out of the judgment).

use crate::common;
use std::path::Path;

/// The live checkout judged the way trend judges every point: a null
/// baseline against the tree's own warn line (no baseline is committed
/// in these fixtures, so that is what trend's pin resolves to).
fn live_score(sup: &Path) -> anyhow::Result<codeeraser::score::Outcome> {
    let soft = codeeraser::config::Config::load(sup)
        .expect("config")
        .thresholds
        .file_lines_warn as u64;
    codeeraser::score::run(
        sup,
        codeeraser::score::Opts {
            db: None,
            core: common::core_bin(),
            days: None,
            floor: None,
            establish: true,
            pinned_soft: Some(soft),
        },
    )
}

#[test]
fn a_seated_gitlink_judges_exactly_as_the_live_checkout() {
    let sup = common::seed_superproject("trend-super", "suite");
    let report = codeeraser::trend::run(&sup, None, &common::core_bin(), 10, None).expect("trend run");
    assert_eq!(
        report.rows.len(),
        2,
        "both mainline commits measured: {:?}",
        report.failed
    );
    let (before, mounted) = (&report.rows[0], &report.rows[1]);
    let live = live_score(&sup).expect("live score");
    assert_eq!(
        (mounted.score, &mounted.axes),
        (live.reply.score, &live.reply.axes),
        "the seated worktree must judge exactly as the live checkout"
    );
    assert_ne!(
        (before.score, &before.axes),
        (mounted.score, &mounted.axes),
        "the mounted pair must be IN the judgment — a hollow seat would repeat the first row"
    );
}

#[test]
fn an_unseated_submodule_is_a_named_refusal() {
    let sup = common::seed_superproject("trend-super-hollow", "suite");
    common::unseat(&sup, "suite");
    let report =
        codeeraser::trend::run(&sup, None, &common::core_bin(), 10, None).expect("trend run");
    assert_eq!(report.rows.len(), 1, "the pre-mount commit still measures");
    assert!(
        report
            .failed
            .iter()
            .any(|(_, why)| why.contains("not checked out")),
        "the mounted commit refuses by name, not a hollow score: {:?}",
        report.failed
    );
}

/// The measurement walk refuses first, so `ce check` cannot pass on a
/// tree missing its tests and `ce baseline` cannot persist the
/// shrunken ratchet as an improvement (fail ≡ added ∨ over, and a
/// vanished file is neither).
#[test]
fn an_unseated_submodule_refuses_the_measurement_walk() {
    let sup = common::seed_superproject("walk-super-hollow", "suite");
    common::unseat(&sup, "suite");
    for (what, err) in [
        ("scan", codeeraser::scan::measure(&sup).err().map(|e| format!("{e:#}"))),
        ("score", live_score(&sup).err().map(|e| format!("{e:#}"))),
    ] {
        let err = err.unwrap_or_else(|| panic!("{what} judged a hollow tree"));
        assert!(
            err.contains("suite") && err.contains("not checked out"),
            "{what} refuses by name: {err}"
        );
    }
    let out = common::run_ce(&sup, &["baseline", "."]);
    assert!(!out.status.success(), "ce baseline must not write: {out:?}");
    assert!(!sup.join("ce-baseline.json").exists(), "no shrunken ratchet on disk");
}

/// Ruling (1): a submodule the exclusion model prunes contributes
/// nothing seated or not, so its absence is not a refusal.
#[test]
fn an_excluded_unseated_submodule_is_not_refused() {
    let sup = common::seed_superproject("walk-super-vendored", "vendor/suite");
    common::unseat(&sup, "vendor/suite");
    codeeraser::scan::measure(&sup).expect("vendor/ is out of the judgment");
    live_score(&sup).expect("…for the score too");
}

/// The exit code reads `failed`, never `pending`: a refused point is a
/// named exit 1, while `--batch` leaving points pending is still 0.
/// Two fixtures: a row measured while seated is cached by its commit
/// and stays truthful after a deinit (history is the source), so the
/// batch run must not pre-measure the refusal run's mounted commit.
#[test]
fn a_refused_point_is_a_named_exit_1() {
    let seated = common::seed_superproject("trend-super-batch", "suite");
    let out = common::run_ce(&seated, &["trend", ".", "--batch", "1"]);
    assert!(out.status.success(), "pending points are not failures: {out:?}");
    let sup = common::seed_superproject("trend-super-exit", "suite");
    common::unseat(&sup, "suite");
    let out = common::run_ce(&sup, &["trend", "."]);
    let (stdout, stderr) = (
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(!out.status.success(), "a refused point exits 1");
    assert!(stdout.contains("FAILED"), "the report still names it: {stdout}");
    assert!(
        stderr.contains("trend check:") && stderr.contains("not checked out"),
        "the veto carries the reason: {stderr}"
    );
}
