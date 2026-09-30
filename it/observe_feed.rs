//! Observe-feed contract (ce.observe/0.12.0): the NDJSON feed is the
//! M4 evaluation-set raw material, so its line shape is pinned by a
//! golden. One deterministic run of every producer — probe, budget
//! (§4.2 step 2), zone unarmed AND armed (plan v2.6 §A / v2.7 ①), the
//! armed map's `ask` and the PostToolUse leg's settlement of it (0.11.0),
//! tombstone (plan v2.26, the per-edit leg; the Stop and precommit
//! lines carry its object), flow (0.12.0, the per-edit leg ahead of its
//! probe line; the audit lines carry its object), stop audit (whose `similar` object, 0.10.0,
//! is ABSENT here by design: the staged twin shares one name word and
//! the core's role bit wants two — similar_face.rs seeds the pair that
//! earns it), precommit, commitmsg — volatile fields
//! normalized (ts_ms,
//! elapsed_ms, absolute file path); key sets, schema/event tags,
//! counts, mode, and degraded flags must match byte-for-byte.
//! No write of this run lands on disk, so since plan v2.30 step 5b the
//! tombstone lines' `session_erased` reads 0 — the union drops a key its
//! erasing file still declares (the session battery carries the positive
//! path). Bless flow: `CE_BLESS=1 cargo test --test it -- observe_feed::`.
//!
//! `session_id` is deliberately NOT normalized: the hook envelopes
//! carry the literal "t", so the golden pins that the id survives the
//! whole path — and that the git-hook faces (precommit, commitmsg),
//! which are not hooks of the session and own none, record null
//! instead of borrowing one.

use crate::common;
use std::path::Path;

/// The whole feed, volatile fields zeroed, as pretty JSON.
fn normalized_feed(dir: &Path) -> String {
    let log = std::fs::read_to_string(dir.join(".ce/observe.ndjson")).expect("feed");
    let entries: Vec<serde_json::Value> = log
        .lines()
        .map(|l| {
            let mut j: serde_json::Value = serde_json::from_str(l).expect("entry json");
            j["ts_ms"] = serde_json::json!(0);
            if j.get("elapsed_ms").is_some() {
                j["elapsed_ms"] = serde_json::json!(0);
            }
            if j.get("file").is_some() {
                j["file"] = serde_json::json!("<file>");
            }
            j
        })
        .collect();
    serde_json::to_string_pretty(&entries).expect("serialize")
}

/// A docstring with a mark and a `without_` unit, over a file that
/// declared `work_1`.
const TOMB: &str = "/// This file no longer needs work_1.\nfn without_work() {}\n";

/// Entries 8-10 (0.11.0): the armed map ASKS about a 700-line write
/// (888‰) — the probe line, written once the event is decided, records
/// `decision` ask under the call's `tool_use_id` and the zone line
/// follows it (the tombstone line of that rewrite reads `applied`
/// null) — and the PostToolUse leg's `settled` line once the tool ran
/// under that id.
fn asked_write_settles(dir: &Path) {
    let asked = "// filler\n".repeat(700);
    let env = common::pretooluse_envelope_id(dir, "b.rs", "Write", &asked, "toolu_ask");
    common::expect_decision(dir, &env, "ask");
    let ran = common::posttooluse_envelope(dir, "b.rs", "Write", "toolu_ask");
    common::run_hook(dir, &["settle", "--hook"], &ran);
}

/// Entry 15: the stop audit (staged b.rs = one touched duplicate);
/// entry 16: precommit (observe mode reports but exits 0); entry 17:
/// commitmsg — the staged set now erases a.rs's `work_1` and the
/// message argues it away: precommit's line shape under its own event,
/// the message's own site, session null.
fn stop_and_git_faces(dir: &Path) {
    common::run_hook(
        dir,
        &["audit", "--hook"],
        &common::stop_envelope(dir, false),
    );
    assert!(common::run_ce(dir, &["precommit"]).status.success());
    common::git(dir, &["rm", "-q", "a.rs"]);
    std::fs::write(
        dir.join(".git/COMMIT_EDITMSG"),
        "Drop a.rs\n\nwork_1 is no longer needed.\n",
    )
    .expect("message");
    assert!(
        common::run_ce(dir, &["commitmsg", ".git/COMMIT_EDITMSG"])
            .status
            .success()
    );
}

#[test]
fn feed_shape_matches_golden() {
    let dir = common::tmp("observe-golden");
    common::seed_git_clone_repo(&dir, "observe");
    common::build_index(&dir);
    // entry 1: probe (T2 rewrite of indexed content -> matches)
    let env = common::pretooluse_envelope(&dir, "Write", &common::rust_fn(3));
    common::run_hook(&dir, &["probe", "--hook"], &env);
    // a filler write of `lines` lines through the probe; each one
    // below leaves the lines its comment names
    let filler = |lines: usize| {
        let env = common::pretooluse_envelope(&dir, "Write", &"// filler\n".repeat(lines));
        common::run_hook(&dir, &["probe", "--hook"], &env);
    };
    // entries 2+3: an over-cap write logs a no-match probe line plus
    // the budget event (0.4.0) — in every tier, observe included
    filler(751);
    // entries 4+5: an IN-ZONE write (400 lines, soft fallback 300,
    // hard 750 -> position 222‰) logs the 0.5.0 zone event — feed
    // only, no enforcement; the producer must ride this golden run
    // or it ships untested (the v0.6 map's own warning)
    filler(400);
    // entries 6+7: the ARMED map (v2.7 ①): ce.toml opts in (keeping
    // the seeded observe mode — the zone's tier is its own, not the
    // class mode) and the same producer's zone line now carries the
    // mapped zone_tier (0.6.0) — 666‰ -> warn, recorded in the
    // feed, decided on stdout at the zone's OWN tier
    std::fs::write(
        dir.join("ce.toml"),
        "[guard]\nmode = \"observe\"\nzone_tiers = true\n",
    )
    .expect("ce.toml");
    filler(600);
    // entries 8-10 (0.11.0): an asked write and its settlement
    asked_write_settles(&dir);
    // entries 11+12: a Write erasing `work_1` and writing it back as an
    // absence logs the 0.8.0 tombstone line after its own probe
    let tomb = common::pretooluse_envelope_at(&dir, "a.rs", "Write", TOMB);
    common::run_hook(&dir, &["probe", "--hook"], &tomb);
    // entries 13+14 (0.12.0): a new file bringing an unreachable
    // statement logs the flow line ahead of its own probe line
    let dead = "fn gone() -> i32 {\n    return 1;\n    let _late = 2;\n}\n";
    let flow = common::pretooluse_envelope_at(&dir, "c.rs", "Write", dead);
    common::run_hook(&dir, &["probe", "--hook"], &flow);
    // entries 15-17: the stop audit and the two git-hook faces
    stop_and_git_faces(&dir);
    common::assert_matches_golden(
        &normalized_feed(&dir),
        &common::golden_path("observe-feed/feed.golden.json"),
    );
}
