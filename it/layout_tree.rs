//! §5.10 of the plan draws the repository layout (plan v2.21 S5:
//! derived tables). The drawing stays honest both ways: every
//! top-level entry it draws is a tracked directory (or the one
//! ignored-by-design directory it names as such), and every tracked
//! top-level directory is drawn — a directory added to the tree
//! without a line in the plan, or a line whose directory left, is a
//! red leg naming it. An ignored-by-design name is a fact of the
//! repository, not of the machine: the tracked root `.gitignore`
//! ignores it, by git's own matcher. Whether the directory exists on
//! this machine is never read.

use crate::common::repo_root;
use std::collections::BTreeSet;

/// Drawn but untracked on purpose — the tree itself says so, in its
/// root `.gitignore`.
/// cc-memory renamed its handoff directory `memory/` -> `.ccm/` on
/// 2026-08-30; the plan's tree followed, and this name follows the
/// plan, because the pair is what the gate compares.
const IGNORED: &[&str] = &[".ccm"];

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
        let rule = ignored_by(&format!("{name}/"));
        assert!(
            matches!(&rule, Some((source, pattern)) if source == ".gitignore" && !pattern.is_empty()),
            "{name}/ is drawn as ignored-by-design, but no pattern of the tracked .gitignore ignores it ({rule:?})"
        );
    }
}

/// The ignore file and the pattern that ignore `path` (`git check-ignore
/// -v` in the repository root), or None when git does not ignore it. The
/// source is asked because `.git/info/exclude` and a global excludes file
/// also answer `check-ignore`, and neither is the repository's; the
/// pattern, because on a CRLF checkout of the LF `.gitignore` git
/// (2.52.0.windows.1, measured) reads a blank line as an empty pattern
/// that answers any `name/`.
fn ignored_by(path: &str) -> Option<(String, String)> {
    let out = std::process::Command::new("git")
        .args(["check-ignore", "-v", path])
        .current_dir(repo_root())
        .output()
        .expect("git check-ignore");
    assert!(
        matches!(out.status.code(), Some(0 | 1)),
        "git check-ignore {path}: {out:?}"
    );
    let text = String::from_utf8_lossy(&out.stdout);
    let (source, rest) = text.split_once(':')?;
    let (pattern, _) = rest.split_once(':')?.1.split_once('\t')?;
    out.status
        .success()
        .then(|| (source.to_string(), pattern.to_string()))
}
