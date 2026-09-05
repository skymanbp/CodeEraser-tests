//! O21 (plan v2.29 step 8): the PreToolUse hard-budget rule is Rust
//! arithmetic with no wire — `guard/budget.rs::budget_breach` passes a
//! write iff `cap == 0 || lines <= cap` — while the CI wall is the
//! core's `gradeWith` over scan/1. Nothing proved the two agree, and a
//! hook that denied a write the wall would pass (or the reverse) would
//! be a silent policy fork. This leg proves the agreement through the
//! real binary, cell by cell: for every (declared table, target, written
//! length) the hook's decision on the write equals `ce scan`'s exit on
//! the same bytes once written — under the global table, a
//! `[[rules.class]]` table, and cap 0 (no hard line). The table has to
//! straddle the line on both sides for the leg to mean anything, and it
//! asserts that it does.

use crate::common;
use crate::common::pretooluse_envelope_at as envelope;
use crate::common::{core_bin, declare, tmp, write_all};
use std::path::Path;

const GLOBAL: &str =
    "[guard]\nmode = \"deny\"\n\n[thresholds]\nfile_lines_warn = 10\nfile_lines_fail = 30\n";
const CLASSED: &str = "[guard]\nmode = \"deny\"\n\n[thresholds]\nfile_lines_warn = 10\nfile_lines_fail = 30\n\n\
[[rules.class]]\nname = \"pkg\"\nglobs = [\"pkg/\"]\n\n[rules.class.knobs]\nfile_lines_warn = 10\nfile_lines_fail = 20\n";
const NO_LINE: &str = "[guard]\nmode = \"deny\"\n\n[thresholds]\nfile_lines_fail = 0\n";

/// `n` distinct lines of Rust, no clone bait (the dup probe must stay
/// silent so the decision is the budget rule's alone).
fn source(n: usize) -> String {
    (1..=n)
        .map(|i| format!("const L{i}: i64 = {i};\n"))
        .collect()
}

/// The hook's verdict on writing `text` at `rel`: true = denied. An
/// empty stdout is the observe/allow shape (nothing fired).
fn hook_denies(dir: &Path, rel: &str, text: &str) -> bool {
    let out = common::run_hook(
        dir,
        &["probe", "--hook"],
        &envelope(dir, rel, "Write", text),
    );
    if out.trim().is_empty() {
        return false;
    }
    let v: serde_json::Value = serde_json::from_str(out.trim()).expect("decision json");
    let reason = v["hookSpecificOutput"]["permissionDecisionReason"].to_string();
    assert!(
        reason.contains("lines"),
        "{rel}: only the budget rule may speak here: {reason}"
    );
    v["hookSpecificOutput"]["permissionDecision"] == "deny"
}

/// `ce scan`'s verdict on the same bytes on disk: true = FAIL (exit 1).
fn scan_fails(dir: &Path, rel: &str, text: &str, core: &str) -> bool {
    write_all(dir, &[(rel, text)]);
    let (code, out, err) = common::ce_triple(dir, &["scan", ".", "--core", core], &[]);
    assert!(
        matches!(code, Some(0 | 1)),
        "{rel}: scan exit {code:?}\n{out}\n{err}"
    );
    code == Some(1)
}

#[test]
fn the_hook_budget_and_the_scan_wall_agree_cell_by_cell() {
    let core = core_bin();
    let cells: [(&str, &str, &str, usize); 7] = [
        ("global-at", GLOBAL, "a.rs", 30),
        ("global-over", GLOBAL, "a.rs", 31),
        ("class-at", CLASSED, "pkg/b.rs", 20),
        ("class-over", CLASSED, "pkg/b.rs", 21),
        ("class-unclassed", CLASSED, "a.rs", 25),
        ("no-line-huge", NO_LINE, "a.rs", 5000),
        ("no-line-warn-only", NO_LINE, "a.rs", 400),
    ];
    let mut verdicts = Vec::new();
    for (tag, toml, rel, lines) in cells {
        let dir = tmp(&format!("budget-parity-{tag}"));
        declare(&dir, toml);
        common::build_index(&dir);
        let text = source(lines);
        let denied = hook_denies(&dir, rel, &text);
        let failed = scan_fails(&dir, rel, &text, &core);
        assert_eq!(
            denied, failed,
            "{tag}: hook denies = {denied}, scan fails = {failed} ({rel}, {lines} lines)"
        );
        verdicts.push(denied);
    }
    assert!(
        verdicts.contains(&true) && verdicts.contains(&false),
        "the table straddles the line: {verdicts:?}"
    );
}
