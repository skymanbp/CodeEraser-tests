//! The Stop / precommit leg of the tombstone measurement through the
//! real hook and the real terminal face (plan v2.26 step 5): the whole
//! session's diff is one changeset, so a name erased in one file and
//! written back as an absence in another is one site; a changelog-
//! role document is exempt and COUNTED; a move keeps the name alive;
//! precommit says what fired, once, and stays exit 0 in observe.

use crate::common;
use crate::tombstone_guard::{site, sites};
use std::path::{Path, PathBuf};

/// The README rewrite every fixture below writes: one heading framing
/// the name a.py declared.
const FRAMED: &str = "--- README.md\n# Intro\n\n## Without braise_pork\n";

/// A committed repo holding a Python module and a README, in observe
/// mode — the two files the cross-file fixtures move a name between —
/// with a.py already deleted from the working tree: `braise_pork` is
/// erased unless a later write brings it back.
fn erased(name: &str) -> PathBuf {
    let dir = common::tmp(name);
    common::write_doc(
        &dir,
        "--- a.py\ndef braise_pork():\n    return 1\n--- README.md\n# Intro\n\nHello.\n--- ce.toml\n[guard]\nmode = \"observe\"\n",
    );
    common::init_and_commit(&dir, "seed");
    std::fs::remove_file(dir.join("a.py")).expect("delete a.py");
    dir
}

/// `ce precommit` on the staged set: exit 0 (observe never blocks),
/// its stdout, and the feed line it wrote.
fn precommit(dir: &Path) -> (String, serde_json::Value) {
    let out = common::run_ce(dir, &["precommit"]);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(out.status.success(), "observe mode never blocks: {stdout}");
    let line = common::last_observe(dir);
    assert_eq!(line["event"], "precommit", "{line}");
    (stdout, line)
}

/// The `tombstone` object of the Stop audit's feed line.
fn stop_tombstone(dir: &Path) -> serde_json::Value {
    let line = common::stop_observe(dir);
    let t = line["tombstone"].clone();
    assert!(t.is_object(), "the Stop line carries the object: {line}");
    t
}

/// The Stop leg on one fixture written over the erased repo: its
/// sites are exactly `expected`, its exemptions exactly `exempt`.
fn stop_judges(tag: &str, doc: &str, expected: &[String], exempt: serde_json::Value) {
    let dir = erased(tag);
    common::write_doc(&dir, doc);
    let t = stop_tombstone(&dir);
    assert_eq!(sites(&t), expected, "{t}");
    assert_eq!(t["exempt"], exempt, "{t}");
}

#[test]
fn a_name_erased_in_one_file_and_framed_in_another_is_one_site() {
    // ④ the changeset is the whole diff, not one file
    let dir = erased("tomb-audit-cross");
    common::write_doc(&dir, FRAMED);
    let t = stop_tombstone(&dir);
    assert_eq!(sites(&t), [site("README.md", 3, "bare")], "{t}");
    assert_eq!(
        (t["label"].as_u64(), t["prose"].as_u64()),
        (Some(1), Some(0))
    );
    assert!(t["erased"].as_u64().expect("erased") >= 1);
    assert!(
        t.get("erased_hashes").is_none(),
        "the Stop line carries no keys"
    );
}

#[test]
fn a_changelog_role_document_is_exempt_and_counted() {
    let dir = erased("tomb-audit-exempt");
    // ⑪ the path convention; ⑫ the ledger shape under an ordinary name
    common::write_doc(
        &dir,
        "--- CHANGELOG.md\n# Changelog\n\n## Unreleased\n\n- braise_pork is no longer needed.\n\
         --- docs/notes.md\n# Notes\n\n## 1.6.0\n\n- Sides (no braise_pork)\n\n## 2026-09-04\n\n- more\n\n## Unreleased\n\n- x\n",
    );
    let t = stop_tombstone(&dir);
    assert!(sites(&t).is_empty(), "{t}");
    let exempt: Vec<String> = t["exempt"]
        .as_array()
        .expect("exempt")
        .iter()
        .map(|e| {
            assert!(e.get("line").is_none(), "a file-level entry: {e}");
            format!(
                "{} {}",
                e["file"].as_str().unwrap(),
                e["why"].as_str().unwrap()
            )
        })
        .collect();
    assert_eq!(exempt, ["CHANGELOG.md path", "docs/notes.md ledger"]);
}

