//! The frozen same-role oracles' CI gate (plan v2.29 steps 2 and 5;
//! the ledger is docs/EVAL-SET-SIMILAR.md): every generation must
//! agree with itself — rank ids re-derive from identities, every truth
//! is in the vocabulary with a why, every metric in the summary
//! re-derives from the rows — with its predecessors (a generation past
//! the first is the holdout: no rank an earlier oracle arbitrated, and
//! it names what it held out from) and with the instrument: its
//! constants are the live constants (a changed constant is a different
//! instrument and a stale doc), and the four fixture corpora, whose
//! files are frozen upstream slices, reproduce their frozen rows byte
//! for byte under the LIVE measurement. The self rows are anchored by
//! file sha only: a self file that moved on since the tip changed the
//! corpus df with it, so those rows are counted, never re-scored.

use crate::eval_similar_precision_parts::metrics;
use crate::eval_support::{content_sha, eval_doc_v, load};
use crate::similar_replay::{CORPORA, measure};
use crate::similar_replay_parts as parts;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const TRUTHS: [&str; 3] = ["same_role", "related", "unrelated"];
/// A why shorter than this is a label, not a reason (the merge script
/// refused the same floor at assembly).
const MIN_WHY: usize = 40;

/// The frozen generations with the regression floor each ledger
/// section publishes (docs/EVAL-SET-SIMILAR.md 「裁定」3 for the first,
/// the holdout section for the second): a percentage on the three
/// published numbers — role-bit-1 p@1, hit@5, and the role bit's
/// precision over every arbitrated candidate — the smallest of the
/// three measurements rounded down to a tenth, allowed only to rise.
/// A re-frozen oracle below it is a worse instrument: an event, not a
/// number to accept. v1 measured 39/59, 74/118, 101/165; the v2
/// holdout 30/56, 69/115, 86/177 — the role bit's precision there is
/// what sets its floor.
const GENERATIONS: [(u32, u64); 2] = [(1, 60), (2, 40)];

fn oracle(generation: u32) -> Value {
    load(&eval_doc_v("similar-oracle", generation))
}

/// The generation a doc says it is; the first oracle predates the
/// field (there was nothing to hold out from) and reads as 1.
fn generation_of(doc: &Value) -> u32 {
    doc["generation"].as_u64().map_or(1, |g| g as u32)
}

/// The arbitration fields a candidate row carries beyond the
/// instrument's own — stripped before comparing against a live row.
const ARBITRATION: [&str; 3] = ["truth", "clone", "why"];

fn instrument_half(candidate: &Value) -> Value {
    let mut c = candidate.clone();
    for k in ARBITRATION {
        c.as_object_mut().expect("candidate").remove(k);
    }
    c
}

/// Per generation: rank order and re-derivation, truth vocabulary, why
/// floor, live constants, the holdout against every earlier
/// generation, summary re-derived from the rows.
#[test]
fn similar_oracle_consistent() {
    let mut arbitrated = BTreeSet::new();
    for (g, _) in GENERATIONS {
        let doc = oracle(g);
        assert_eq!(generation_of(&doc), g, "v{g}: generation field");
        assert_eq!(
            doc["constants"],
            parts::constants(),
            "v{g}: instrument constants moved"
        );
        assert_eq!(doc["truths"], serde_json::json!(TRUTHS));
        let rows = doc["rows"].as_array().expect("rows");
        assert!(
            rows.len() >= 100,
            "v{g}: the spec asks for at least 100 arbitrated queries"
        );
        for pair in rows.windows(2) {
            assert!(
                pair[0]["rank"].as_str() < pair[1]["rank"].as_str(),
                "v{g}: rows unsorted"
            );
        }
        rows.iter().for_each(assert_row);
        assert_holdout(g, &doc, rows, &mut arbitrated);
        assert_summary(g, &doc, rows);
    }
}

/// A generation past the first is the holdout: none of its ranks was
/// arbitrated by an earlier generation (disjoint by construction in
/// the draw, checked here against the frozen docs themselves), and it
/// names the oracle it held out from.
fn assert_holdout(g: u32, doc: &Value, rows: &[Value], arbitrated: &mut BTreeSet<String>) {
    if g > 1 {
        assert_eq!(
            doc["holdout_of"],
            format!("similar-oracle-v{}", g - 1),
            "v{g}: holdout_of"
        );
    }
    for r in rows {
        let rank = r["rank"].as_str().expect("rank").to_string();
        assert!(
            arbitrated.insert(rank),
            "v{g}: rank {} was arbitrated by an earlier generation",
            r["rank"]
        );
    }
}

