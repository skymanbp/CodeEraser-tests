//! The PreToolUse leg of the tombstone class through the real hook
//! (plan v2.26 step 5, judged over the wire since v2.27 step 4): the
//! spec's positive fixtures must leave a `tombstone` feed line whose
//! judgment names their site, the negative ones must leave no site —
//! and where nothing was erased, no line at all. The hook writes
//! nothing to disk, so each case seeds the BEFORE text on disk and
//! sends the AFTER text as the Write payload. The session carry-over,
//! the revival and the declared tier live in tombstone_guard_session.rs.

use crate::common;
use std::path::{Path, PathBuf};

pub(crate) fn seed(name: &str) -> PathBuf {
    let dir = common::tmp(name);
    common::declare(&dir, "[guard]\nmode = \"observe\"\n");
    dir
}

/// Seed `before` at `rel` (None = a brand-new file), send `after` as a
/// Write, and hand back the tombstone line the hook left — or None
/// when the last line is still the probe's (nothing erased, nothing
/// bound).
pub(crate) fn written(
    dir: &Path,
    rel: &str,
    before: Option<&str>,
    after: &str,
) -> Option<serde_json::Value> {
    if let Some(text) = before {
        std::fs::write(dir.join(rel), text).expect("before");
    }
    let env = common::pretooluse_envelope_at(dir, rel, "Write", after);
    let out = common::run_hook(dir, &["probe", "--hook"], &env);
    assert!(out.trim().is_empty(), "observe stays silent: {out}");
    let line = common::last_observe(dir);
    (line["event"] == "tombstone").then_some(line)
}

/// A feed site spelled from its parts (the feed's `file:line kind`):
/// written as a literal, that shape reads as a citation of the page
/// to the source-citation gate, and these are fixtures, not claims.
pub(crate) fn site(file: &str, line: usize, kind: &str) -> String {
    format!("{file}:{line} {kind}")
}

/// The sites the core seated, as the feed's `judged` object names them.
pub(crate) fn sites(line: &serde_json::Value) -> Vec<String> {
    line["judged"]["sites"]
        .as_array()
        .unwrap_or_else(|| panic!("a judged object with sites: {line}"))
        .iter()
        .map(|s| s.as_str().expect("site").to_string())
        .collect()
}

/// One positive fixture: the site the hook must name, and the counts.
fn fires(dir: &Path, rel: &str, before: &str, after: &str, site: &str) -> serde_json::Value {
    let line = written(dir, rel, Some(before), after).expect("a tombstone line");
    assert_eq!(sites(&line), [site], "{line}");
    assert!(line["erased"].as_u64().expect("erased") > 0);
    assert_eq!(line["mode"], "observe");
    assert_eq!(line["judged"]["over"], false, "no budget declared: {line}");
    assert_eq!(
        line["applied"], true,
        "observe lets the write through: {line}"
    );
    line
}

/// One negative fixture: whatever the hook wrote seats no site.
fn silent(dir: &Path, rel: &str, before: &str, after: &str) {
    if let Some(line) = written(dir, rel, Some(before), after) {
        assert!(sites(&line).is_empty(), "{rel}: {line}");
    }
}

#[test]
fn a_heading_that_frames_the_erased_name_is_a_bracketed_site() {
    let dir = seed("tomb-guard-heading");
    // ① English, ⑥ Chinese: the same frame in both scripts
    let en = fires(
        &dir,
        "r.md",
        "# Dongpo Pork\n\nBraise.\n",
        "# Tomato and Egg (no Dongpo Pork)\n\nStir.\n",
        &site("r.md", 1, "bracketed"),
    );
    assert_eq!(en["judged"]["label"], 1, "{en}");
    assert_eq!(en["judged"]["prose"], 0, "{en}");
    fires(
        &dir,
        "zh.md",
        "# 东坡肉\n\n红烧。\n",
        "# 番茄炒蛋（无东坡肉）\n\n翻炒。\n",
        &site("zh.md", 1, "bracketed"),
    );
}

#[test]
fn a_docstring_with_a_mark_and_the_name_is_a_prose_site() {
    // ② the conjunction: retrospective mark AND the erased name
    let dir = seed("tomb-guard-docstring");
    let after =
        "def cook():\n    \"\"\"This recipe no longer braises braise_pork.\"\"\"\n    return 2\n";
    let line = fires(
        &dir,
        "k.py",
        "def braise_pork():\n    return 1\n",
        after,
        "k.py:2 prose",
    );
    assert_eq!(line["judged"]["prose"], 1);
}

#[test]
fn an_identifier_that_frames_the_erased_name_is_a_bare_site() {
    // ③ a declared unit's own name is a naming surface
    let dir = seed("tomb-guard-ident");
    fires(
        &dir,
        "k.rs",
        "struct DongpoPork;\n",
        "fn cook_without_dongpo() {}\n",
        "k.rs:1 bare",
    );
}

#[test]
fn nothing_erased_means_no_line_at_all() {
    // ⑦ a heading with an absence word the file never named
    let dir = seed("tomb-guard-nothing");
    assert!(
        written(
            &dir,
            "r.md",
            Some("# Intro\n"),
            "# Intro\n\n## no_std support\n"
        )
        .is_none()
    );
    // ⑭ a keyword going away is not a name going away
    assert!(
        written(
            &dir,
            "k.rs",
            Some("fn f() {\n    while true {}\n}\n"),
            "fn f() {\n    loop {}\n}\n"
        )
        .is_none()
    );
}

#[test]
fn a_mark_alone_a_mention_alone_or_no_frame_names_no_site() {
    let dir = seed("tomb-guard-silent");
    // ⑧ the mark is there but the name survives on a structural line
    silent(
        &dir,
        "c.rs",
        "fn cache() {}\n",
        "/// no longer needs the cache warmup\nfn cache() {}\n",
    );
    // ⑨ a rename whose new heading only spells the new name
    silent(
        &dir,
        "w.rs",
        "fn old_way() {}\n",
        "/// # New way\nfn new_way() {}\n",
    );
    // ⑩ the erased thing named in a heading WITHOUT an absence frame
    silent(&dir, "m.md", "# Dongpo\n", "# Migration from Dongpo\n");
    // ⑬ a deleted comment's word is prose, never a name
    silent(
        &dir,
        "d.md",
        "Zero downtime deploys.\n\n# Deploy\n",
        "# Deploy (without downtime)\n",
    );
    // ⑯ a frame inside a fenced example is an example
    silent(
        &dir,
        "f.md",
        "# Dongpo Pork\n",
        "# Menu\n\n```\n# 番茄炒蛋（无东坡肉）\n(no Dongpo Pork)\n```\n",
    );
    // ⑰ a pure deletion has no naming surface
    silent(&dir, "e.md", "# Dongpo Pork\n", "");
}

#[test]
fn an_absence_word_is_never_a_name_even_as_a_compound() {
    // ⑱ `NotFound` is V₀ whole; its `found` half must not enter R
    let dir = seed("tomb-guard-negation");
    silent(
        &dir,
        "n.rs",
        "struct NotFound;\n",
        "fn not_found_handler() {}\n",
    );
}

#[test]
fn the_allow_pragma_is_deliberately_unwired() {
    // ⑲ no per-site opt-out: the exemption channels are the three
    // witnesses and `[tombstone] ledger`, all of them ledgered
    let dir = seed("tomb-guard-pragma");
    fires(
        &dir,
        "p.md",
        "# Dongpo Pork\n",
        "<!-- ce:allow(tombstone) -->\n# Tomato (no Dongpo Pork)\n",
        &site("p.md", 2, "bracketed"),
    );
}
