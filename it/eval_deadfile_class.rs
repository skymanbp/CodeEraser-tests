//! The dead-file exam's class docs, CI side (lane v235_licence, phase
//! 2A; family in eval_deadfile_parts/classes.rs). Every language of the
//! universes has exactly one frozen class doc; everything in it that
//! needs no corpus clone is re-derived — the row set and order, sample
//! membership, document class, K0, the classified bytes' sha256 (the
//! slice's), the per-corpus counts, the summary — and the definitions
//! of addenda 2 and 3 hold row by row. Both K1 readings themselves are
//! re-derived from the clones by `classes_rederive` (`#[ignore]`).

use crate::eval_deadfile_parts::classes::{
    BOTH_ACTIVE, CONFIRMED, FAMILY, K1_KEYS, SCHEMA, STRICT, constants, doc,
};
use crate::eval_deadfile_parts::gate::assert_language_docs;
use crate::eval_support::{frozen_docs, load};
use serde_json::Value;

#[test]
fn every_class_doc_re_derives_from_the_frozen_exam() {
    assert_language_docs(FAMILY, SCHEMA, &constants(), |lang, pool, filed| {
        let rows = filed["rows"].as_array().expect("rows");
        assert_eq!(rows.len(), pool.len(), "{lang}: row count");
        // the filed K1 fields ride along; everything else is derived here
        doc(lang, pool, |r| {
            let at = pool.iter().position(|p| p == r).expect("pool row");
            let keep = K1_KEYS
                .iter()
                .map(|k| (k.to_string(), rows[at][*k].clone()));
            Value::Object(keep.collect())
        })
    });
}

/// The definitions of addenda 2 and 3, row by row.
fn definitional(r: &Value) -> Vec<&'static str> {
    let mut bad = Vec::new();
    for (class, witness) in [("k1", "k1_witness"), ("k1_strict", "k1_strict_witness")] {
        if (r[class] == "runs-alone") == r[witness].is_null() {
            bad.push("runs-alone exactly when its witness is non-null");
        }
    }
    let doc = !r["doc_class"].is_null();
    if (r["k1"] == "document") != doc || (r["k1_strict"] == "document") != doc {
        bad.push("document exactly when doc_class is non-null, under both");
    }
    if r["k1"] == "runs-alone" && r["k1_strict"] != "runs-alone" {
        bad.push("runs-alone under k1 is runs-alone under k1_strict");
    }
    let rulings: Vec<u64> = r["rulings"]
        .as_array()
        .expect("rulings")
        .iter()
        .map(|n| n.as_u64().expect("n"))
        .collect();
    let strict_met = rulings.iter().any(|n| STRICT.contains(&(*n as u8)));
    if !strict_met && r["k1"] != r["k1_strict"] {
        bad.push("no undecided kind: k1 == k1_strict");
    }
    let known = |n: &u64| {
        STRICT
            .iter()
            .chain(&BOTH_ACTIVE)
            .chain(&CONFIRMED)
            .any(|k| u64::from(*k) == *n)
    };
    if !rulings.iter().all(known) {
        bad.push("every ruling number is one of addenda 2 and 3");
    }
    let fragments = !r["fragments"].as_array().expect("fragments").is_empty();
    if fragments != rulings.contains(&16) {
        bad.push("fragment lines exactly when ruling 16 is met");
    }
    bad
}

#[test]
fn every_row_keeps_the_definitions() {
    let mut bad = Vec::new();
    for path in frozen_docs(&format!("{FAMILY}-")) {
        for r in load(&path)["rows"].as_array().expect("rows") {
            for why in definitional(r) {
                bad.push(format!("{path} {}: {why}", r["path"]));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
