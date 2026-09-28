//! The graded-zone ledger's EXECUTOR (plan v2.29 step 10,
//! C-zone_tiers). `[guard] zone_tiers` ships at a default,
//! docs/FPR-REPLAY.md states the evidence for it, and
//! contracts/eval/fpr-zone-v1.json is the measurement — and until now
//! nothing held the three together. A rule written in prose and
//! executed by nobody is how a default outlives its evidence; this
//! repository has caught that failure often enough to give every such
//! rule an executor.
//!
//! Two legs. The frozen row's arithmetic closes against its own raw
//! counts (a hand-edited number reddens), and the SHIPPED default is
//! exactly what that arithmetic licenses: armed if and only if every
//! corpus measured at or under the §4.2 line. Plus one e2e over a
//! synthetic history, so the walk itself is tested and not only its
//! record. The ledger shapes — the doc, the table, the closing check,
//! the two-commit history — are common::ledger's, shared with the
//! per-language ledger (fpr_lang_gate.rs).

use crate::common::ledger::{closes, count, intercepts, rate_pair, two_commits};
use crate::common::tmp;
use crate::fpr_zone_gate_helpers::filler;
use crate::fpr_zone_replay::replay;
use crate::fpr_zone_replay_parts::{GATE_PPM, LEDGER, false_asks, shadowed, table};
use serde_json::Value;

#[test]
fn the_shipped_default_is_the_one_the_frozen_ledger_licenses() {
    let mut licensed = true;
    for c in &LEDGER.rows(&["requests", "self"]) {
        coherent(c);
        licensed &= count(c, "rate_ppm") <= GATE_PPM;
    }
    assert_eq!(
        codeeraser::config::Guard::default().zone_tiers,
        licensed,
        "the shipped `[guard] zone_tiers` default and the measured ledger disagree — \
         plan §4.2: a guard class arms only on its own FPR record, and the record is \
         docs/FPR-REPLAY.md's graded-zone section"
    );
}

fn coherent(c: &Value) {
    let n = |k: &str| count(c, k);
    let asks = intercepts(c);
    let hidden = asks.iter().filter(|r| r["shadowed"] == true).count() as u64;
    let (k, events) = (n("false_asks") as usize, n("events") as usize);
    assert!(events > 0 && n("events_shared") <= n("events"));
    assert!(n("in_zone") <= n("events") && hidden <= n("ask"));
    let soft_events: u64 = c["soft_lines"]
        .as_object()
        .expect("soft lines")
        .values()
        .map(|v| v.as_u64().expect("soft-line events"))
        .sum();
    assert_eq!(soft_events, n("events"), "each event has a soft line");
    closes(
        c,
        &[
            ("in_zone", n("observe") + n("warn") + n("ask")),
            ("ask", asks.len() as u64),
            ("ask_shadowed", hidden),
            ("false_asks", n("ask") - hidden),
        ],
    );
    closes(c, &rate_pair(k, events, ("rate_ppm", "cp_upper_ppm")));
}

#[test]
fn the_zone_doc_quotes_the_frozen_table() {
    LEDGER.page_quotes("docs/FPR-REPLAY.md", &table(&LEDGER.corpora()));
}

/// The ce.toml the synthetic history declares: S = 10, H = 30, so the
/// zone is twenty lines wide and each landing's tier is arithmetic a
/// reader can check in their head.
const TABLE: &str = "[thresholds]\nfile_lines_warn = 10\nfile_lines_fail = 30\n";

/// A synthetic two-commit history the walk reads end to end: with
/// S = 10 and H = 30, a 26-line write is 800‰ in (ask), a 16-line
/// write is 300‰ (warn), and a 40-line write is past H — the
/// hard-budget class fires there, so the zone's ask is shadowed and
/// is not this rule's false intercept.
#[test]
fn a_synthetic_history_lands_one_ask_one_warn_and_one_shadowed() {
    let repo = two_commits(
        "fpr-zone-e2e",
        &[
            ("ce.toml", TABLE),
            ("a.rs", &filler(1)),
            ("b.rs", &filler(1)),
            ("c.rs", &filler(1)),
        ],
        &[
            ("a.rs", &filler(26)),
            ("b.rs", &filler(16)),
            ("c.rs", &filler(40)),
        ],
    );
    let c = replay(&repo, &tmp("fpr-zone-e2e-shadow"), "e2e");
    assert_eq!(
        (c.commits, c.events, c.rows.len()),
        (1, 3, 3),
        "one event per changed in-scope file, each of them in the zone"
    );
    let seen: Vec<(&str, usize, bool)> = c
        .rows
        .iter()
        .map(|r| (r.tier, r.permille, r.shadowed))
        .collect();
    assert_eq!(
        seen,
        [
            ("ask", 800, false),
            ("warn", 300, false),
            ("ask", 1500, true)
        ]
    );
    assert_eq!(
        (false_asks(&c.rows), shadowed(&c.rows)),
        (1, 1),
        "the shadowed ask belongs to the hard-budget class, not to the zone"
    );
}
