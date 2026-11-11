//! The dead-file exam's class docs (lane v235_licence, phase 2A):
//! `deadfile-class-<lang>-v1.json`, one per language of the universes.
//! Every universe row of the language with its sample membership
//! (primary / reserve / undrawn), its document class, its K0 class
//! (`universe_counts.py`, from the slice's kinds), its K1 and K1-strict
//! classes and witnesses (BARS.md §2 with addenda 2 and 3, k1.rs), the
//! ruling numbers its K1 walk met, the lines of ruling 16's fragments,
//! and the sha256 of the bytes classified; per corpus the pool and
//! primary counts BARS §1's weights read. Frozen and committed before
//! the lane opens any answer file.
//!   CE_DEADFILE_ROOT=<dir of checkouts> [CE_DEADFILE_OUT=<dir>] cargo test --test it -- --ignored eval_deadfile_parts::classes::deadfile_classes --nocapture
//! The CI gate (eval_deadfile_class.rs) re-derives everything but the
//! two K1 readings; `classes_rederive` re-derives those from the clones.

use super::{SAMPLES, doc_path, generate::pools, k1::Classes, k1::classify, k1::k0};
use crate::{
    common::ledger::out_file,
    eval_lang_parts::{freezing::frozen, generate::env, generate::freeze},
    eval_support::{content_sha, git_in, load},
};
use codeeraser::scan::lang::Lang;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

pub const SCHEMA: &str = "ce.eval-deadfile-class/1.0.0";
pub const FAMILY: &str = "deadfile-class";
/// Rulings K1-strict reads active / not constant-like (addendum 3);
/// rulings active under both readings; rulings inert under both.
pub const STRICT: [u8; 9] = [1, 2, 3, 4, 5, 6, 8, 10, 16];
pub const BOTH_ACTIVE: [u8; 4] = [7, 9, 14, 15];
pub const CONFIRMED: [u8; 3] = [11, 12, 13];
/// The three readings every summary counts.
pub const READINGS: [&str; 3] = ["k0", "k1", "k1_strict"];

const METHOD: &str = "every universe row of one language, from the frozen slices: sample \
    membership from the frozen sample (primary / reserve / undrawn); doc_class and K0 from \
    the slice row (universe_counts.py's DECLARES over the slice's top-level kinds); K1 and \
    K1-strict from the text at the pinned tip through the product's parser \
    (BARS.md section 2 with addenda 2 and 3: k1.rs, k1_lang.rs, k1_tables.rs), each with \
    the kind and line of its first active top-level statement; the ruling numbers the K1 \
    walk met; the lines of ruling 16's fragments; the sha256 of the classified bytes. Per \
    corpus the pool and primary counts of BARS section 1's weights.";

pub fn constants() -> Value {
    json!({
        "bars_sha256": {
            "BARS.md": "2934ab1c7f33f3373065a8fbc37b349a35b816481a882a4e94b7c3145a29fa65",
            "BARS-ADDENDUM-1.md": "d42c6f7020c05dac791901f75d137fd9d6c42f29a94bee643cbc4f80a0024a31",
            "BARS-ADDENDUM-2.md": "e2d5860908eb00ca722d4b83a1c557563e72feba3d1fb59f88e8c63b11baa07b",
            "BARS-ADDENDUM-3.md": "8bc0404efe4e4b5fc32bc41029c9c3c95bb7f55fcfecf4b2111c6668aaf59667",
        },
        "k0_sha256": "549337f8b658574a1181ee79836b00a1293dbc3f4a5279ca5a088822dd5dd0f4",
        "rulings": {"strict_active": STRICT, "both_active": BOTH_ACTIVE, "confirmed": CONFIRMED},
    })
}

/// (corpus, path) → membership, from the language's frozen sample.
fn membership(lang: &str) -> BTreeMap<(String, String), &'static str> {
    let sample = load(&doc_path(SAMPLES, lang));
    let mut out = BTreeMap::new();
    for (key, word) in [("rows", "primary"), ("reserve", "reserve")] {
        for r in sample[key].as_array().expect(key) {
            let id = (
                r["corpus"].as_str().expect("corpus").into(),
                r["path"].as_str().expect("path").into(),
            );
            out.insert(id, word);
        }
    }
    out
}

/// A row's fields the CI gate re-derives (no clone needed).
pub fn skeleton(r: &Value, sample: &BTreeMap<(String, String), &'static str>) -> Value {
    let s = |k: &str| r[k].as_str().expect(k).to_string();
    let kinds: Vec<String> = serde_json::from_value(r["kinds"].clone()).expect("kinds");
    let doc = !r["doc_class"].is_null();
    json!({
        "corpus": s("corpus"), "commit": s("commit"), "path": s("path"),
        "sample": sample.get(&(s("corpus"), s("path"))).copied().unwrap_or("undrawn"),
        "doc_class": r["doc_class"], "k0": k0(&s("lang"), &kinds, doc), "sha256": s("sha256"),
    })
}

/// The K1 fields of a row, from its classification.
pub fn k1_fields(c: &Classes) -> Value {
    let w = |r: &super::k1::Reading| r.witness.as_ref().map(|(k, l)| json!([k, l]));
    json!({
        "k1": c.k1.class, "k1_witness": w(&c.k1),
        "k1_strict": c.strict.class, "k1_strict_witness": w(&c.strict),
        "rulings": c.rulings, "fragments": c.fragments,
    })
}

