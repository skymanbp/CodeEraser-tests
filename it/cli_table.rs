//! The README's CLI carrier table (plan v2.21 S5: derived tables).
//! The table is authored — its second column is prose — but its
//! first column is a claim about clap's roster, so the claim is
//! closed both ways: every subcommand the binary ships is either
//! named in a row or omitted here BY NAME with its reason, and every
//! name a row spells is a subcommand that ships. Both languages must
//! name the same set.

use crate::common::repo_root;
use crate::face_parity::cli_subcommands;
use std::collections::BTreeSet;

/// Subcommands the table leaves out on purpose — `name: why` per
/// line, one literal (a tuple table is a clone of every other one).
const OMITTED: &str = "
daemon: machine surface — every face starts it lazily
ping: daemon liveness probe, not a judgment
probe: the PreToolUse hook's private face
audit: the Stop hook's private face
health: the SessionStart hook's private face
precommit: the git pre-commit hook's private face
commitmsg: the git commit-msg hook's private face
";

/// The subcommand names the table's first column spells: every
/// backticked `ce <name>` in the rows under the `| Command |` /
/// `| 命令 |` header, up to the blank line that ends the table.
fn table_names(rel: &str) -> BTreeSet<String> {
    let text =
        std::fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
    let rows = text
        .lines()
        .skip_while(|l| !(l.starts_with("| Command |") || l.starts_with("| 命令 |")))
        .skip(2)
        .take_while(|l| l.starts_with('|'));
    let mut names = BTreeSet::new();
    for row in rows {
        let first = row.split('|').nth(1).expect("first cell");
        for cell in first.split('`').skip(1).step_by(2) {
            if let Some(name) = cell.strip_prefix("ce ") {
                names.insert(name.to_string());
            }
        }
    }
    assert!(!names.is_empty(), "{rel}: no CLI carrier table found");
    names
}

#[test]
fn the_carrier_table_names_every_subcommand_or_omits_it_by_name() {
    let roster = cli_subcommands();
    let omitted: BTreeSet<String> = OMITTED
        .lines()
        .filter_map(|l| l.split_once(':'))
        .map(|(name, _)| name.to_string())
        .collect();
    for rel in ["README.md", "README.zh.md"] {
        let named = table_names(rel);
        let both: Vec<&String> = named.intersection(&omitted).collect();
        assert!(both.is_empty(), "{rel}: named AND omitted: {both:?}");
        let claimed: BTreeSet<String> = named.union(&omitted).cloned().collect();
        if let Some(note) = crate::facts::both_ways(rel, &roster, &claimed) {
            panic!("{note}");
        }
    }
    assert_eq!(
        table_names("README.md"),
        table_names("README.zh.md"),
        "the two READMEs' carrier tables name different subcommands"
    );
}
