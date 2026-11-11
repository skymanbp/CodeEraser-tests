//! The PreToolUse size classes judge what a write ADDS (v2.35 guard
//! lane, GD1): the hard budget refuses a write that takes a file
//! across its line or grows a file already past it, and records — never
//! refuses — one that keeps or shrinks it, so "split the file" can be
//! done one step at a time; the graded zone's tier likewise fires only
//! on a growing write. Through the real binary, every case reading the
//! feed line the hook left beside its decision.

use crate::common::{self, last_observe, run_hook, tmp, write_all};
use std::path::Path;

fn filler(n: usize) -> String {
    "// filler\n".repeat(n)
}

/// The hook's stdout and the last feed line of the given event.
fn hook(dir: &Path, envelope: &str, event: &str) -> (String, serde_json::Value) {
    let out = run_hook(dir, &["probe", "--hook"], envelope);
    let line = common::observe_lines(dir)
        .into_iter()
        .rev()
        .find(|l| l["event"] == event)
        .unwrap_or_else(|| panic!("no {event} line"));
    (out, line)
}

/// Past the line, a write that keeps the size (a one-line Edit, a
/// rewrite with its own content) or shrinks it (an Edit deleting lines)
/// is silent and leaves `carried: true` with both sizes; one more line
/// is a growth, refused at the class tier with both numbers named.
#[test]
fn a_file_past_the_line_may_keep_or_shrink_but_not_grow() {
    let dir = tmp("guard-growth-carried");
    let body = format!("// head
{}", filler(799));
    write_all(&dir, &[("b.rs", &body)]);
    let own = common::pretooluse_envelope_at(&dir, "b.rs", "Write", &body);
    let (out, line) = hook(&dir, &own, "budget");
    assert!(out.trim().is_empty(), "a rewrite with its own content: {out}");
    assert_eq!(
        (&line["carried"], &line["before_lines"], &line["resulting_lines"]),
        (&true.into(), &800.into(), &800.into()),
        "{line}"
    );
    assert!(line.get("kind").is_none(), "a carried write has no kind: {line}");
    let edit = common::pretooluse_edit_envelope(&dir, "b.rs", "// head
", "// kept
");
    let (out, line) = hook(&dir, &edit, "budget");
    assert!(out.trim().is_empty(), "a one-line edit past the line: {out}");
    assert_eq!((&line["carried"], &line["before_lines"]), (&true.into(), &800.into()));
    let head = format!("// head
{}", filler(20));
    let cut = common::pretooluse_edit_envelope(&dir, "b.rs", &head, "");
    let (out, line) = hook(&dir, &cut, "budget");
    assert!(out.trim().is_empty(), "a shrinking edit: {out}");
    assert_eq!(
        (&line["carried"], &line["resulting_lines"]),
        (&true.into(), &779.into())
    );
    let grow = common::pretooluse_envelope_at(&dir, "b.rs", "Write", &filler(801));
    let reason = common::expect_decision(&dir, &grow, "deny");
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
    let w = common::pretooluse_envelope_at(&dir, "d.rs", "Write", &filler(751));
    let reason = common::expect_decision(&dir, &w, "deny");
    assert!(reason.contains("leaves") && reason.contains("at 751 lines"), "{reason}");
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
    let keep = common::pretooluse_envelope_at(&dir, "z.rs", "Write", &filler(700));
    let (out, line) = hook(&dir, &keep, "zone");
    assert!(out.trim().is_empty(), "an in-zone write that keeps its size: {out}");
    assert_eq!(
        (&line["carried"], &line["zone_tier"], &line["before_lines"]),
        (&true.into(), &"observe".into(), &700.into())
    );
    let grow = common::pretooluse_envelope_at(&dir, "z.rs", "Write", &filler(701));
    common::expect_decision(&dir, &grow, "ask");
    let past = common::pretooluse_envelope_at(&dir, "h.rs", "Write", &filler(800));
    let out = run_hook(&dir, &["probe", "--hook"], &past);
    assert!(out.trim().is_empty(), "a carried write past the line: {out}");
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
    let keep = common::pretooluse_envelope_at(&dir, "b.rs", "Write", &filler(800));
    let out = run_hook(&dir, &["probe", "--hook"], &keep);
    assert!(out.trim().is_empty(), "{out}");
    let grow = common::pretooluse_envelope_at(&dir, "b.rs", "Write", &filler(801));
    let reason = common::expect_decision(&dir, &grow, "allow");
    assert!(reason.contains("grows it to 801"), "{reason}");
}
