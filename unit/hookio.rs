use super::*;

/// The intake is bounded: an envelope under the cap still parses,
/// one over it is a NAMED refusal (v2.35 G4: the gate writes it to the
/// feed) instead of being materialized whole, and garbage is ignored.
/// Without the cap the over-cap case parses fine and this asserts
/// nothing — that WAS the hole.
#[test]
fn envelope_intake_is_capped() {
    let small: Result<serde_json::Value, Intake> = parse_envelope(&br#"{"pad":"x"}"#[..]);
    assert_eq!(small.ok().expect("under cap parses")["pad"], "x");
    let huge = format!(r#"{{"pad":"{}"}}"#, "x".repeat(ENVELOPE_CAP as usize));
    match parse_envelope::<serde_json::Value>(huge.as_bytes()) {
        Err(Intake::Refused(why)) => assert!(why.contains("cap"), "{why}"),
        _ => panic!("over-cap envelope must be a named refusal"),
    }
    let junk = parse_envelope::<serde_json::Value>(&b"not json"[..]);
    assert!(
        matches!(junk, Err(Intake::Ignored)),
        "garbage is no hook call"
    );
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
    let dir = crate::testutil::scratch("b4");
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
