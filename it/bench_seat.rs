//! The bench replay's seat helper (the four-ruling commit 1de6696:
//! "the bench per-tag backfill gets a seat helper before v1.3.0"). A
//! `git worktree add` of a tag leaves every gitlink EMPTY, and since
//! the tests moved into a submodule (L round step #11) the product
//! refuses an unseated declaration by name — so a replay that did not
//! seat would measure a refusal, not the tag. Two facts: a detached
//! worktree of a superproject seats its declared submodule through the
//! helper, and a tree without `.gitmodules` seats nothing.

use crate::bench_support as bs;
use crate::common;
use std::process::Command;

#[test]
fn a_detached_worktree_seats_its_declared_submodule() {
    let sup = common::seed_superproject("bench-seat", "suite");
    let wt = common::tmp("bench-seat-wt");
    // git wants the worktree path ABSENT — tmp() hands back an empty
    // dir, which git 2.52 refuses as "already exists"
    std::fs::remove_dir_all(&wt).expect("clear worktree path");
    let added = Command::new("git")
        .args(["worktree", "add", "--detach"])
        .arg(&wt)
        .arg("HEAD")
        .current_dir(&sup)
        .output()
        .expect("git worktree add");
    assert!(
        added.status.success(),
        "worktree add: {}",
        String::from_utf8_lossy(&added.stderr)
    );
    assert!(
        codeeraser::gitmodules::seated(&wt).is_empty(),
        "a fresh worktree carries the gitlink unseated"
    );

    let seated = bs::seat_submodules(&wt).expect("seat");
    assert_eq!(
        seated,
        vec!["suite".to_string()],
        "the declared mount seats"
    );
    assert!(
        wt.join("suite").join(".git").exists(),
        "the seated checkout carries its own git anchor"
    );
    assert_eq!(
        codeeraser::gitmodules::seated(&wt),
        vec!["suite".to_string()],
        "the product's own seated predicate agrees"
    );

    let plain = common::tmp("bench-seat-plain");
    assert!(
        bs::seat_submodules(&plain).expect("plain").is_empty(),
        "no .gitmodules — nothing to seat, no git call"
    );
    let _ = Command::new("git")
        .args(["worktree", "remove", "--force"])
        .arg(&wt)
        .current_dir(&sup)
        .output();
}
