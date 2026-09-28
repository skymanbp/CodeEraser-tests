//! `ce join` with the T3 family in its similarity leg (plan v2.30 step
//! 5b-9): a pair the T3 family alone relates rides the file tier with
//! no block, its near-miss count and the core's verdict, and the unit
//! tier as a `t3` row carrying the tree edit distance with churn
//! joined; the block pair beside it keeps its `t1t2` shape, and the
//! JSON document says 0.4.0.

use crate::common;
use codeeraser::join::{self, churn_unit::UnitSim};

/// Tier F: the near-miss pair has no block, a near-miss count and the
/// lattice's verdict; the block pair keeps its blocks.
fn assert_tier_f(r: &join::Report) {
    let row = |a: &str, b: &str| {
        r.files
            .iter()
            .find(|f| f.a == a && f.b == b)
            .unwrap_or_else(|| panic!("file row {a} <-> {b} in {:?}", r.files))
    };
    let cd = row("src/c.rs", "src/d.rs");
    assert_eq!(
        (cd.blocks, cd.tokens),
        (0, 0),
        "no block relates them: {cd:?}"
    );
    assert!(cd.near_miss >= 1, "the T3 family does: {cd:?}");
    assert_eq!(
        cd.verdict,
        Some("merge_candidate"),
        "sim + graph + both referenced + distinct SCCs: {cd:?}"
    );
    let ab = row("src/a.rs", "src/b.rs");
    assert!(ab.blocks >= 1, "the block pair keeps its blocks: {ab:?}");
}

/// Tier U: a `t3` row for the pair with the core's distance and both
/// node counts, churn joined onto scan_1 alone; the block row keeps
/// its tokens.
fn assert_tier_u(r: &join::Report) {
    let is_t3 = |u: &&join::churn_unit::UnitRow| {
        matches!(u.sim, UnitSim::T3 { .. }) && u.a.path == "src/c.rs" && u.b.path == "src/d.rs"
    };
    let u = r
        .units
        .iter()
        .find(is_t3)
        .expect("a t3 unit row for the pair");
    assert_eq!(
        (u.a.key.as_str(), u.b.key.as_str()),
        ("scan_1/2", "scan_2/2")
    );
    assert!(
        matches!(u.sim, UnitSim::T3 { ted, n1, n2 } if ted >= 1 && n1 >= 24 && n2 >= 24),
        "{:?}",
        u.sim
    );
    assert!(
        u.churn_a.rewrote >= 1 && u.churn_b.rewrote == 0,
        "the tweak joined onto scan_1 alone: {u:?}"
    );
    assert!(
        r.units
            .iter()
            .any(|u| matches!(u.sim, UnitSim::T1t2 { tokens } if tokens >= 50)),
        "the block pair's row keeps its tokens"
    );
}

/// The document: 0.4.0, every unit row named by kind and carrying its
/// own family's metric and nothing of the other's.
fn assert_document(r: &join::Report) {
    let doc = join::report_json(r);
    assert_eq!(doc["schema"], "ce.join-report/0.4.0");
    let units = doc["units"].as_array().expect("units");
    let own_metric = |u: &serde_json::Value| match u["kind"].as_str() {
        Some("t3") => {
            u["ted"].is_i64() && u["n1"].is_i64() && u["n2"].is_i64() && u["tokens"].is_null()
        }
        Some("t1t2") => u["tokens"].is_u64() && u["ted"].is_null(),
        _ => false,
    };
    let seen = units.iter().fold((false, false), |(a, b), u| {
        assert!(own_metric(u), "each kind carries its own metric alone: {u}");
        (a || u["kind"] == "t3", b || u["kind"] == "t1t2")
    });
    assert_eq!(seen, (true, true), "both kinds are in the document");
}

/// main.rs imports all four: a.rs / b.rs the T2 seeds, c.rs / d.rs
/// the near-miss pair; the second commit changes a literal inside
/// scan_1 (T2-invariant, tree-invariant), so the t3 row's churn leg
/// has one rewrite to join and the pair survives it.
#[test]
fn a_near_miss_pair_rides_both_tiers_beside_the_block_pair() {
    let dir = common::crate_history(
        "join-t3",
        &[
            ("a", common::rust_fn(1)),
            ("b", common::rust_fn(2)),
            ("c", common::rust_near_miss(1)),
            ("d", common::rust_near_miss(2)),
        ],
        &[("c", common::rust_near_miss(1).replace("^= 1;", "^= 5;"))],
    );
    let r = common::join_report(&dir, 30);
    assert_tier_f(&r);
    assert_tier_u(&r);
    assert_document(&r);
}
