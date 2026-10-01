//! The Stop / precommit leg of the flow class (plan v2.31 step 5,
//! design booklet §5.4): every changed file's after side lowered and
//! judged once over the audit's core link, the feed line's `flow`
//! object counting what the core found — never a block at any tier,
//! and never the sites (those are `ce flow`'s); the object is in the
//! feed since ce.observe/0.12.0. The fixture is Python, a judged language while
//! the mask reads it so: a finding the class would refuse at write
//! time still never stops a Stop.

use crate::common;
use codeeraser::scan::lang::Lang;
use serde_json::json;
use std::path::PathBuf;

const BEFORE: &str = "def gone():\n    return 1\n";
const AFTER: &str = "def gone():\n    return 1\n    print(\"never\")\n";

/// A committed repo whose b.py then gains an unreachable statement in
/// the working tree, at `[flow] tier = "deny"` — which must still not
/// block a Stop.
fn changed(name: &str) -> PathBuf {
    let dir = common::tmp(name);
    common::declare(
        &dir,
        "[guard]\nmode = \"observe\"\n[flow]\ntier = \"deny\"\n",
    );
    // the seed committed, then the change left in the working tree
    for (text, seed) in [(BEFORE, true), (AFTER, false)] {
        std::fs::write(dir.join("b.py"), text).expect("b.py");
        if seed {
            common::init_and_commit(&dir, "seed");
        }
    }
    dir
}

fn expected_flow() -> serde_json::Value {
    let judged = u64::from(codeeraser::flow_report::lang_judged(Lang::Python));
    json!({
        "files": 1, "units": 1, "findings": 1, "judged": judged,
        "kinds": {"unreachable": 1, "dead_store": 0, "unused_local": 0, "unused_param": 0},
    })
}

#[test]
fn the_stop_line_carries_the_flow_counts_and_never_blocks() {
    let dir = changed("flow-audit-stop");
    let line = common::stop_observe(&dir);
    assert_eq!(line["schema"], codeeraser::hookio::OBSERVE_SCHEMA);
    assert_eq!(line["flow"], expected_flow(), "{line}");
}

#[test]
fn precommit_counts_the_staged_set_and_exits_0() {
    let dir = changed("flow-audit-precommit");
    common::git(&dir, &["add", "b.py"]);
    let out = common::run_ce(&dir, &["precommit"]);
    assert!(out.status.success(), "{out:?}");
    let line = common::last_observe(&dir);
    assert_eq!(line["event"], "precommit");
    assert_eq!(line["flow"], expected_flow(), "{line}");
}
