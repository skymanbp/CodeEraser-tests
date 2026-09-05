//! The PreToolUse leg across a SESSION and at its declared tier (split
//! from tombstone_guard.rs at the 300-line wall): a name erased edits
//! ago still binds through the feed's union; a name written back leaves
//! it (a revival, codex review 2026-09-04); the class speaks at its own
//! tier past its own budget and records the decision as `applied` — a
//! denied erasure never entered the union; `observe` stays silent.

use crate::common;
use crate::tombstone_guard::{seed, site, sites, written};
use std::path::Path;

#[test]
fn a_name_erased_edits_ago_still_binds_through_the_session() {
    // ⑤ edit 1 erases `dongpo`; two unrelated edits pass; edit 4 frames it
    let dir = seed("tomb-guard-session");
    let first = written(&dir, "a.rs", Some("fn dongpo() {}\n"), "fn other() {}\n").expect("erased");
    assert!(sites(&first).is_empty() && first["session_erased"] == 0);
    for n in 0..2 {
        assert!(
            written(
                &dir,
                "z.md",
                Some("# Notes\n"),
                &format!("# Notes {n}\n\nmore\n")
            )
            .is_none()
        );
    }
    let hit = written(
        &dir,
        "r.md",
        Some("# Menu\n"),
        "# Menu\n\n## Sides (no dongpo)\n",
    )
    .expect("bound");
    assert_eq!(sites(&hit), [site("r.md", 3, "bracketed")], "{hit}");
    assert!(hit["session_erased"].as_u64().expect("union") >= 1);
    assert_eq!(hit["erased"], 0, "this edit erased nothing itself");
}

#[test]
fn a_name_written_back_leaves_the_session_union() {
    // codex review 2026-09-04: edit 1 erases `dongpo`; edit 2 declares
    // it again — a revival the line records — and edit 3's frame then
    // binds nothing: the union subtracts what a later edit brought back
    let dir = seed("tomb-guard-revive");
    let first = written(&dir, "a.rs", Some("fn dongpo() {}\n"), "fn other() {}\n").expect("erased");
    let key = first["erased_hashes"][0].as_u64().expect("one key");
    let back = written(
        &dir,
        "b.rs",
        Some("fn b() {}\n"),
        "fn b() {}\nfn dongpo() {}\n",
    )
    .expect("a revival is a line");
    assert_eq!(back["revived_hashes"], serde_json::json!([key]), "{back}");
    assert_eq!(back["erased"], 0, "{back}");
    assert!(
        written(
            &dir,
            "r.md",
            Some("# Menu\n"),
            "# Menu\n\n## Sides (no dongpo)\n"
        )
        .is_none(),
        "the union forgot the name"
    );
}

/// The last `tombstone` event in the feed, wherever the decision's own
/// lines landed after it.
fn last_tombstone(dir: &Path) -> serde_json::Value {
    let feed = std::fs::read_to_string(dir.join(".ce/observe.ndjson")).expect("feed");
    feed.lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .rfind(|v| v["event"] == "tombstone")
        .expect("a tombstone line")
}

#[test]
fn the_declared_tier_and_budget_decide_and_observe_stays_silent() {
    // plan v2.27 step 4: the class speaks at ITS tier when the core
    // says the declared budget is exceeded — deny refuses the write
    // with the class's sentence and the first site; the route default
    // records the same judgment and says nothing
    let dir = common::tmp("tomb-guard-tier");
    let toml =
        |tier: &str| format!("[guard]\nmode = \"observe\"\n\n[tombstone]\n{tier}budget = 0\n");
    common::declare(&dir, &toml("tier = \"deny\"\n"));
    std::fs::write(dir.join("r.md"), "# Dongpo Pork\n").expect("before");
    let env = common::pretooluse_envelope_at(&dir, "r.md", "Write", "# Tomato (no Dongpo Pork)\n");
    let reason = common::expect_decision(&dir, &env, "deny");
    assert!(
        reason.contains("[tombstone] budget") && reason.contains(&site("r.md", 1, "bracketed")),
        "{reason}"
    );
    let line = last_tombstone(&dir);
    assert_eq!(
        (line["mode"].as_str(), line["judged"]["over"].as_bool()),
        (Some("deny"), Some(true)),
        "{line}"
    );
    // the line waited for the decision: a denied write erased nothing,
    // so the next edit's frame binds nothing (codex review 2026-09-04)
    assert_eq!(line["applied"], false, "{line}");
    assert!(
        written(
            &dir,
            "z.md",
            Some("# Menu\n"),
            "# Menu\n\n## Sides (no dongpo)\n"
        )
        .is_none(),
        "a denied erasure never entered the union"
    );
    common::declare(&dir, &toml(""));
    let line = written(
        &dir,
        "r.md",
        Some("# Dongpo Pork\n"),
        "# Tomato (no Dongpo Pork)\n",
    )
    .expect("a tombstone line");
    assert_eq!(
        (line["mode"].as_str(), line["judged"]["over"].as_bool()),
        (Some("observe"), Some(true)),
        "{line}"
    );
}
