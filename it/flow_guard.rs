//! The PreToolUse leg of the flow class through the real hook (plan
//! v2.31 step 5, design booklet §5.4): each side of a write lowered
//! and judged over the daemon, and the after side's findings the
//! before side does not account for counted as NOVEL — a moved
//! finding is not new, an unused parameter never makes the condition.
//! The class speaks at its own `[flow] tier` and only for a judged
//! language (`flow::judged_mask()`): with the language outside the
//! mask the leg records `judged: false` and never speaks. Python is
//! the judged side's fixture and Rust the advisory side's while the
//! mask reads them so; every assertion still branches on the mask, so
//! a later mask moves the legs and never breaks them. The hook writes
//! nothing to disk, so each case seeds the BEFORE text and sends the
//! AFTER text as the Write payload.

use crate::common;
use codeeraser::flow_report::lang_judged;
use codeeraser::scan::lang::Lang;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// One language's fixture: the file a write lands in, its text before
/// the write, and the same unit with an unreachable statement.
struct Side {
    lang: Lang,
    file: &'static str,
    before: &'static str,
    unreachable: &'static str,
}

const PY: Side = Side {
    lang: Lang::Python,
    file: "b.py",
    before: "def gone():\n    return 1\n",
    unreachable: "def gone():\n    return 1\n    print(\"never\")\n",
};

const RS: Side = Side {
    lang: Lang::Rust,
    file: "b.rs",
    before: "fn gone() -> i32 {\n    return 1;\n}\n",
    unreachable: "fn gone() -> i32 {\n    return 1;\n    let _late = 2;\n}\n",
};

fn seed(name: &str, flow_tier: Option<&str>, side: &Side) -> PathBuf {
    let dir = common::tmp(name);
    let tier = flow_tier.map_or(String::new(), |t| format!("[flow]\ntier = \"{t}\"\n"));
    common::declare(&dir, &format!("[guard]\nmode = \"observe\"\n{tier}"));
    std::fs::write(dir.join(side.file), side.before).expect("before");
    dir
}

/// Write `after` over the side's file: the hook's stdout, and the
/// event's `flow` line when it left one.
fn written(dir: &Path, file: &str, after: &str) -> (String, Option<Value>) {
    let env = common::pretooluse_envelope_at(dir, file, "Write", after);
    let out = common::run_hook(dir, &["probe", "--hook"], &env);
    let line = common::observe_lines(dir)
        .into_iter()
        .rfind(|v| v["event"] == "flow");
    (out, line)
}

#[test]
fn an_introduced_unreachable_statement_is_one_novel_finding_in_observe() {
    let dir = seed("flow-guard-observe", None, &PY);
    let (out, line) = written(&dir, PY.file, PY.unreachable);
    assert!(out.trim().is_empty(), "observe stays silent: {out}");
    let line = line.expect("a flow line");
    assert_eq!(line["mode"], "observe");
    let flow = &line["flow"];
    assert_eq!(
        (&flow["before"], &flow["after"], &flow["novel"]),
        (&0.into(), &1.into(), &1.into()),
        "{line}"
    );
    assert_eq!(flow["kinds"]["unreachable"], 1);
    assert_eq!(flow["judged"], lang_judged(PY.lang));
    // the flow line lands ahead of the probe line, which stays the event's last
    assert_eq!(common::last_observe(&dir)["event"], "probe");
}

/// At warn the class speaks one sentence, at deny it refuses the
/// write — for a judged language; outside the mask neither tier
/// speaks and the feed says `judged: false`. Both sides are asked.
#[test]
fn the_class_speaks_at_its_own_tier_only_for_a_judged_language() {
    for side in [&PY, &RS] {
        for (tier, decision) in [("warn", "allow"), ("deny", "deny")] {
            let dir = seed(
                &format!("flow-guard-{}-{tier}", side.file),
                Some(tier),
                side,
            );
            let (out, line) = written(&dir, side.file, side.unreachable);
            let line = line.expect("a flow line");
            assert_eq!(line["mode"], tier);
            assert_eq!(line["flow"]["judged"], lang_judged(side.lang));
            if !lang_judged(side.lang) {
                assert!(
                    out.trim().is_empty(),
                    "{tier}: advisory never speaks: {out}"
                );
                continue;
            }
            let v: Value = serde_json::from_str(out.trim()).expect("a decision");
            assert_eq!(v["hookSpecificOutput"]["permissionDecision"], decision);
            let why = v["hookSpecificOutput"]["permissionDecisionReason"]
                .as_str()
                .unwrap();
            assert!(
                why.contains("1 new dead-code finding(s)") && why.contains("gone unreachable"),
                "{why}"
            );
        }
    }
}

/// A finding that only moved is not new, and an unused parameter is
/// never part of the condition.
#[test]
fn a_moved_finding_and_an_unused_parameter_are_not_novel() {
    let dir = seed("flow-guard-moved", Some("deny"), &PY);
    std::fs::write(dir.join(PY.file), PY.unreachable).expect("before");
    let moved = format!("# a comment above\n\n{}", PY.unreachable);
    let (out, line) = written(&dir, PY.file, &moved);
    assert!(out.trim().is_empty(), "nothing new: {out}");
    let flow = &line.expect("a flow line")["flow"];
    assert_eq!(
        (&flow["before"], &flow["after"], &flow["novel"]),
        (&1.into(), &1.into(), &0.into())
    );
    let dir = seed("flow-guard-param", Some("deny"), &PY);
    let (out, line) = written(&dir, PY.file, "def gone(unused):\n    return 1\n");
    assert!(out.trim().is_empty(), "a parameter is advisory: {out}");
    let flow = &line.expect("a flow line")["flow"];
    assert_eq!(
        (&flow["kinds"]["unused_param"], &flow["novel"]),
        (&1.into(), &0.into())
    );
}

/// A clean write leaves no flow line at all.
#[test]
fn a_write_without_findings_leaves_no_line() {
    let dir = seed("flow-guard-clean", None, &PY);
    let (_, line) = written(&dir, PY.file, "def kept():\n    return 2\n");
    assert!(line.is_none(), "{line:?}");
}
