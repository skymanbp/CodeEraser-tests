//! `ce join` end to end (M5-3h): the three legs assemble over a real
//! fixture — a T2 clone pair both imported by main.rs, with a window
//! history that rewrites inside one clone body and appends outside
//! the other. Pins: the file tier carries all three legs (graph
//! position answered through the SAME wire deadcode judges), the
//! unit tier joins churn on the (path, key, anchor) identity, and the
//! run is report-only (no verdicts before 3i).

use crate::common;
use codeeraser::join;
use std::path::PathBuf;

/// Two commits of a real Cargo layout: without Cargo.toml the rs
/// ladder has no crate root and `mod a;` correctly refuses. The
/// window edit is a literal tweak INSIDE work_1 (T2-invariant, so
/// the clone pair survives; churn sees a rewrite) plus a top-level
/// comment on b.rs (append; comments never enter the token stream).
fn fixture() -> PathBuf {
    common::crate_history(
        "join-e2e",
        &[("a", common::rust_fn(1)), ("b", common::rust_fn(2))],
        &[
            ("a", common::rust_fn(1).replace("+ 7;", "+ 9;")),
            ("b", format!("{}// touched\n", common::rust_fn(2))),
        ],
    )
}

/// Tier F: the one similar pair carries all three legs.
fn assert_tier_f(r: &join::Report) {
    let [f] = r.files.as_slice() else {
        panic!("one file pair, got {:?}", r.files);
    };
    assert_eq!((f.a.as_str(), f.b.as_str()), ("src/a.rs", "src/b.rs"));
    assert!(f.blocks >= 1 && f.tokens >= 50, "similarity leg: {f:?}");
    let ga = f.graph_a.expect("a.rs graph leg answered");
    let gb = f.graph_b.expect("b.rs graph leg answered");
    assert!(
        ga[0] >= 1 && gb[0] >= 1,
        "main.rs imports both: {ga:?} {gb:?}"
    );
    assert!(f.churn_a.rewrote >= 1, "work_1 tweak is a rewrite: {f:?}");
    assert!(f.churn_b.appended >= 1, "b.rs comment is an append: {f:?}");
    assert_eq!(f.cochange, Some(2), "both commits touched the pair");
}

/// Tier U: the block's sides attribute to the two work units, and
/// churn joins on the SAME (path, key, anchor) identity the ledger
/// wrote — the rewrite lands on work_1, never its twin.
fn assert_tier_u(r: &join::Report) {
    let u = r
        .units
        .iter()
        .find(|u| u.a.key == "work_1/2" || u.b.key == "work_1/2")
        .expect("unit row for the clone pair");
    let (one, two, c1, c2) = if u.a.key == "work_1/2" {
        (&u.a, &u.b, u.churn_a, u.churn_b)
    } else {
        (&u.b, &u.a, u.churn_b, u.churn_a)
    };
    assert_eq!((one.path.as_str(), one.nth), ("src/a.rs", 0));
    assert_eq!(
        (two.path.as_str(), two.key.as_str(), two.nth),
        ("src/b.rs", "work_2/2", 0)
    );
    assert!(c1.rewrote >= 1, "the tweak joined onto work_1: {u:?}");
    assert!(
        c2.appended >= 8 && c2.rewrote == 0,
        "work_2 only appended: {u:?}"
    );
}

#[test]
fn three_legs_assemble_on_both_tiers() {
    let r = common::join_report(&fixture(), 30);
    assert_eq!(r.commits, 2);
    assert_tier_f(&r);
    assert_tier_u(&r);
    // the JSON form declares itself and never fabricates a unit-tier
    // graph leg: null plus the R6 caveat CODE on every unit row
    // (plan v2.15 — the sentence stopped riding the machine face)
    let doc = join::report_json(&r);
    assert_eq!(doc["schema"], "ce.join-report/0.4.0");
    for row in doc["units"].as_array().expect("units") {
        assert!(row["graph"].is_null(), "unit graph leg is null: {row}");
        assert!(row["caveat"].is_null(), "the prose field is gone: {row}");
        assert_eq!(
            row["caveatCode"],
            join::churn_unit::GRAPH_NULL_IMPORT_GRANULARITY
        );
    }
    // the judgment road (2.33.0): every non-self file pair carries
    // the core's verdict with its severity rank and leg-agreement
    // confidence — the fixture's pair is similar and both-alive, so
    // the lattice answers merge_candidate with all three legs heard
    for f in &r.files {
        if f.a == f.b {
            continue; // the wire's u < v contract: self-pairs carry no row
        }
        assert_eq!(f.verdict, Some("merge_candidate"), "{} <-> {}", f.a, f.b);
        assert_eq!(f.severity, Some(2));
        assert!(
            f.confidence == Some(2) || f.confidence == Some(3),
            "legs present and held: {:?}",
            f.confidence
        );
    }
}
