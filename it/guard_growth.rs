//! The PreToolUse size classes judge what a write ADDS (v2.35 guard
//! lane, GD1): the hard budget refuses a write that takes a file
//! across its line or grows a file already past it, and records — never
//! refuses — one that keeps or shrinks it, so "split the file" can be
//! done one step at a time; the graded zone's tier likewise fires only
//! on a growing write. Through the real binary, every case reading the
//! feed line the hook left beside its decision.

use crate::common::{self, last_observe, run_hook, tmp, write_all};
use serde_json::Value;
use std::path::Path;

fn filler(n: usize) -> String {
    "// filler\n".repeat(n)
}

/// A Write of `n` filler lines to `rel`.
fn write_of(dir: &Path, rel: &str, n: usize) -> String {
    common::pretooluse_envelope_at(dir, rel, "Write", &filler(n))
}

/// A write the hook must let through in silence, and the last feed line
/// of `event` it left.
fn silent(dir: &Path, envelope: &str, event: &str) -> Value {
    let out = run_hook(dir, &["probe", "--hook"], envelope);
    assert!(out.trim().is_empty(), "{event}: a silent write, got {out}");
    common::observe_lines(dir)
        .into_iter()
        .rev()
        .find(|l| l["event"] == event)
        .unwrap_or_else(|| panic!("no {event} line"))
}

/// Past the line, a write that keeps the size (a rewrite with its own
/// content, a one-line Edit) or shrinks it (an Edit deleting lines) is
/// silent and leaves `carried: true` with both sizes and no kind; one
/// more line is a growth, refused at the class tier with both numbers
/// named.
#[test]
fn a_file_past_the_line_may_keep_or_shrink_but_not_grow() {
    let dir = tmp("guard-growth-carried");
    let body = format!("// head\n{}", filler(799));
    write_all(&dir, &[("b.rs", &body)]);
    let head = format!("// head\n{}", filler(20));
    let cases = [
        (
            common::pretooluse_envelope_at(&dir, "b.rs", "Write", &body),
            800,
        ),
        (
            common::pretooluse_edit_envelope(&dir, "b.rs", "// head\n", "// kept\n"),
            800,
        ),
        (
            common::pretooluse_edit_envelope(&dir, "b.rs", &head, ""),
            779,
        ),
    ];
    for (envelope, after) in cases {
        let line = silent(&dir, &envelope, "budget");
        let seen = [
            &line["carried"],
            &line["before_lines"],
            &line["resulting_lines"],
        ];
        assert_eq!(
            seen,
            [&Value::from(true), &800.into(), &after.into()],
            "{line}"
        );
        assert!(
            line.get("kind").is_none(),
            "a carried write has no kind: {line}"
        );
    }
    let reason = common::expect_decision(&dir, &write_of(&dir, "b.rs", 801), "deny");
    assert!(
        reason.contains("at 800 lines; this write grows it to 801"),
        "{reason}"
    );
    assert_eq!(last_observe(&dir)["kind"], "growth");
}

/// A target whose size before cannot be read (a directory sits at the
/// path) gets no allowance: refused as the rule stood, with today's
/// one-number sentence and a degraded stamp naming why.
#[test]
fn an_unread_size_before_is_no_allowance() {
    let dir = tmp("guard-growth-unread");
    std::fs::create_dir_all(dir.join("d.rs")).expect("a directory at the path");
    let reason = common::expect_decision(&dir, &write_of(&dir, "d.rs", 751), "deny");
    assert!(
        reason.contains("leaves") && reason.contains("at 751 lines"),
        "{reason}"
    );
    let line = last_observe(&dir);
    assert!(
        line["before_unread"].is_string()
            && line["degraded"].is_string()
            && line["before_lines"].is_null()
            && line.get("kind").is_none(),
        "{line}"
    );
}

/// The armed graded zone: a write inside it that does not grow the file
/// fires no tier (its line says `carried` and `zone_tier: observe`); a
/// carried write past the hard line leaves a budget line and no zone
/// line at all, where it used to map past 1000‰ to `ask`.
#[test]
fn the_zone_tier_fires_only_on_growth() {
    let dir = tmp("guard-growth-zone");
    common::declare(&dir, "[guard]\nzone_tiers = true\n");
    write_all(&dir, &[("z.rs", &filler(700)), ("h.rs", &filler(800))]);
    let line = silent(&dir, &write_of(&dir, "z.rs", 700), "zone");
    let seen = [&line["carried"], &line["zone_tier"], &line["before_lines"]];
    assert_eq!(seen, [&Value::from(true), &"observe".into(), &700.into()]);
    common::expect_decision(&dir, &write_of(&dir, "z.rs", 701), "ask");
    silent(&dir, &write_of(&dir, "h.rs", 800), "budget");
    let mine: Vec<_> = common::observe_lines(&dir)
        .into_iter()
        .filter(|l| l["file"].as_str().is_some_and(|f| f.ends_with("h.rs")))
        .map(|l| l["event"].as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(mine, ["probe", "budget"], "no zone line past the hard line");
}

/// At `warn` the per-session suppression reads the feed: a carried
/// budget line warned nobody, so a growth of the same file in the same
/// session still warns.
#[test]
fn a_carried_line_does_not_spend_the_warning() {
    let dir = tmp("guard-growth-warn");
    common::declare(&dir, "[guard]\nmode = \"warn\"\n");
    write_all(&dir, &[("b.rs", &filler(800))]);
    silent(&dir, &write_of(&dir, "b.rs", 800), "budget");
    let reason = common::expect_decision(&dir, &write_of(&dir, "b.rs", 801), "allow");
    assert!(reason.contains("grows it to 801"), "{reason}");
}
