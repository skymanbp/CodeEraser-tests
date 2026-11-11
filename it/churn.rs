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
/// base. Before the fix `sha^..sha` failed, the window's additions stayed 0,
/// and blame still counted every surviving root line.
#[test]
fn root_commit_additions_are_counted() {
    let root = one_fn_repo("churn-root", 0);
    let r = churn::run(&root, 30).expect("churn");
    assert_eq!(r.commits, 1);
    assert!(added(&r) > 0, "root additions counted");
    assert_eq!(added(&r), r.surviving, "everything survives");
    // the whole seed lands in ONE ledger row: a.rs's work_1 unit
    assert_eq!(r.units.len(), 1, "rows: {:?}", r.units);
    let row = &r.units[0];
    assert_eq!((row.path.as_str(), row.key.as_str()), ("a.rs", "work_1/2"));
    assert!(
        row.anchor.ends_with("#0"),
        "a top-level unit's chain: {row:?}"
    );
    assert_eq!(row.appended, added(&r));
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
    assert!(
        r.commits >= 3,
        "the pointer bump is still a commit: {}",
        r.commits
    );
    assert_eq!(r.submodules_without_history, ["suite"]);
    let json = laid_out(&r);
    assert_eq!(json["schema"], common::stub_core::catalogue_schema("churn"));
    assert_eq!(
        json["submodules_without_file_history"],
        serde_json::json!(["suite"])
    );
}

/// Plan v2.30 step 5b item 29: a same-key sibling deleted AFTER the
/// commit that edited the survivor. Under the old (path, key, nth) key
/// the survivor's row said nth 1 (impl A's `add` still stood when it
/// was edited) while HEAD says nth 0, and a HEAD-side join found no
/// row; under the anchor the row and HEAD's unit agree.
#[test]
fn a_sibling_deleted_later_in_the_window_leaves_the_survivors_ledger_identity() {
    let root = common::tmp("churn-anchor");
    common::git(&root, &["init", "-q"]);
    let both = "impl A {\n    fn add(&self) -> u32 {\n        1\n    }\n}\nimpl B {\n    fn add(&self) -> u32 {\n        2\n    }\n}\n";
    std::fs::write(root.join("x.rs"), both).expect("x.rs");
    common::commit_all(&root, "both impls");
    let edited = both.replace("        2\n", "        2 + 40\n");
    std::fs::write(root.join("x.rs"), &edited).expect("edit B");
    common::commit_all(&root, "edit impl B's add");
    let b_only = &edited[edited.find("impl B").expect("impl B")..];
    std::fs::write(root.join("x.rs"), b_only).expect("delete A");
    common::commit_all(&root, "delete impl A");
    let r = churn::run(&root, 30).expect("churn");
    let head = codeeraser::fourclass::units::segments(b_only, codeeraser::scan::lang::Lang::Rust);
    let anchors = codeeraser::fourclass::anchor::for_units(&head);
    let (i, _) = head
        .iter()
        .enumerate()
        .find(|(_, u)| u.key == "add/1")
        .expect("HEAD's add");
    let row = r
        .units
        .iter()
        .find(|u| u.path == "x.rs" && u.key == "add/1" && u.anchor == anchors[i])
        .unwrap_or_else(|| panic!("no ledger row at HEAD's anchor: {:?}", r.units));
    assert!(
        row.rewrote >= 1,
        "the commit-2 edit is impl B's rewrite: {row:?}"
    );
}

/// v2.35 V1: one parent rule on both sides of the window. A feature
/// branch's 100 lines merged `--no-ff`, then half of them rewritten on
/// the main line, read as their linear twin: the merge's first-parent
/// diff is what it brought (it read 0 under `--no-merges` while blame
/// still counted every merged survivor, churn clamped to 0).
#[test]
fn a_merge_workflow_reads_as_its_linear_twin() {
    let lines = |odd: &str| -> String {
        (0..100)
            .map(|i| format!("v{i} = {i} * {} + 1\n", if i % 2 == 0 { odd } else { "7" }))
            .collect()
    };
    let seeded = |name: &str| {
        let root = common::tmp(name);
        common::git(&root, &["init", "-q"]);
        std::fs::write(root.join("a.py"), "x = 0\n").expect("a.py");
        common::commit_all(&root, "seed");
        root
    };
    let merged = seeded("churn-merged");
    common::git(&merged, &["checkout", "-q", "-b", "feat"]);
    common::append(&merged.join("a.py"), &lines("7"));
    common::commit_all(&merged, "feature adds 100");
    common::git(&merged, &["checkout", "-q", "-"]);
    let linear = seeded("churn-linear");
    common::append(&linear.join("a.py"), &lines("7"));
    common::commit_all(&linear, "adds 100");
    let mut sums = Vec::new();
    for root in [&merged, &linear] {
        std::fs::write(root.join("b.py"), "y = 1\n").expect("b.py");
        common::commit_all(root, "one line elsewhere");
        if root == &merged {
            common::git(
                root,
                &["merge", "-q", "--no-ff", "feat", "-m", "merge feat"],
            );
        }
        std::fs::write(root.join("a.py"), format!("x = 0\n{}", lines("9"))).expect("rewrite");
        common::commit_all(root, "rewrite 50 lines");
        let r = churn::run(root, 30).expect("churn");
        sums.push((added(&r), r.surviving, laid_out(&r)["churned"].clone()));
    }
    assert_eq!(sums[0], (152, 102, serde_json::json!(50)), "merged");
    assert_eq!(sums[0], sums[1], "the merge reads as its linear twin");
}

