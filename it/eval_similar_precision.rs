//! The frozen same-role oracle's CI gate (plan v2.29 step 2; the
//! ledger is docs/EVAL-SET-SIMILAR.md): the oracle must agree with
//! itself — rank ids re-derive from identities, every truth is in the
//! vocabulary with a why, every metric in the summary re-derives from
//! the rows — and with the instrument: its constants are the live
//! constants (a changed constant is a different instrument and a
//! stale doc), and the four fixture corpora, whose files are frozen
//! upstream slices, reproduce their frozen rows byte for byte under
//! the LIVE measurement. The self rows are anchored by file sha only:
//! a self file that moved on since the tip changed the corpus df with
//! it, so those rows are counted, never re-scored.

use crate::eval_support::{content_sha, eval_doc, load};
use crate::similar_replay::{CORPORA, measure};
use crate::similar_replay_parts as parts;
use serde_json::Value;
use std::collections::BTreeMap;

const TRUTHS: [&str; 3] = ["same_role", "related", "unrelated"];
/// A why shorter than this is a label, not a reason (the merge script
/// refused the same floor at assembly).
const MIN_WHY: usize = 40;

fn oracle() -> Value {
    load(&eval_doc("similar-oracle"))
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

/// Rank order and re-derivation, truth vocabulary, why floor, live
/// constants, summary re-derived from the rows.
#[test]
fn similar_oracle_consistent() {
    let doc = oracle();
    assert_eq!(
        doc["constants"],
        parts::constants(),
        "instrument constants moved"
    );
    assert_eq!(doc["truths"], serde_json::json!(TRUTHS));
    let rows = doc["rows"].as_array().expect("rows");
    assert!(
        rows.len() >= 100,
        "the spec asks for at least 100 arbitrated queries"
    );
    for pair in rows.windows(2) {
        assert!(
            pair[0]["rank"].as_str() < pair[1]["rank"].as_str(),
            "rows unsorted"
        );
    }
    rows.iter().for_each(assert_row);
    assert_eq!(
        doc["summary"]["all"],
        metrics(rows),
        "summary drifted from the rows"
    );
    for (name, _) in CORPORA {
        let sub: Vec<Value> = rows
            .iter()
            .filter(|r| r["corpus"] == name)
            .cloned()
            .collect();
        assert!(!sub.is_empty(), "{name}: no arbitrated row");
        assert_eq!(
            doc["summary"][name],
            metrics(&sub),
            "{name}: summary drifted"
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

/// The candidate at an arm's rank 1, if the arm answered.
fn top1<'r>(r: &'r Value, arm: &str) -> Option<&'r Value> {
    r["candidates"]
        .as_array()
        .expect("candidates")
        .iter()
        .find(|c| c[arm]["rank"] == 1)
}

fn frac(rows: &[&Value], hit: impl Fn(&Value) -> bool) -> Value {
    serde_json::json!([rows.iter().filter(|c| hit(c)).count(), rows.len()])
}

/// The metric block the merge script wrote, re-derived here: p@1 per
/// arm over all answered queries, over the role-bit-1 and role-bit-0
/// top-1s, over non-clone top-1s; hit@5 per arm; the role bit's
/// confusion over every arbitrated candidate.
fn metrics(rows: &[Value]) -> Value {
    let same = |c: &Value| c["truth"] == "same_role";
    let mut m = serde_json::Map::new();
    m.insert("queries".into(), rows.len().into());
    let all: Vec<&Value> = rows
        .iter()
        .flat_map(|r| r["candidates"].as_array().expect("candidates"))
        .collect();
    m.insert("candidates".into(), all.len().into());
    for arm in ["bare", "widened"] {
        arm_metrics(rows, arm, &mut m);
    }
    let cell = |role: bool, truth: bool| {
        all.iter()
            .filter(|c| (c["role"] == role) && (same(c) == truth))
            .count()
    };
    m.insert(
        "role_bit_over_candidates".into(),
        serde_json::json!({"tp": cell(true, true), "fp": cell(true, false), "fn": cell(false, true), "tn": cell(false, false)}),
    );
    Value::Object(m)
}

/// One arm's block: p@1 over all answered queries and over the
/// role-bit-1, role-bit-0 and non-clone top-1s; hit@5.
fn arm_metrics(rows: &[Value], arm: &str, m: &mut serde_json::Map<String, Value>) {
    let same = |c: &Value| c["truth"] == "same_role";
    let answered: Vec<&Value> = rows.iter().filter_map(|r| top1(r, arm)).collect();
    let subset = |field: &str, want: bool| -> Vec<&Value> {
        answered
            .iter()
            .copied()
            .filter(|c| c[field] == want)
            .collect()
    };
    m.insert(format!("p_at_1_{arm}"), frac(&answered, same));
    m.insert(
        format!("p_at_1_{arm}_nonclone"),
        frac(&subset("clone", false), same),
    );
    m.insert(
        format!("p_at_1_{arm}_role1"),
        frac(&subset("role", true), same),
    );
    m.insert(
        format!("p_at_1_{arm}_role0"),
        frac(&subset("role", false), same),
    );
    let hit5 = rows
        .iter()
        .filter(|r| {
            r["candidates"]
                .as_array()
                .expect("candidates")
                .iter()
                .any(|c| !c[arm].is_null() && same(c))
        })
        .count();
    m.insert(
        format!("hit_at_5_{arm}"),
        serde_json::json!([hit5, rows.len()]),
    );
}

/// The regression floor the ledger publishes (docs/EVAL-SET-SIMILAR.md
/// 「裁定」3): 60 % on the three published numbers — role-bit-1 p@1
/// (measured 39/59), hit@5 (74/118) and the role bit's precision over
/// every arbitrated candidate (101/165) — each the measurement rounded
/// down to a tenth, allowed only to rise. A re-frozen oracle below it
/// is a worse instrument: an event, not a number to accept.
const FLOOR_PERCENT: u64 = 60;

#[test]
fn similar_oracle_floors() {
    let doc = oracle();
    let all = &doc["summary"]["all"];
    let ratio = |v: &Value| (v[0].as_u64().expect("num"), v[1].as_u64().expect("den"));
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
            n * 100 >= FLOOR_PERCENT * d,
            "{name}: {n}/{d} below the {FLOOR_PERCENT} % floor"
        );
    }
}

/// The four fixture corpora re-measured live: every frozen fixture
/// row's candidate list (identity, evidence, arms) must come back
/// identical, and every file the rows name must still read the same
/// (the fixtures are pinned upstream slices — a changed sha there is
/// a changed fixture, which is its own event).
#[test]
fn similar_fixture_rows_replay_byte_for_byte() {
    let doc = oracle();
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
        let rows: Vec<&Value> = doc["rows"]
            .as_array()
            .expect("rows")
            .iter()
            .filter(|r| r["corpus"] == *name)
            .collect();
        assert!(!rows.is_empty(), "{name}: no frozen row");
        for r in rows {
            let key = (
                r["path"].as_str().expect("path"),
                r["key"].as_str().expect("key"),
                r["nth"].as_i64().expect("nth"),
            );
            let live = parts::row(
                &m,
                *seats.get(&key).unwrap_or_else(|| panic!("{key:?} gone")),
            );
            assert_eq!(live["sha"], r["sha"], "{name}: {} moved on", key.0);
            let frozen: Vec<Value> = r["candidates"]
                .as_array()
                .expect("candidates")
                .iter()
                .map(instrument_half)
                .collect();
            assert_eq!(
                live["candidates"],
                serde_json::json!(frozen),
                "{name}: {key:?} re-ranked"
            );
        }
    }
}
