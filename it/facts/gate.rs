//! Gate facts: the two CI floors and the two dedup budgets.

use super::{Fact, linked, scraped};
use crate::common::repo_root;
use std::collections::BTreeSet;
use std::path::Path;

const FLOOR_DEBT: &str = "spelled on the ce check line of each CI job; promote = one workflow-level env var the jobs and the docs read";

pub fn facts() -> Vec<Fact> {
    let root = repo_root();
    let ci = std::fs::read_to_string(root.join(".github/workflows/ci.yml")).expect("ci.yml");
    vec![
        scraped(
            "gate:floor.main#digits",
            floor(&ci, "check .. "),
            ".github/workflows/ci.yml::--fail-under (check ..)",
            FLOOR_DEBT,
        ),
        scraped(
            "gate:floor.tests#digits",
            floor(&ci, "check tests "),
            ".github/workflows/ci.yml::--fail-under (check tests)",
            FLOOR_DEBT,
        ),
        linked(
            "gate:dedup.main#digits",
            budget(&root, "ce.toml"),
            "ce.toml::[dedup] budget (config::Config::load)",
        ),
        linked(
            "gate:dedup.tests#digits",
            budget(&root.join("cli/tests"), "cli/tests/ce.toml"),
            "cli/tests/ce.toml::[dedup] budget (config::Config::load)",
        ),
    ]
}

/// The one `--fail-under N` every ci.yml line running `ce check` on
/// `target` agrees on.
fn floor(ci: &str, target: &str) -> String {
    let values: BTreeSet<&str> = ci
        .lines()
        .filter(|l| l.contains(target))
        .filter_map(|l| l.split("--fail-under ").nth(1))
        .filter_map(|rest| rest.split_whitespace().next())
        .collect();
    assert_eq!(
        values.len(),
        1,
        "ci.yml: --fail-under for {target:?} = {values:?}"
    );
    values.into_iter().next().expect("one floor").to_string()
}

/// The declared dedup budget of the project at `root`, through the
/// product's own loader.
fn budget(root: &Path, rel: &str) -> usize {
    codeeraser::config::Config::load(root)
        .unwrap_or_else(|e| panic!("{rel}: {e}"))
        .dedup
        .budget
        .unwrap_or_else(|| panic!("{rel}: no [dedup] budget"))
}
