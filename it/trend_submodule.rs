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
use std::path::PathBuf;

/// The seeded superproject: one commit of its own, then one that
/// mounts a submodule holding common::seed_clone_pair's T2 pair at
/// `suite/` — not under a name the judgment excludes (`vendor/` hid
/// the pair from the live score too, measured). The local-path
/// transport git refuses by default since 2.38.1 is allowed for this
/// fixture alone. Returns the superproject.
fn superproject(name: &str) -> PathBuf {
    let sub = tmp(&format!("{name}-sub"));
    common::seed_clone_pair(&sub);
    common::init_and_commit(&sub, "pair");
    let sup = tmp(name);
    std::fs::write(sup.join("root.rs"), rust_fn(3)).expect("root.rs");
    common::init_and_commit(&sup, "root");
    let url = sub.to_str().expect("utf8").replace('\\', "/");
    git(
        &sup,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            &url,
            "suite",
        ],
    );
    common::commit_all(&sup, "mount");
    sup
}

#[test]
fn a_seated_gitlink_judges_exactly_as_the_live_checkout() {
    let sup = superproject("trend-super");
    let core = common::core_bin();
    let report = codeeraser::trend::run(&sup, None, &core, 10, None).expect("trend run");
    assert_eq!(
        report.rows.len(),
        2,
        "both mainline commits measured: {:?}",
        report.failed
    );
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
    let sup = superproject("trend-super-hollow");
    git(&sup, &["submodule", "deinit", "-f", "-q", "suite"]);
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
