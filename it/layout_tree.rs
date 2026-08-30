//! §5.10 of the plan draws the repository layout (plan v2.21 S5:
//! derived tables). The drawing stays honest both ways: every
//! top-level entry it draws is a tracked directory (or the one
//! ignored-by-design directory it names as such), and every tracked
//! top-level directory is drawn — a directory added to the tree
//! without a line in the plan, or a line whose directory left, is a
//! red leg naming it.

use crate::common::repo_root;
use std::collections::BTreeSet;

/// Drawn but untracked on purpose — the tree itself says so.
const IGNORED: &[&str] = &["memory"];

/// The top-level entries of the layout tree: lines `├── name/` or
/// `└── name/` at depth one inside the §5.10 fence.
fn drawn() -> BTreeSet<String> {
    let plan = std::fs::read_to_string(repo_root().join("docs/DEVELOPMENT_PLAN.md")).expect("plan");
    let fence = plan
        .split("### 5.10 仓库布局")
        .nth(1)
        .expect("§5.10 heading")
        .split("```")
        .nth(1)
        .expect("§5.10 fence");
    fence
        .lines()
        .filter_map(|l| l.strip_prefix("├── ").or_else(|| l.strip_prefix("└── ")))
        .map(|rest| {
            rest.split_whitespace()
                .next()
                .expect("entry name")
                .trim_end_matches('/')
                .to_string()
        })
        .collect()
}

/// Every top-level directory with a tracked file under it.
fn tracked() -> BTreeSet<String> {
    let out = std::process::Command::new("git")
        .args(["ls-files"])
        .current_dir(repo_root())
        .output()
        .expect("git ls-files");
    assert!(out.status.success(), "git ls-files failed");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.split_once('/').map(|(dir, _)| dir.to_string()))
        .collect()
}

#[test]
fn the_plan_draws_every_top_level_directory_and_nothing_else() {
    let drawn = drawn();
    let mut expected = tracked();
    expected.extend(IGNORED.iter().map(|s| s.to_string()));
    if let Some(note) = crate::facts::both_ways("plan §5.10 layout tree", &expected, &drawn) {
        panic!("{note}");
    }
    for name in IGNORED {
        assert!(
            repo_root().join(name).is_dir() || std::env::var_os("CI").is_some(),
            "{name}/ is drawn as ignored-by-design and should exist locally"
        );
    }
}
