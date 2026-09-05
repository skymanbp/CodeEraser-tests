//! Observe-feed contract (ce.observe/0.10.0): the NDJSON feed is the
//! M4 evaluation-set raw material, so its line shape is pinned by a
//! golden. One deterministic run of every producer — probe, budget
//! (§4.2 step 2), zone unarmed AND armed (plan v2.6 §A / v2.7 ①),
//! tombstone (plan v2.26, the per-edit leg; the Stop and precommit
//! lines carry its object), stop audit (whose `similar` object, 0.10.0,
//! is ABSENT here by design: the staged twin shares one name word and
//! the core's role bit wants two — similar_face.rs seeds the pair that
//! earns it), precommit, commitmsg — volatile fields
//! normalized (ts_ms,
//! elapsed_ms, absolute file path); key sets, schema/event tags,
//! counts, mode, and degraded flags must match byte-for-byte.
//! Bless flow: `CE_BLESS=1 cargo test --test it -- observe_feed::`.
//!
//! `session_id` is deliberately NOT normalized: the hook envelopes
//! carry the literal "t", so the golden pins that the id survives the
//! whole path — and that the git-hook faces (precommit, commitmsg),
//! which are not hooks of the session and own none, record null
//! instead of borrowing one.

use crate::common;
/// The whole feed, volatile fields zeroed, as pretty JSON.
fn normalized_feed(dir: &std::path::Path) -> String {
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

#[test]
fn feed_shape_matches_golden() {
    let dir = common::tmp("observe-golden");
    common::seed_git_clone_repo(&dir, "observe");
    common::build_index(&dir);
    // entry 1: probe (T2 rewrite of indexed content -> matches)
    let env = common::pretooluse_envelope(&dir, "Write", &common::rust_fn(3));
    common::run_hook(&dir, &["probe", "--hook"], &env);
    // entries 2+3: an over-cap write logs a no-match probe line plus
    // the budget event (0.4.0) — in every tier, observe included
    let big = "// filler\n".repeat(751);
    let env2 = common::pretooluse_envelope(&dir, "Write", &big);
    common::run_hook(&dir, &["probe", "--hook"], &env2);
    // entries 4+5: an IN-ZONE write (400 lines, soft fallback 300,
    // hard 750 -> position 222‰) logs the 0.5.0 zone event — feed
    // only, no enforcement; the producer must ride this golden run
    // or it ships untested (the v0.6 map's own warning)
    let mid = "// filler\n".repeat(400);
    let env3 = common::pretooluse_envelope(&dir, "Write", &mid);
    common::run_hook(&dir, &["probe", "--hook"], &env3);
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
    let deep = "// filler\n".repeat(600);
    let env4 = common::pretooluse_envelope(&dir, "Write", &deep);
    common::run_hook(&dir, &["probe", "--hook"], &env4);
    // entries 8+9: a Write erasing `work_1` and writing it back as an
    // absence logs the 0.8.0 tombstone line after its own probe
    let tomb = common::pretooluse_envelope_at(&dir, "a.rs", "Write", TOMB);
    common::run_hook(&dir, &["probe", "--hook"], &tomb);
    // entry 10: stop audit (staged b.rs = one touched duplicate)
    common::run_hook(
        &dir,
        &["audit", "--hook"],
        &common::stop_envelope(&dir, false),
    );
    // entry 11: precommit (observe mode reports but exits 0)
    assert!(common::run_ce(&dir, &["precommit"]).status.success());
    // entry 12: commitmsg — the staged set now erases a.rs's `work_1`
    // and the message argues it away: precommit's line shape under its
    // own event, the message's own site, session null
    common::git(&dir, &["rm", "-q", "a.rs"]);
    std::fs::write(
        dir.join(".git/COMMIT_EDITMSG"),
        "Drop a.rs\n\nwork_1 is no longer needed.\n",
    )
    .expect("message");
    assert!(
        common::run_ce(&dir, &["commitmsg", ".git/COMMIT_EDITMSG"])
            .status
            .success()
    );
    common::assert_matches_golden(
        &normalized_feed(&dir),
        &common::golden_path("observe-feed/feed.golden.json"),
    );
}
