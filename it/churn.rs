//! `ce churn` end-to-end over a synthetic history: append vs rewrite
//! classification, per-unit ledger attribution, co-change counting,
//! and window survival, on commits authored NOW (so any window covers
//! them — no clock injection needed for determinism).

use crate::common;
use codeeraser::churn;

/// The three-commit shape this battery asserts on lives in the
/// fixtures leaf (common::seed_churn_history) since the progress
/// face needed the same history: the comments describing what each
/// commit contributes moved with it, and a second copy of the
/// sequence here was the twin the dedup ratchet caught.
#[test]
fn churn_classifies_append_rewrite_cochange_and_survival() {
    let root = common::tmp("churn-e2e");
    common::seed_churn_history(&root);
    assert_report(&churn::run(&root, 30).expect("churn"));
}

/// Attack review F12 (minimal surviving-root case): a root commit
/// inside the window must count its additions via the empty-tree
/// base. Before the fix `sha^..sha` failed, added_in_window stayed 0,
/// and blame still counted every surviving root line.
#[test]
fn root_commit_additions_are_counted() {
    let root = common::tmp("churn-root");
    common::git(&root, &["init", "-q"]);
    std::fs::write(root.join("a.rs"), common::rust_fn(1)).expect("a.rs");
    common::commit_all(&root, "seed");
    let r = churn::run(&root, 30).expect("churn");
    assert_eq!(r.commits, 1);
    assert!(r.added_in_window() > 0, "root additions counted");
    assert_eq!(r.added_in_window(), r.surviving, "everything survives");
    // the whole seed lands in ONE ledger row: a.rs's work_1 unit
    assert_eq!(r.units.len(), 1, "rows: {:?}", r.units);
    let row = &r.units[0];
    assert_eq!(
        (row.path.as_str(), row.key.as_str(), row.nth),
        ("a.rs", "work_1/2", 0)
    );
    assert_eq!(row.appended, r.added_in_window());
    assert_eq!(row.rewrote, 0, "an empty before side cannot rewrite");
}

/// A declared submodule's judged files can take no ledger row — their
/// history is the child's, the parent sees pointer bumps — and the
/// report SAYS so (plan v2.18 follow-up), while the bump itself still
/// counts as a commit: absent rows, named, not degraded ones.
#[test]
fn a_declared_submodule_with_no_parent_history_is_named_not_a_silent_zero() {
    let sup = common::seed_superproject("churn-submodule", "suite");
    common::append(&sup.join("suite/b.rs"), "// child edit\n");
    common::commit_all(&sup.join("suite"), "child edit");
    common::commit_all(&sup, "bump");
    let r = churn::run(&sup, 30).expect("churn");
    assert!(
        r.units.iter().all(|u| !u.path.starts_with("suite/")),
        "no ledger row from the submodule: {:?}",
        r.units
    );
    assert!(r.commits >= 3, "the pointer bump is still a commit: {}", r.commits);
    assert_eq!(r.submodules_without_history, ["suite"]);
    let json = churn::report_json(&r);
    assert_eq!(json["schema"], "ce.churn-report/0.2.0");
    assert_eq!(json["submodules_without_file_history"], serde_json::json!(["suite"]));
}

fn assert_report(r: &churn::Report) {
    assert_eq!(r.commits, 3);
    assert!(r.submodules_without_history.is_empty(), "nothing declared, nothing named");
    assert!(
        r.rewrite_lines() >= 2,
        "work_1 edits are rewrite: {}",
        r.rewrite_lines()
    );
    assert!(
        r.append_lines() > r.rewrite_lines(),
        "seed + b.rs dominate as append: {} vs {}",
        r.append_lines(),
        r.rewrite_lines()
    );
    assert_ledger(r);
    assert!(
        r.cochange.contains(&("a.rs".into(), "b.rs".into(), 2)),
        "co-change pair counted twice: {:?}",
        r.cochange
    );
    assert!(
        r.surviving < r.added_in_window(),
        "b.rs's emptied function must count as churned: {} of {}",
        r.surviving,
        r.added_in_window()
    );
    let json = churn::report_json(r);
    assert_eq!(json["schema"], "ce.churn-report/0.2.0");
    assert_eq!(
        json["added_in_window"].as_u64().unwrap(),
        json["surviving"].as_u64().unwrap() + json["churned"].as_u64().unwrap(),
        "survival ledger must balance"
    );
}

/// Per-unit attribution (M5-3h): every fixture edit is pinned to its
/// owning (path, key, nth) row, and the top level ("" key) is a real
/// destination, not lost lines.
fn assert_ledger(r: &churn::Report) {
    let row = |p: &str, k: &str| {
        r.units
            .iter()
            .find(|u| u.path == p && u.key == k && u.nth == 0)
    };
    let work1 = row("a.rs", "work_1/2").expect("a.rs work_1 row");
    assert!(work1.rewrote >= 2, "commit-2 edits: {work1:?}");
    assert!(work1.appended >= 8, "the seed body: {work1:?}");
    let work2 = row("b.rs", "work_2/2").expect("b.rs work_2 row");
    assert!(work2.appended >= 8, "new file is append: {work2:?}");
    assert_eq!(work2.rewrote, 0, "no before side in b.rs: {work2:?}");
    let tail = row("a.rs", "").expect("a.rs top-level row");
    assert!(tail.appended >= 1, "// tail lands at top level: {tail:?}");
    let emptied = row("b.rs", "").expect("b.rs top-level row");
    assert!(
        emptied.appended >= 1,
        "// emptied is top-level: {emptied:?}"
    );
}