#[test]
fn a_ledger_segment_is_exempt_by_line_and_counted() {
    // the third witness (plan v2.27): the README's banner is a version
    // ledger by itself — exempt as a segment, the section below judged
    stop_judges(
        "tomb-audit-segment",
        "--- README.md\n# Intro\n\n> v1.5.1 2026-09-02 47efc44 · v1.5.0 2026-09-01 · v1.4.1 65928ac — \
         braise_pork is no longer needed.\n\n## Without braise_pork\n\nSince 1.6.0.\n",
        &[site("README.md", 5, "bare")],
        serde_json::json!([{"file": "README.md", "line": 3, "why": "segment"}]),
    );
}

#[test]
fn a_declared_ledger_is_exempt_whole_and_a_term_is_never_a_name() {
    // `[tombstone] ledger` (plan v2.27 step 3): the repository's own
    // word for a ledger no witness reads — exempt whole, feed `declared`;
    // `terms` keeps `pork` out of every name, so `braise_pork` erased
    // leaves only `braise` to bind, and the README's heading binds it
    stop_judges(
        "tomb-audit-declared",
        "--- ce.toml\n[guard]\nmode = \"observe\"\n\n[tombstone]\nledger = [\"docs/\"]\nterms = [\"pork\"]\n\
         --- docs/plan.md\n# Plan\n\n## Without braise_pork\n\n- Sides (no braise)\n\
         --- README.md\n# Intro\n\n## Without braise\n",
        &[site("README.md", 3, "bare")],
        serde_json::json!([{"file": "docs/plan.md", "why": "declared"}]),
    );
}

#[test]
fn a_move_keeps_the_name_alive() {
    // ⑮ the name went to another changed file: nothing was erased
    let dir = erased("tomb-audit-move");
    common::write_doc(
        &dir,
        &format!("--- b.py\ndef braise_pork():\n    return 1\n{FRAMED}"),
    );
    let t = stop_tombstone(&dir);
    assert_eq!(
        (t["erased"].as_u64(), t["label"].as_u64()),
        (Some(0), Some(0)),
        "{t}"
    );
}

#[test]
fn precommit_says_what_fired_once_and_stays_open() {
    let dir = erased("tomb-audit-precommit");
    common::write_doc(&dir, FRAMED);
    common::git(&dir, &["add", "-A"]);
    let (stdout, line) = precommit(&dir);
    assert_eq!(stdout.matches("tombstone site").count(), 1, "{stdout}");
    assert!(stdout.contains(&site("README.md", 3, "bare")), "{stdout}");
    assert_eq!(line["tombstone"]["label"], 1, "{line}");
}

#[test]
fn an_unborn_head_erases_nothing_on_precommit_and_says_nothing_on_stop() {
    // the first commit's staged set has an empty before: measured, zero
    // — while `diff HEAD` cannot run, so the Stop line carries no key
    let dir = common::tmp("tomb-audit-unborn");
    common::write_doc(
        &dir,
        "--- README.md\n# Intro (no pork)\n--- ce.toml\n[guard]\nmode = \"observe\"\n",
    );
    common::git(&dir, &["init", "-q"]);
    common::git(&dir, &["add", "-A"]);
    let (stdout, line) = precommit(&dir);
    assert!(!stdout.contains("tombstone"), "{stdout}");
    assert_eq!(line["tombstone"]["erased"], 0, "{line}");
    let stop = common::stop_observe(&dir);
    assert!(stop.get("tombstone").is_none(), "{stop}");
}
