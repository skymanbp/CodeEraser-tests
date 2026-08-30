use super::*;

/// The intake is bounded: an envelope under the cap still parses,
/// one over it takes the fail-open None instead of being
/// materialized whole. Without the cap the over-cap case parses
/// fine and this asserts nothing — that WAS the hole.
#[test]
fn envelope_intake_is_capped() {
    let small: Option<serde_json::Value> = parse_envelope(&br#"{"pad":"x"}"#[..]);
    assert_eq!(small.expect("under cap parses")["pad"], "x");
    let huge = format!(r#"{{"pad":"{}"}}"#, "x".repeat(ENVELOPE_CAP as usize));
    let over: Option<serde_json::Value> = parse_envelope(huge.as_bytes());
    assert!(over.is_none(), "over-cap envelope must fail open");
}

/// B4 acceptance half 1: the clip is identity under budget, caps
/// at budget with the on-disk pointer over it, and never splits a
/// multi-byte char (the marker itself starts with one). The tail is
/// asserted through `clip_mark`, not as a literal: the marker is a
/// sentence a person reads, so it answers CE_LANG, and the language
/// is pinned once per PROCESS (i18n's OnceLock) — a literal here
/// would turn an operator's ambient `CE_LANG=zh` into a red test.
/// Both halves of the marker are asked from the binary in guard_hook.
#[test]
fn clip_caps_at_budget_and_respects_char_boundaries() {
    assert_eq!(clip("short", WARN_BUDGET_TOKENS), "short");
    let long = "预算".repeat(400); // 800 chars, 2400 bytes
    let clipped = clip(&long, WARN_BUDGET_TOKENS);
    assert!(clipped.len() <= WARN_BUDGET_TOKENS * CHARS_PER_TOKEN);
    assert!(clipped.ends_with(clip_mark()), "{clipped}");
    assert!(clip_mark().contains("observe.ndjson"), "points at the feed");
    assert!(clipped.chars().count() > 0); // boundary-safe slice
}

/// B4 acceptance half 2: one warn per (rule, file) per session —
/// a clean probe line never counts, a fired one does, and other
/// files, sessions and rules stay unsuppressed.
#[test]
fn already_warned_is_per_session_rule_and_file() {
    let dir = std::env::temp_dir().join(format!("ce-b4-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let line = |m: u64| serde_json::json!({"event": "probe", "file": "a.rs", "matches": m});
    observe_append(&dir, Some("s1"), line(0));
    assert!(!already_warned(&dir, "s1", "probe", "a.rs"), "clean probe");
    observe_append(&dir, Some("s1"), line(2));
    assert!(already_warned(&dir, "s1", "probe", "a.rs"), "fired probe");
    assert!(
        !already_warned(&dir, "s2", "probe", "a.rs"),
        "other session"
    );
    assert!(!already_warned(&dir, "s1", "probe", "b.rs"), "other file");
    assert!(!already_warned(&dir, "s1", "budget", "a.rs"), "other rule");
    observe_append(
        &dir,
        Some("s1"),
        serde_json::json!({"event": "budget", "file": "a.rs"}),
    );
    assert!(already_warned(&dir, "s1", "budget", "a.rs"), "budget fired");
    assert!(!already_warned(&dir, "", "probe", "a.rs"), "no session");
    std::fs::remove_dir_all(&dir).ok();
}
