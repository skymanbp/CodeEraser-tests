//! The same-role oracles' provenance (plan v2.29 steps 2 and 5; the
//! ledger is docs/EVAL-SET-SIMILAR.md): each frozen oracle was derived
//! from an arbitration record kept beside the suite
//! (it/eval_similar_review/: `sample.json` for the first generation,
//! `sample-v<g>.json` after it). The record holds one row per (query
//! rank, candidate, 1-based); the oracle's truth, clone flag and why
//! must be the record's, row for row — a label edited in the oracle
//! alone, or a candidate the record never answered, is a provenance
//! break, the check the other four review directories already have.

use crate::eval_similar_precision::GENERATIONS;
use crate::eval_support::{eval_doc_v, load};
use serde_json::Value;
use std::collections::BTreeMap;

/// The record of generation `g`, keyed (rank, candidate), each row's
/// three labels.
fn record(g: u32) -> (String, BTreeMap<(String, u64), [Value; 3]>) {
    let name = if g == 1 {
        "sample.json".to_string()
    } else {
        format!("sample-v{g}.json")
    };
    let path = crate::common::repo_root()
        .join("cli/tests/it/eval_similar_review")
        .join(&name);
    let doc: Value = serde_json::from_str(
        &std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{name}: {e}")),
    )
    .unwrap_or_else(|e| panic!("{name}: {e}"));
    let mut labels = BTreeMap::new();
    for r in doc["rows"].as_array().expect("record rows") {
        let at = (
            r["rank"].as_str().expect("rank").to_string(),
            r["candidate"].as_u64().expect("candidate"),
        );
        let label = [r["truth"].clone(), r["clone"].clone(), r["why"].clone()];
        assert!(
            labels.insert(at.clone(), label).is_none(),
            "{name}: {at:?} answered twice"
        );
    }
    (name, labels)
}

#[test]
fn similar_oracle_labels_are_the_arbitration_record() {
    for (g, _) in GENERATIONS {
        let (name, labels) = record(g);
        let oracle = load(&eval_doc_v("similar-oracle", g));
        let mut seen = 0;
        for q in oracle["rows"].as_array().expect("rows") {
            let rank = q["rank"].as_str().expect("rank");
            for (i, c) in q["candidates"]
                .as_array()
                .expect("candidates")
                .iter()
                .enumerate()
            {
                let at = (rank.to_string(), i as u64 + 1);
                let label = labels
                    .get(&at)
                    .unwrap_or_else(|| panic!("v{g}: {at:?} has no row in {name}"));
                let held = [c["truth"].clone(), c["clone"].clone(), c["why"].clone()];
                assert_eq!(label, &held, "v{g}: {at:?} differs from {name}");
                seen += 1;
            }
        }
        assert_eq!(
            seen,
            labels.len(),
            "v{g}: {name} answers candidates the oracle does not carry"
        );
    }
}
