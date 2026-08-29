//! A fixture's `_why` names the gate that reads it. When the suite
//! moved into the cli/tests submodule (plan v2.18), gui-lens.json kept
//! saying `gui/tests/lens_invariant.js` and nothing noticed — the field
//! is prose to every parser. Ruling (2026-08-28, user): guard the
//! fixtures' `_why` only, not the tree's prose at large — every
//! repo-relative path a `_why` spells must exist. Anti-vacuity: at
//! least one such path is spelled somewhere, else the parser went quiet.

use crate::common;

const EXTS: [&str; 8] = ["js", "rs", "hs", "json", "md", "toml", "ts", "py"];

/// The tokens of a `_why` that read as a repo-relative path: a `/`
/// inside and a source extension at the end, punctuation shed.
fn paths(why: &str) -> Vec<&str> {
    why.split(|c: char| c.is_whitespace() || "()[]{},;:\"'`".contains(c))
        .filter(|t| t.contains('/') && !t.starts_with('/'))
        .filter(|t| {
            t.rsplit_once('.')
                .is_some_and(|(_, ext)| EXTS.contains(&ext))
        })
        .collect()
}

#[test]
fn every_path_a_fixture_why_spells_exists() {
    let root = common::repo_root();
    let mut files = Vec::new();
    common::files_with_ext(&root.join("contracts/fixtures"), "json", &mut files);
    let (mut seen, mut missing) = (0usize, Vec::new());
    for file in &files {
        let text = std::fs::read_to_string(file).expect("fixture");
        let doc: serde_json::Value = serde_json::from_str(&text).expect("fixture json");
        let Some(why) = doc.get("_why").and_then(|w| w.as_str()) else {
            continue;
        };
        for rel in paths(why) {
            seen += 1;
            if !root.join(rel).exists() {
                missing.push(format!(
                    "{}: {rel}",
                    file.strip_prefix(&root).expect("under root").display()
                ));
            }
        }
    }
    assert!(
        seen >= 1,
        "no fixture _why spells a path: the parser went quiet"
    );
    assert!(
        missing.is_empty(),
        "a fixture's _why names a path that is not there:\n{}",
        missing.join("\n")
    );
}

#[test]
fn the_path_reading_is_narrow() {
    let why = "shaped for cli/tests/gui/lens_invariant.js (see docs/reference/gui.md); not a/b, not x.js, not /abs/p.rs";
    assert_eq!(
        paths(why),
        ["cli/tests/gui/lens_invariant.js", "docs/reference/gui.md"]
    );
}