/// The K1 keys of a row (what CI cannot re-derive).
pub const K1_KEYS: [&str; 6] = [
    "k1",
    "k1_witness",
    "k1_strict",
    "k1_strict_witness",
    "rulings",
    "fragments",
];

/// Per corpus: pool, primary and reserve counts (BARS §1 weights).
pub fn per_corpus(rows: &[Value]) -> Value {
    let mut out: BTreeMap<String, BTreeMap<&str, u64>> = BTreeMap::new();
    for r in rows {
        let cell = out
            .entry(r["corpus"].as_str().expect("corpus").into())
            .or_default();
        *cell.entry("pool").or_insert(0) += 1;
        *cell
            .entry(r["sample"].as_str().expect("sample"))
            .or_insert(0) += 1;
    }
    json!(out)
}

/// Class counts per reading over the universe and over the primary
/// rows, the rows where K1 and K1-strict differ, rows per ruling.
pub fn summary(rows: &[Value]) -> Value {
    let count = |pick: &dyn Fn(&Value) -> bool| {
        let mut by: BTreeMap<&str, BTreeMap<String, u64>> = BTreeMap::new();
        for r in rows.iter().filter(|r| pick(r)) {
            for reading in READINGS {
                let class = r[reading].as_str().expect(reading).to_string();
                *by.entry(reading).or_default().entry(class).or_insert(0) += 1;
            }
        }
        by
    };
    let mut rulings: BTreeMap<String, u64> = BTreeMap::new();
    for r in rows {
        for n in r["rulings"].as_array().expect("rulings") {
            *rulings.entry(n.to_string()).or_insert(0) += 1;
        }
    }
    json!({
        "rows": rows.len(),
        "universe": count(&|_| true),
        "primary": count(&|r| r["sample"] == "primary"),
        "k1_differs_from_strict": rows.iter().filter(|r| r["k1"] != r["k1_strict"]).count(),
        "rows_per_ruling": rulings,
    })
}

/// One language's class doc body, given each row's K1 fields.
pub fn doc(lang: &str, pool: &[Value], k1: impl Fn(&Value) -> Value) -> Value {
    let sample = membership(lang);
    let rows: Vec<Value> = pool
        .iter()
        .map(|r| {
            let mut row = skeleton(r, &sample);
            let extra = k1(r);
            for (k, v) in extra.as_object().expect("k1 fields") {
                row[k] = v.clone();
            }
            row
        })
        .collect();
    json!({"lang": lang, "per_corpus": per_corpus(&rows), "summary": summary(&rows), "rows": rows})
}

/// K1 of one slice row, read from its clone at the pinned tip; the
/// classified text must be the text the slice hashed.
pub fn classify_row(root: &str, r: &Value) -> Value {
    let (corpus, tip, path) = (
        r["corpus"].as_str().unwrap(),
        r["commit"].as_str().unwrap(),
        r["path"].as_str().unwrap(),
    );
    let text = git_in(
        Some(&format!("{root}/{corpus}")),
        &["show", &format!("{tip}:{path}")],
    );
    assert_eq!(
        content_sha(&text),
        r["sha256"],
        "{corpus}:{path}: bytes differ from the slice"
    );
    let c = classify(
        &text,
        Lang::from_path(Path::new(path)),
        !r["doc_class"].is_null(),
    );
    assert!(
        c.undecided.is_empty(),
        "{corpus}:{path}: no rule or ruling places {:?}",
        c.undecided
    );
    k1_fields(&c)
}

#[test]
#[ignore = "reads every corpus clone at its pinned tip and writes the class docs"]
fn deadfile_classes() {
    let root = env("CE_DEADFILE_ROOT");
    let (by, _) = pools();
    for (lang, pool) in &by {
        let body = doc(lang, pool, |r| classify_row(&root, r));
        println!("{lang}: {}", body["summary"]);
        let doc = frozen(SCHEMA, METHOD, constants(), body);
        freeze(&out_file("CE_DEADFILE_OUT", &doc_path(FAMILY, lang)), &doc);
    }
}

#[test]
#[ignore = "reads every corpus clone and re-derives both K1 readings of every class doc"]
fn classes_rederive() {
    let root = env("CE_DEADFILE_ROOT");
    let (mut checked, mut differ) = (0, Vec::new());
    for (lang, pool) in pools().0 {
        let filed = load(&doc_path(FAMILY, &lang));
        let rows = filed["rows"].as_array().cloned().unwrap_or_default();
        if rows.len() != pool.len() {
            differ.push(format!(
                "{lang}: {} rows filed, {} in the pool",
                rows.len(),
                pool.len()
            ));
            continue;
        }
        let fresh = pool.iter().map(|r| classify_row(&root, r));
        for (frozen_row, fresh) in rows.iter().zip(fresh) {
            checked += 1;
            let keys = K1_KEYS.iter().filter(|k| frozen_row[**k] != fresh[**k]);
            differ.extend(keys.map(|k| format!("{lang} {}: {k}", frozen_row["path"])));
        }
    }
    assert!(differ.is_empty(), "{}", differ.join("\n"));
    println!("re-derived both K1 readings of {checked} rows: equal");
}
