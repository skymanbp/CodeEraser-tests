//! `ce erase` refusals by name (split from erase_e2e.rs at its 300-line
//! soft line, lane v235_licence): a hand-built plan aiming below a
//! gitlink, a root that is no repository, and a hand-built whole-file
//! row (no planner makes one since 9.1.0) — each refused before any
//! write, the fixture's canary file intact.

use crate::common::{self, commit_all, core_bin, tmp};
use crate::erase_e2e::{PARA, hand_built, planned, refuses};
use codeeraser::erase;

/// A declared submodule is a READER of this tree (plan v2.18 step
/// #12): a dead file inside it is nobody's verdict here and is never
/// planned. The apply-side fence stays as defence in depth — the
/// repository ce verified and the subtree it writes must be one — so
/// a row a hand-built plan aims below a gitlink is REFUSED by name,
/// with nothing written in either repository.
#[test]
fn apply_refuses_a_target_below_a_gitlink() {
    let sup = common::seed_superproject("erase-submodule", "suite");
    std::fs::write(
        sup.join("suite/stray.md"),
        "an unlinked note nobody references\n",
    )
    .expect("md");
    commit_all(&sup.join("suite"), "note");
    std::fs::write(sup.join(".gitignore"), ".ce/\n").expect(".gitignore");
    commit_all(&sup, "bump");
    let core = core_bin();
    let mut plan = erase::plan(&sup, None, &core).expect("plan");
    assert!(
        plan.rows.iter().all(|r| r.path != "suite/stray.md"),
        "a reader is never planned: {:?}",
        plan.rows.iter().map(|r| &r.path).collect::<Vec<_>>()
    );
    plan.rows
        .push(hand_built(&sup, "suite/stray.md", Some((1, 1))));
    let err = erase::apply_plan(&sup, None, &core, &plan).expect_err("refuses");
    assert!(
        err.to_string().contains("below the submodule suite"),
        "{err:#}"
    );
    assert!(sup.join("suite/stray.md").exists(), "nothing written");
    for repo in [sup.clone(), sup.join("suite")] {
        let out = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&repo)
            .output()
            .expect("git status");
        assert!(out.stdout.is_empty(), "{}: untouched", repo.display());
    }
}

#[test]
fn apply_outside_a_repo_refuses() {
    let dir = tmp("erase-norepo"); // tmp()'s .git is an empty anchor dir, not a repository
    let w = |name: &str, text: String| std::fs::write(dir.join(name), text).expect(name);
    w("orphan.md", "an unlinked note nobody references\n".into());
    w("README.md", format!("# a\n\n[b](b.md)\n\n{PARA}\n"));
    w("b.md", format!("# b\n\n{PARA}\n"));
    let core = core_bin();
    let plan = erase::plan(&dir, None, &core).expect("plan");
    assert!(plan.counts.eraseable >= 1, "the verbatim span plans");
    refuses(&dir, &core, &plan, "not a git repository");
}

/// A hand-built plan that asks to delete a whole file (the shape every
/// dead_file / t1_twin row had before 9.1.0) is refused by name before
/// any precondition runs, and nothing is written: the executor holds
/// no file deletion.
#[test]
fn apply_refuses_a_whole_file_row_by_name() {
    let (dir, core, mut plan) = planned("erase-wholefile");
    plan.rows.push(hand_built(&dir, "orphan.md", None));
    refuses(&dir, &core, &plan, "orphan.md: a whole-file erase row");
    let doc2 = std::fs::read_to_string(dir.join("doc2.md")).expect("doc2");
    assert!(
        doc2.contains("alpha beta gamma"),
        "the span row was not applied either"
    );
}
