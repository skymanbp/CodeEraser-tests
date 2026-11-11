//! A run SCOPED below the project and a project ROOTED below git's
//! toplevel (v2.35 G1, R2-1 / R2-3). A scoped `ce dedup a` used to
//! rewrite the project's shared index with scope-relative keys and
//! reap every other row, so the PreToolUse probe had nothing to match
//! outside the scope until the next whole run, and the package cache
//! landed in `a/.ce/`. A ce.toml sub-project inside a git repository
//! read toplevel-relative paths off git and joined them against its
//! root-relative files: every surviving line read as churned.

use crate::common;
use std::path::Path;

/// Every path the shared index holds, in order.
pub fn indexed(dir: &Path) -> Vec<String> {
    let db = rusqlite::Connection::open(dir.join(".ce/index.db")).expect("open the shared index");
    let mut stmt = db
        .prepare("SELECT path FROM files ORDER BY 1")
        .expect("prepare");
    let paths = stmt.query_map([], |r| r.get(0)).expect("query");
    paths.collect::<Result<_, _>>().expect("rows")
}

#[test]
fn a_scoped_run_leaves_the_shared_index_and_the_probe_alone() {
    let dir = common::tmp("scoped-index");
    let (seven, one) = (common::rust_fn(7), common::rust_fn(1));
    common::write_all(
        &dir,
        &[
            ("ce.toml", "[guard]\nmode = \"deny\"\n"),
            ("a/one.rs", seven.as_str()),
            ("b/two.rs", one.as_str()),
        ],
    );
    common::build_index(&dir);
    let whole = indexed(&dir);
    assert_eq!(whole, ["a/one.rs", "b/two.rs"], "the whole run's rows");

    let out = common::run_ce(&dir, &["dedup", "a"]);
    assert!(out.status.success(), "scoped dedup: {out:?}");
    assert_eq!(
        indexed(&dir),
        whole,
        "the scoped run neither rekeys nor reaps the shared index"
    );
    assert!(
        !dir.join("a/.ce").exists(),
        "nothing is planted beside the scope"
    );

    // the probe judges from the project root: the duplicate of b/two.rs
    // written into a/ is still found
    let env = common::pretooluse_envelope_at(&dir, "a/new.rs", "Write", &one);
    let reason = common::expect_decision(&dir, &env, "deny");
    std::fs::remove_dir_all(&dir).ok();
    assert!(
        reason.contains("two.rs"),
        "the deny names b/two.rs: {reason}"
    );
}

#[test]
fn a_sub_project_below_the_git_toplevel_reads_its_own_history() {
    let top = common::tmp("scoped-subproject");
    let x = "def f(a):\n    return a + 1\n";
    common::write_all(
        &top,
        &[
            ("sub/ce.toml", "\n"),
            ("sub/x.py", x),
            ("other/y.py", "def g(b):\n    return b\n"),
        ],
    );
    common::init_and_commit(&top, "one");
    let more: String = (0..5)
        .map(|i| format!("\ndef h{i}(v):\n    return v * {i} + 3\n"))
        .collect();
    common::write_all(&top, &[("sub/x.py", format!("{x}{more}"))]);
    common::commit_all(&top, "two");

    let report = codeeraser::churn::run(&top.join("sub"), 14).expect("churn");
    let paths: Vec<&str> = report.units.iter().map(|u| u.path.as_str()).collect();
    assert!(
        paths.iter().all(|p| *p == "x.py"),
        "ledger rows are root-relative and the other directory is not ours: {paths:?}"
    );
    let added: usize = report.units.iter().map(|u| u.appended + u.rewrote).sum();
    assert!(added > 0, "the window's additions were attributed");
    assert_eq!(report.surviving, added, "every added line survives at HEAD");
    std::fs::remove_dir_all(&top).ok();
}
