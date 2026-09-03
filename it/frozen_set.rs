//! The frozen set (plan v2.21 ⑦): CHANGELOG.md with its archive, the
//! EVAL-SET records and FIELD-TEST.md are history — every number on them was a fact on
//! its day, and a writer that "fixed" one would rewrite the record.
//! No generator scans or writes them: no chip, no generated block, no
//! chip enrolment, and an explicit opt-out from the citation gate,
//! whose opt-out file names nothing outside this set. A page joins the
//! set here, by name.

use crate::common::repo_root;
use crate::facts::{chip, read};
use std::collections::BTreeMap;

pub const FROZEN: &[&str] = &[
    "CHANGELOG.md",
    "docs/CHANGELOG-ARCHIVE.md",
    "docs/EVAL-SET.md",
    "docs/EVAL-SET-M5-3.md",
    "docs/EVAL-SET-M5-CLOSE.md",
    "docs/FIELD-TEST.md",
];

/// A marker spelled on a frozen page may only DESCRIBE one (inside a
/// code span, so preceded by a backtick) — never open one.
fn describes_only(rel: &str, text: &str, marker: &str) {
    for (i, _) in text.match_indices(marker) {
        assert!(
            text[..i].ends_with('`'),
            "{rel}: `{marker}` on a frozen page outside a code span (byte {i})"
        );
    }
}

#[test]
fn the_frozen_pages_are_never_scanned_or_written() {
    let root = repo_root();
    let optout: BTreeMap<String, String> =
        serde_json::from_str(&read(&root, "contracts/docs-citations-optout.json"))
            .expect("parse citation opt-outs");
    let chipped = crate::facts_chips::surfaces();
    for rel in FROZEN {
        let text = read(&root, rel);
        describes_only(rel, &text, chip::OPEN);
        describes_only(rel, &text, ":begin -->");
        assert!(
            optout.contains_key(*rel),
            "{rel}: frozen but not opted out of the citation gate"
        );
        assert!(
            !chipped.iter().any(|(s, ..)| s == rel),
            "{rel}: frozen but enrolled as a chip surface"
        );
    }
    for page in optout.keys() {
        assert!(
            FROZEN.contains(&page.as_str()),
            "{page}: opted out of the citation gate but not frozen — name it here or take the opt-out back"
        );
    }
}
