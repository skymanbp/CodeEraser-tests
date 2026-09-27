//! The PostToolUse leg through the real hook (plan v2.30 step 5b, the
//! 2026-09-27 ruling): a write the guard answered `ask` for, and the
//! tool then ran, leaves one `settled` line under the same
//! `tool_use_id`; nothing else does — a second PostToolUse under that
//! id, an id the session never asked about, a write decided at
//! observe, another tool under an asked id, the PreToolUse envelope
//! itself. And the tombstone union reads an asked erasure only through
//! that line: the same `(no X)` frame binds nothing before the
//! settlement and binds after it — the tree lacks the name both times,
//! so the feed's record, not the disk, is what decides.

use crate::common;
use crate::tombstone_guard::{site, sites, written};
use std::path::{Path, PathBuf};

/// A project whose zone map is armed: a 700-line write lands at 888‰
/// of the soft→hard zone and is asked about (guard_hook's fixture), a
/// 400-line one at 222‰ is observed.
fn armed(name: &str) -> PathBuf {
    let dir = common::tmp(name);
    common::declare(&dir, "[guard]\nmode = \"observe\"\nzone_tiers = true\n");
    dir
}

fn filler(lines: usize) -> String {
    "// filler\n".repeat(lines)
}

/// A 700-line write to `rel` under `id`, asked about (888‰ of the
/// armed zone); the envelope, for a leg that wants it again.
fn ask(dir: &Path, rel: &str, id: &str) -> String {
    let env = common::pretooluse_envelope_id(dir, rel, "Write", &filler(700), id);
    common::expect_decision(dir, &env, "ask");
    env
}

/// `ce settle --hook` on a PostToolUse envelope: silent always, and
/// the line it appended when it appended one.
fn settle(dir: &Path, rel: &str, tool: &str, id: &str) -> Option<serde_json::Value> {
    let before = common::observe_lines(dir).len();
    let env = common::posttooluse_envelope(dir, rel, tool, id);
    let out = common::run_hook(dir, &["settle", "--hook"], &env);
    assert!(out.trim().is_empty(), "the leg never speaks: {out}");
    let mut lines = common::observe_lines(dir);
    assert!(lines.len() <= before + 1, "at most one line per settlement");
    (lines.len() > before).then(|| lines.pop().expect("the appended line"))
}

#[test]
fn an_asked_write_that_ran_is_settled_once() {
    let dir = armed("settle-asked");
    ask(&dir, "b.rs", "toolu_asked");
    let probe = common::observe_lines(&dir)
        .into_iter()
        .find(|v| v["event"] == "probe")
        .expect("the probe line");
    assert_eq!(
        (probe["decision"].as_str(), probe["tool_use_id"].as_str()),
        (Some("ask"), Some("toolu_asked")),
        "{probe}"
    );
    let line = settle(&dir, "b.rs", "Write", "toolu_asked").expect("settled");
    assert_eq!(
        (
            line["event"].as_str(),
            line["tool_use_id"].as_str(),
            line["session_id"].as_str()
        ),
        (Some("settled"), Some("toolu_asked"), Some("t")),
        "{line}"
    );
    assert!(
        line["file"].as_str().expect("file").ends_with("b.rs"),
        "{line}"
    );
    // once: the harness re-firing under the settled id adds no second line
    assert!(settle(&dir, "b.rs", "Write", "toolu_asked").is_none());
}

#[test]
fn nothing_else_is_settled() {
    let dir = armed("settle-others");
    // an id the session never asked about; a write decided at observe
    // (222‰: a feed line, no decision); another tool under an asked id
    let low = common::pretooluse_envelope_id(&dir, "b.rs", "Write", &filler(400), "toolu_low");
    assert!(
        common::run_hook(&dir, &["probe", "--hook"], &low)
            .trim()
            .is_empty()
    );
    let asked = ask(&dir, "b.rs", "toolu_two");
    for (tool, id) in [
        ("Write", "toolu_other"),
        ("Write", "toolu_low"),
        ("Bash", "toolu_two"),
    ] {
        assert!(
            settle(&dir, "b.rs", tool, id).is_none(),
            "{tool} under {id}"
        );
    }
    // and the PreToolUse envelope itself is not this leg's event
    let before = common::observe_lines(&dir).len();
    assert!(
        common::run_hook(&dir, &["settle", "--hook"], &asked)
            .trim()
            .is_empty()
    );
    assert_eq!(
        common::observe_lines(&dir).len(),
        before,
        "not this leg's event"
    );
}

#[test]
fn an_asked_erasure_enters_the_union_only_once_settled() {
    let dir = armed("settle-union");
    std::fs::write(dir.join("a.rs"), "fn dongpo() {}\n").expect("before");
    ask(&dir, "a.rs", "toolu_erase");
    std::fs::write(dir.join("a.rs"), filler(700)).expect("the person lets it land");
    let tomb = common::observe_lines(&dir)
        .into_iter()
        .rfind(|v| v["event"] == "tombstone")
        .expect("a tombstone line");
    assert!(tomb["applied"].is_null() && tomb["erased"] == 1, "{tomb}");
    let frame = || {
        written(
            &dir,
            "r.md",
            Some("# Menu\n"),
            "# Menu\n\n## Sides (no dongpo)\n",
        )
    };
    assert!(
        frame().is_none(),
        "unsettled: the union does not know the write landed"
    );
    settle(&dir, "a.rs", "Write", "toolu_erase").expect("settled");
    let hit = frame().expect("bound");
    assert_eq!(sites(&hit), [site("r.md", 3, "bracketed")], "{hit}");
    assert_eq!(hit["session_erased"], 1, "{hit}");
}
