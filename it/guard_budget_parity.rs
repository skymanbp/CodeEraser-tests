//! O21 (plan v2.29 step 8): the PreToolUse hard-budget rule is Rust
//! arithmetic with no wire — `guard::zone::budget_kind` refuses a write
//! iff `cap != 0 && after > cap && after > before` (v2.35 GD1: the
//! write is judged for what it ADDS) — while the CI wall is the core's
//! `gradeWith` over scan/1. Nothing proved the two agree, and a hook
//! that denied a write the wall would pass (or the reverse) would be a
//! silent policy fork. This leg proves the agreement through the real
//! binary, cell by cell: for every (declared table, target, size on disk
//! before, written length) the hook denies exactly when `ce scan` fails
//! on the bytes once written AND the write grew the file — under the
//! global table, a `[[rules.class]]` table, and cap 0 (no hard line). A
//! file already past the line that the write keeps or shrinks is the
//! wall's failure, never the write's. The table has to straddle the
//! line on both sides for the leg to mean anything, and it asserts that
//! it does.

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
    // tag, table, target, lines on disk before (0 = absent), written
    let cells: [(&str, &str, &str, usize, usize); 10] = [
        ("global-at", GLOBAL, "a.rs", 0, 30),
        ("global-over", GLOBAL, "a.rs", 0, 31),
        ("class-at", CLASSED, "pkg/b.rs", 0, 20),
        ("class-over", CLASSED, "pkg/b.rs", 0, 21),
        ("class-unclassed", CLASSED, "a.rs", 0, 25),
        ("no-line-huge", NO_LINE, "a.rs", 0, 5000),
        ("no-line-warn-only", NO_LINE, "a.rs", 0, 400),
        ("global-carried", GLOBAL, "a.rs", 40, 40),
        ("global-shrinks", GLOBAL, "a.rs", 40, 35),
        ("global-grows", GLOBAL, "a.rs", 40, 41),
    ];
    let mut verdicts = Vec::new();
    for (tag, toml, rel, before, lines) in cells {
        let dir = tmp(&format!("budget-parity-{tag}"));
        declare(&dir, toml);
        if before > 0 {
            write_all(&dir, &[(rel, &source(before))]);
        }
        common::build_index(&dir);
        let text = source(lines);
        let denied = hook_denies(&dir, rel, &text);
        let failed = scan_fails(&dir, rel, &text, &core);
        assert_eq!(
            denied,
            failed && lines > before,
            "{tag}: hook denies = {denied}, scan fails = {failed} ({rel}, {before} -> {lines} lines)"
        );
        verdicts.push((denied, failed));
    }
    assert!(
        verdicts.contains(&(true, true))
            && verdicts.contains(&(false, false))
            && verdicts.contains(&(false, true)),
        "the table straddles the line, and holds a failing file the write did not grow: {verdicts:?}"
    );
}
