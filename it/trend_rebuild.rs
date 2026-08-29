//! M7-P4 acceptance: trend data re-derives from git history — the
//! cache is a CACHE (charter ruling ②). Three legs on one seeded
//! two-commit repo: (1) a full run measures every mainline commit;
//! (2) wiping the database and re-running reproduces the SAME rows
//! (history is the source of truth); (3) an untouched re-run is a
//! pure cache read — same rows, nothing pending, nothing failed.

use crate::common;
use crate::common::{rust_fn, tmp};
use std::path::Path;

fn seed_two_commits(dir: &Path) {
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    common::init_and_commit(dir, "one");
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs");
    // a non-default knob: the tree's digest is Some from here on, the
    // road every real project's history walks (O34)
    common::seed_budget(dir, 41);
    common::commit_all(dir, "two");
}

fn run_all(dir: &Path, core: &str) -> codeeraser::trend::Report {
    codeeraser::trend::run(dir, None, core, 10, None).expect("trend run")
}

#[test]
fn trend_rows_rebuild_from_history() {
    let dir = tmp("trend-rebuild");
    seed_two_commits(&dir);
    let core = common::core_bin();

    assert!(
        codeeraser::config::Config::load(&dir)
            .expect("config")
            .knobs_digest()
            .is_some(),
        "the seeded tree declares a non-default knob"
    );
    let first = run_all(&dir, &core);
    assert_eq!(first.window, 2, "two mainline commits seeded");
    assert_eq!(first.rows.len(), 2, "both measured: {:?}", first.failed);
    assert_eq!((first.pending, first.failed.len()), (0, 0));
    assert!(
        first.rows[0].ts <= first.rows[1].ts,
        "rows come oldest-first"
    );
    // the second commit lands a T2 clone pair, so its judged facts
    // must differ — a constant-score fixture would prove nothing
    // about per-commit measurement (commit hashes differing is not
    // evidence; the judgment payload is)
    assert!(
        (first.rows[0].score, &first.rows[0].axes) != (first.rows[1].score, &first.rows[1].axes),
        "the two commits must JUDGE differently: {:?}",
        first.rows
    );

    // Leg 3 first (cheap): an untouched re-run is a pure cache read.
    let cached = run_all(&dir, &core);
    assert_eq!(cached.rows, first.rows, "cache read drifted");

    // Leg 2: history is the truth — wipe the whole database and the
    // same rows come back from re-measurement.
    std::fs::remove_dir_all(dir.join(".ce")).expect("wipe .ce");
    let rebuilt = run_all(&dir, &core);
    assert_eq!(
        rebuilt.rows, first.rows,
        "rebuild from history must reproduce the cached rows"
    );
    assert_eq!((rebuilt.pending, rebuilt.failed.len()), (0, 0));
}

/// Batching arithmetic: batch=1 measures exactly one commit per run
/// and reports the remainder as pending — the GUI's progress loop
/// contract, pinned as a table over consecutive runs.
#[test]
fn trend_batch_measures_incrementally() {
    let dir = tmp("trend-batch");
    seed_two_commits(&dir);
    let core = common::core_bin();

    for (rows_want, pending_want) in [(1, 1), (2, 0)] {
        let r = codeeraser::trend::run(&dir, None, &core, 10, Some(1)).expect("batch run");
        assert_eq!((r.rows.len(), r.pending), (rows_want, pending_want));
    }
}

/// O34: the pinned-soft baseline is the request's own identity, so a
/// tree whose ce.toml is not the shipped default judges clean under
/// its own digest with a clone pair in it — the old two-empty-tables
/// pin failed every such point by `knobs_digest` and `discrete_added`,
/// and trend recorded the score anyway. The echoed newBaseline
/// carries the digest and the pinned line, nothing added, nothing over.
#[test]
fn a_pinned_soft_point_judges_clean_under_its_own_digest() {
    let dir = tmp("trend-pin");
    common::seed_clone_pair(&dir);
    common::seed_budget(&dir, 41);
    let digest = codeeraser::config::Config::load(&dir)
        .expect("config")
        .knobs_digest()
        .expect("non-default");
    let out = codeeraser::score::run(
        &dir,
        codeeraser::score::Opts {
            db: None,
            core: common::core_bin(),
            days: None,
            floor: None,
            establish: true,
            pinned_soft: Some(300),
            baseline: None,
        },
    )
    .expect("pinned run");
    let r = &out.reply;
    assert!(!r.fail, "identity pin: nothing holds: {:?}", r.failed);
    assert!(
        r.added.is_empty() && r.over.is_empty(),
        "{:?} {:?}",
        r.added,
        r.over
    );
    assert_eq!(r.new_baseline["knobsDigest"], serde_json::json!(digest));
    assert_eq!(r.new_baseline["softLine"], serde_json::json!(300));
    assert!(out.members > 0, "the clone pair is in the discrete set");
}
