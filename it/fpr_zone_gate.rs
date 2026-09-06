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
//! record.

use crate::common::{self, repo_root, tmp};
use crate::eval_support::{eval_doc, load};
use crate::fpr_zone_gate_helpers::filler;
use crate::fpr_zone_replay::replay;
use crate::fpr_zone_replay_parts::{
    GATE_PPM, SCHEMA, cp_upper_ppm, false_asks, rate_ppm, shadowed, table,
};
use serde_json::Value;

#[test]
fn the_shipped_default_is_the_one_the_frozen_ledger_licenses() {
    let doc = load(&eval_doc("fpr-zone"));
    assert_eq!(doc["schema"], SCHEMA);
    assert_eq!(doc["gate_ppm"].as_u64(), Some(GATE_PPM));
    let corpora = doc["corpora"].as_array().expect("a corpora array");
    let names: Vec<&str> = corpora.iter().filter_map(|c| c["name"].as_str()).collect();
    assert_eq!(
        names,
        ["requests", "self"],
        "both measured corpora, once each"
    );
    let mut licensed = true;
    for c in corpora {
        coherent(c);
        licensed &= c["rate_ppm"].as_u64().expect("rate") <= GATE_PPM;
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
    let n = |k: &str| {
        c[k].as_u64()
            .unwrap_or_else(|| panic!("{}: no {k}", c["name"]))
    };
    let asks = c["intercepts"].as_array().expect("an intercepts array");
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
    assert_eq!(
        (
            n("observe") + n("warn") + n("ask"),
            asks.len() as u64,
            hidden,
            n("ask") - hidden,
            rate_ppm(k, events),
            cp_upper_ppm(k, events),
        ),
        (
            n("in_zone"),
            n("ask"),
            n("ask_shadowed"),
            n("false_asks"),
            n("rate_ppm"),
            n("cp_upper_ppm"),
        ),
        "{}: re-run the instrument; the row's arithmetic does not close",
        c["name"]
    );
}

#[test]
fn the_zone_doc_quotes_the_frozen_table() {
    let doc = load(&eval_doc("fpr-zone"));
    let text = crate::facts::read(&repo_root(), "docs/FPR-REPLAY.md");
    let printed = table(doc["corpora"].as_array().expect("corpora"));
    assert!(
        text.contains(printed.trim()),
        "paste the instrument's measured zone table"
    );
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
    let repo = tmp("fpr-zone-e2e");
    common::write_all(
        &repo,
        &[
            ("ce.toml", TABLE),
            ("a.rs", &filler(1)),
            ("b.rs", &filler(1)),
            ("c.rs", &filler(1)),
        ],
    );
    common::init_and_commit(&repo, "seed");
    common::write_all(
        &repo,
        &[
            ("a.rs", &filler(26)),
            ("b.rs", &filler(16)),
            ("c.rs", &filler(40)),
        ],
    );
    common::commit_all(&repo, "three landings");
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