/// v2.35 V1 review: the ledger splits lines and does not see a CR or a
/// missing final newline; blame compares bytes and gives such a line to
/// the commit that only turned its end. An old commit outside the
/// window, then one window commit: the newline-only and CRLF → LF
/// commits read 0 survivors with the lines counted beyond the ledger
/// (they refused the whole report before), the control is unmoved, and
/// a mixed commit caps its survivors at the two lines it added.
/// Columns: old text, window text, [added, surviving, beyond_ledger].
#[test]
fn lines_blame_gives_past_the_ledger_are_counted_not_refused() {
    let three = [
        "fn one() -> i64 { 1 }",
        "fn two() -> i64 { 2 }",
        "fn three() -> i64 { 3 }",
    ];
    let (lf, crlf) = (three.join("\n") + "\n", three.join("\r\n") + "\r\n");
    let more = format!("{lf}fn four() -> i64 {{ 4 }}\nfn five() -> i64 {{ 5 }}\n");
    let cases = [
        ("newline", three.join("\n"), lf.clone(), [0, 0, 1]),
        ("crlf", crlf.clone(), lf.clone(), [0, 0, 3]),
        ("control", format!("{}\n", three[0]), lf.clone(), [2, 2, 0]),
        ("mixed", crlf, more, [2, 2, 3]),
    ];
    for (name, old, new, want) in cases {
        let root = common::tmp(&format!("churn-beyond-{name}"));
        common::git(&root, &["init", "-q"]);
        std::fs::write(root.join("m.rs"), old).expect("old");
        commit_bytes(&root, "old", 100);
        std::fs::write(root.join("m.rs"), new).expect("new");
        commit_bytes(&root, "window", 0);
        let r = churn::run(&root, 30).expect("churn");
        let doc = laid_out(&r);
        assert_eq!([added(&r), r.surviving, r.beyond_ledger], want, "{name}");
        assert_eq!(doc["beyond_ledger"], want[2], "{name}: the document's");
    }
}

/// A rename inside the window: blame names each line's path in its own
/// commit, so the two lines added to `a.rs` survive in `b.rs` under the
/// (commit, `a.rs`) the ledger counted them by.
#[test]
fn survivors_follow_a_rename_inside_the_window() {
    let root = one_fn_repo("churn-rename", 100);
    common::append(&root.join("a.rs"), "// one\n// two\n");
    commit_bytes(&root, "two lines", 0);
    common::git(&root, &["mv", "a.rs", "b.rs"]);
    commit_bytes(&root, "rename", 0);
    let r = churn::run(&root, 30).expect("churn");
    assert_eq!((r.surviving, r.beyond_ledger), (2, 0), "{:?}", r.units);
}

/// A fresh repository whose one commit, `days_ago` days back, seeds
/// `a.rs` with one function.
fn one_fn_repo(name: &str, days_ago: u64) -> std::path::PathBuf {
    let root = common::tmp(name);
    common::git(&root, &["init", "-q"]);
    std::fs::write(root.join("a.rs"), common::rust_fn(1)).expect("a.rs");
    commit_bytes(&root, "seed", days_ago);
    root
}

/// Stage every byte as written (no line-end conversion on add) and
/// commit, dated `days_ago` days back.
fn commit_bytes(root: &std::path::Path, msg: &str, days_ago: u64) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_secs();
    let when = format!("@{} +0000", now - days_ago * 86_400);
    let env = [("GIT_AUTHOR_DATE", &*when), ("GIT_COMMITTER_DATE", &*when)];
    common::git_env(root, &["-c", "core.autocrlf=false", "add", "."], &env);
    common::git_env(root, &["commit", "-qm", msg], &env);
}

fn assert_report(r: &churn::Report) {
    assert_eq!(r.commits, 3);
    assert!(
        r.submodules_without_history.is_empty(),
        "nothing declared, nothing named"
    );
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
        r.surviving < added(r),
        "b.rs's emptied function must count as churned: {} of {}",
        r.surviving,
        added(r)
    );
    let json = laid_out(r);
    assert_eq!(json["schema"], common::stub_core::catalogue_schema("churn"));
    assert_eq!(
        json["added_in_window"].as_u64().unwrap(),
        json["surviving"].as_u64().unwrap() + json["churned"].as_u64().unwrap(),
        "survival ledger must balance"
    );
}

/// The report as the core lays it out (plan v2.32 step 5: the churn
/// document is the core's, this side sends the window's integers).
/// Every line the window added: appended plus rewritten.
fn added(r: &churn::Report) -> usize {
    r.append_lines() + r.rewrite_lines()
}

fn laid_out(r: &churn::Report) -> serde_json::Value {
    churn::answer(&common::core_bin(), r, 30)
        .expect("laid out")
        .document
}

/// Per-unit attribution (M5-3h): every fixture edit is pinned to its
/// owning (path, key, anchor) row, and the top level ("" key) is a
/// real destination, not lost lines.
fn assert_ledger(r: &churn::Report) {
    let row = |p: &str, k: &str| r.units.iter().find(|u| u.path == p && u.key == k);
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