/// `summary.all` and every corpus block re-derive from the rows.
fn assert_summary(g: u32, doc: &Value, rows: &[Value]) {
    assert_eq!(
        doc["summary"]["all"],
        metrics(rows),
        "v{g}: summary drifted from the rows"
    );
    for (name, _) in CORPORA {
        let sub: Vec<Value> = rows
            .iter()
            .filter(|r| r["corpus"] == name)
            .cloned()
            .collect();
        assert!(!sub.is_empty(), "v{g} {name}: no arbitrated row");
        assert_eq!(
            doc["summary"][name],
            metrics(&sub),
            "v{g} {name}: summary drifted"
        );
    }
}

/// One row's own consistency: the rank re-derives from the identity,
/// every candidate carries a truth in the vocabulary, a clone flag
/// and a why past the floor.
fn assert_row(r: &Value) {
    let rank = content_sha(&format!(
        "similar|{}|{}|{}|{}",
        r["corpus"].as_str().expect("corpus"),
        r["path"].as_str().expect("path"),
        r["key"].as_str().expect("key"),
        r["nth"]
    ));
    assert_eq!(r["rank"], rank, "rank does not re-derive from the identity");
    for c in r["candidates"].as_array().expect("candidates") {
        assert!(
            TRUTHS.contains(&c["truth"].as_str().expect("truth")),
            "{rank}"
        );
        assert!(c["clone"].is_boolean(), "{rank}: clone flag");
        assert!(
            c["why"].as_str().expect("why").chars().count() >= MIN_WHY,
            "{rank}: why"
        );
    }
}

/// Every generation's three published numbers hold its floor
/// (`GENERATIONS`).
#[test]
fn similar_oracle_floors() {
    let ratio = |v: &Value| (v[0].as_u64().expect("num"), v[1].as_u64().expect("den"));
    for (g, floor) in GENERATIONS {
        let doc = oracle(g);
        let all = &doc["summary"]["all"];
        let role = &all["role_bit_over_candidates"];
        let tp = role["tp"].as_u64().expect("tp");
        let fp = role["fp"].as_u64().expect("fp");
        let published = [
            ("p_at_1_bare_role1", ratio(&all["p_at_1_bare_role1"])),
            ("hit_at_5_bare", ratio(&all["hit_at_5_bare"])),
            ("role_bit_precision", (tp, tp + fp)),
        ];
        for (name, (n, d)) in published {
            assert!(
                n * 100 >= floor * d,
                "v{g} {name}: {n}/{d} below the {floor} % floor"
            );
        }
    }
}

/// The four fixture corpora re-measured live, once each: every frozen
/// fixture row of every generation must come back with its candidate
/// list (identity, evidence, arms) identical, and every file the rows
/// name must still read the same (the fixtures are pinned upstream
/// slices — a changed sha there is a changed fixture, which is its
/// own event).
#[test]
fn similar_fixture_rows_replay_byte_for_byte() {
    let docs: Vec<(u32, Value)> = GENERATIONS.iter().map(|(g, _)| (*g, oracle(*g))).collect();
    let root = crate::common::repo_root();
    for (name, rel) in CORPORA.iter().filter(|(n, _)| *n != "self") {
        let m = measure(&root.join(rel), name);
        let seats: BTreeMap<(&str, &str, i64), usize> = m
            .corpus
            .docs
            .iter()
            .enumerate()
            .map(|(i, d)| ((d.path.as_str(), d.bag.key.as_str(), d.bag.nth), i))
            .collect();
        for (g, doc) in &docs {
            let rows: Vec<&Value> = doc["rows"]
                .as_array()
                .expect("rows")
                .iter()
                .filter(|r| r["corpus"] == *name)
                .collect();
            assert!(!rows.is_empty(), "v{g} {name}: no frozen row");
            for r in rows {
                assert_replays(&m, &seats, r, &format!("v{g} {name}"));
            }
        }
    }
}

/// One frozen row against the live measurement: the file's identity
/// unchanged, the candidate list identical once the arbitration
/// fields are stripped.
fn assert_replays(
    m: &crate::similar_replay::Measured,
    seats: &BTreeMap<(&str, &str, i64), usize>,
    r: &Value,
    at: &str,
) {
    let key = (
        r["path"].as_str().expect("path"),
        r["key"].as_str().expect("key"),
        r["nth"].as_i64().expect("nth"),
    );
    let live = parts::row(
        m,
        *seats
            .get(&key)
            .unwrap_or_else(|| panic!("{at}: {key:?} gone")),
    );
    assert_eq!(live["sha"], r["sha"], "{at}: {} moved on", key.0);
    let frozen: Vec<Value> = r["candidates"]
        .as_array()
        .expect("candidates")
        .iter()
        .map(instrument_half)
        .collect();
    assert_eq!(
        live["candidates"],
        serde_json::json!(frozen),
        "{at}: {key:?} re-ranked"
    );
}
