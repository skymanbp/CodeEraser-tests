//! The trend worktree seats a commit's gitlinks (plan v2.18: the suite
//! rides at `cli/tests` as the CodeEraser-tests submodule). A detached
//! worktree renders a gitlink as an EMPTY directory, so without the
//! seat every post-switch commit would score a tree without its tests
//! and the trajectory would step where the tree did not. Two legs on
//! one seeded superproject: (1) the seated commit's row equals the
//! live checkout's own judgment under the same pinned soft line — the
//! seat put the same files under the same paths; (2) a superproject
//! whose submodule is not checked out is a NAMED refusal, never a
//! hollow score.

use crate::common::{self, git, rust_fn, tmp};
use std::path::Path;

/// A submodule repository with one committed T2 clone pair (the
/// trend_rebuild fixture's own pair) — content the superproject's
/// judgment can only see through the seat.
fn seed_sub(dir: &Path) {
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs");
    git(dir, &["init", "-q"]);
    common::commit_all(dir, "pair");
}

/// A superproject: one commit without the submodule, one that mounts
/// it at `suite/` — not under a name the scan walk excludes, which
/// would hide the pair from the live judgment too (the local-path
/// transport git refuses by default since 2.38.1 is allowed for this
/// fixture alone).
fn seed_super(dir: &Path, sub: &Path) {
    std::fs::write(dir.join("root.rs"), rust_fn(3)).expect("root.rs");
    git(dir, &["init", "-q"]);
    common::commit_all(dir, "root");
    let url = sub.to_str().expect("utf8").replace('\\', "/");
    git(
        dir,
        &["-c", "protocol.file.allow=always", "submodule", "add", "-q", &url, "suite"],
    );
    common::commit_all(dir, "mount");
}

fn trend(dir: &Path, core: &str) -> codeeraser::trend::Report {
    codeeraser::trend::run(dir, None, core, 10, None).expect("trend run")
}

#[test]
fn a_seated_gitlink_judges_exactly_as_the_live_checkout() {
    let sub = tmp("trend-sub");
    seed_sub(&sub);
    let sup = tmp("trend-super");
    seed_super(&sup, &sub);
    let core = common::core_bin();
    let report = trend(&sup, &core);
    assert_eq!(report.rows.len(), 2, "both mainline commits measured: {:?}", report.failed);
    let (before, mounted) = (&report.rows[0], &report.rows[1]);
    // the live checkout, judged the way trend judges every point: a
    // null baseline against the tree's own warn line (no baseline is
    // committed here, so that is what trend's pin resolves to)
    let soft = codeeraser::config::Config::load(&sup)
        .expect("config")
        .thresholds
        .file_lines_warn as u64;
    let live = codeeraser::score::run(
        &sup,
        codeeraser::score::Opts {
            db: None,
            core,
            days: None,
            floor: None,
            establish: true,
            pinned_soft: Some(soft),
        },
    )
    .expect("live score");
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
    let sub = tmp("trend-sub-hollow");
    seed_sub(&sub);
    let sup = tmp("trend-super-hollow");
    seed_super(&sup, &sub);
    git(&sup, &["submodule", "deinit", "-f", "-q", "suite"]);
    let report = trend(&sup, &common::core_bin());
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
