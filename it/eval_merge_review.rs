//! The clone-merge family's audit (plan v2.31 step 7, step-7 ruling 2):
//! the sample is the hash-ranked draw of the frozen set, row for row;
//! the audit is blind to the verdict — feasibility, reason, savings and
//! the member kept are withheld — and sees the core's parameterisation,
//! which it may reject. The audit table (`merge-review-v<n>.json`, filled
//! by independent agents who read the sample alone and assembled from
//! their answer files by `assemble`, answers.rs) answers each sampled
//! row with `{id, feasible, reason, params, note}`, `reason` one of the
//! six names; the precision doc reads, per corpus and overall, how often
//! the core's feasibility and reason agree with the audit's, and
//! `params_agree` — how often the count the audit gives after seeing the
//! core's parameterisation equals the core's. The precision doc is a
//! pure function of the three frozen docs (set, sample, review) and
//! lands with the review (design booklet §13 item 36); each gate reads a
//! doc only once it is filed, and says so while it is not. Every
//! generation is read the same way (merge generation 2): the exam's
//! docs as they are filed, an earlier generation's as the record they
//! are, its review and precision doc still what its three docs read.
//!   CE_BLESS=1 CE_MERGE_BATCH_DIR=<dir> CE_MERGE_AUDITOR="<one sentence>" cargo test --test it -- --ignored eval_merge_review::assemble --nocapture
//!   CE_BLESS=1 cargo test --test it -- --ignored eval_merge_review::regenerate --nocapture

mod answers;

use crate::eval_lang_parts::generate::env;
use crate::eval_merge_batches::answer_file;
use crate::eval_merge_parts::sample::{draw, id_of};
use crate::eval_merge_parts::{EXAM, GENERATIONS, Generation, load, repo_file, write};
use crate::eval_support::{Forgery, assert_forgeries_refused, assert_tampering_refused};
use crate::eval_support::{generated_from, load as load_path};
use answers::{NOTE_MAX, REASONS, verify_review};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const PRECISION_SCHEMA: &str = "ce.eval-merge-precision/1.0.0";

fn rows(doc: &Value) -> &Vec<Value> {
    doc["rows"].as_array().expect("rows")
}

/// Every generation names its docs by its number, oldest first, and
/// keeps every doc of a record generation on disk.
#[test]
fn every_generation_names_its_docs() {
    for (i, g) in GENERATIONS.iter().enumerate() {
        assert_eq!(g.n as usize, i + 1, "generations are numbered in order");
        let docs = [g.set, g.sample, g.review, g.precision];
        let suffix = format!("-v{}.json", g.n);
        assert!(
            docs.iter().all(|d| d.ends_with(&suffix)),
            "generation {}: {docs:?}",
            g.n
        );
        if g.n != EXAM.n {
            assert!(
                docs.iter().all(|d| load(d).is_some()),
                "record {}: {docs:?}",
                g.n
            );
        }
    }
}

/// In every generation the sample holds exactly the draw of its frozen
/// set, in its order.
#[test]
fn the_sample_is_the_draw_of_the_frozen_set() {
    for g in &GENERATIONS {
        let (Some(set), Some(sample)) = (load(g.set), load(g.sample)) else {
            panic!("{} and {} are filed together", g.set, g.sample);
        };
        let drawn: Vec<String> = draw(&set)
            .iter()
            .map(|(c, r)| id_of(c, &r["members"]))
            .collect();
        let held: Vec<&str> = rows(&sample)
            .iter()
            .map(|r| r["id"].as_str().expect("id"))
            .collect();
        assert_eq!(held, drawn, "generation {}", g.n);
        let verdicts = ["feasible", "reason", "params", "savings"];
        for r in rows(&sample) {
            assert!(
                verdicts.iter().all(|k| r.get(*k).is_none()),
                "a sampled row shows no verdict"
            );
        }
    }
}

/// The readings: per corpus and overall, the feasibility, reason and
/// parameter-count agreements over the rows audited, from generation
/// g's three docs.
fn readings(g: &Generation, set: &Value, sample: &Value, review: &Value) -> Value {
    let core: BTreeMap<String, &Value> = set["corpora"]
        .as_array()
        .expect("corpora")
        .iter()
        .flat_map(|c| {
            rows(c)
                .iter()
                .map(move |r| (id_of(c["name"].as_str().expect("name"), &r["members"]), r))
        })
        .collect();
    let mut by: BTreeMap<String, [u64; 4]> = BTreeMap::new();
    for (s, a) in rows(sample).iter().zip(rows(review)) {
        assert_eq!(s["id"], a["id"], "the audit answers the sample in order");
        let c = core[s["id"].as_str().expect("id")];
        let feasible = a["feasible"].as_bool().expect("an audited feasibility");
        let reason = a["reason"].as_str().expect("an audited reason");
        assert!(
            REASONS.contains(&reason),
            "a reason is one of the six: {reason}"
        );
        let params = a["params"].as_u64().expect("an audited parameter count");
        for key in [s["corpus"].as_str().expect("corpus"), "all"] {
            let t = by.entry(key.to_string()).or_default();
            t[0] += u64::from(c["feasible"] == feasible);
            t[1] += u64::from(c["reason"] == reason);
            t[2] += u64::from(c["params"] == params);
            t[3] += 1;
        }
    }
    let shown: serde_json::Map<String, Value> = by
        .into_iter()
        .map(|(k, [f, r, p, n])| {
            let read =
                json!({"feasible_agree": f, "reason_agree": r, "params_agree": p, "rows": n});
            (k, read)
        })
        .collect();
    json!({"schema": PRECISION_SCHEMA, "from": [g.set, g.sample, g.review], "readings": shown})
}

/// In every generation the audit, once filed, is a review of the sample
/// (answers.rs) and answers every sampled row; the precision doc, once
/// filed, is what the three docs read.
#[test]
fn the_review_scores_when_filed() {
    for g in &GENERATIONS {
        let Some(review) = load(g.review) else {
            println!("eval_merge_review: {} not filed yet — skipped", g.review);
            continue;
        };
        let (set, sample) = (load(g.set).expect("set"), load(g.sample).expect("sample"));
        verify_review(&sample, &review);
        let read = readings(g, &set, &sample, &review);
        match load(g.precision) {
            Some(doc) => assert_eq!(doc, read, "{} reads what the docs read", g.precision),
            None => println!("eval_merge_review: {} not filed yet — skipped", g.precision),
        }
    }
}

fn rows_mut(doc: &mut Value) -> &mut Vec<Value> {
    doc["rows"].as_array_mut().expect("rows")
}

/// Row 0 answered a second time, at the end.
fn duplicate_first(doc: &mut Value) {
    let row = doc["rows"][0].clone();
    rows_mut(doc).push(row);
}

/// Row 0's feasibility flipped against its reason.
fn flip_feasible(doc: &mut Value) {
    let row = &mut doc["rows"][0];
    row["feasible"] = json!(row["feasible"] != json!(true));
}

/// The review's tamper battery: the pristine doc passes; a reason
/// outside the six, a parameter count that is no integer, a note past
/// NOTE_MAX and a dropped row refuse through the shared field-mutation
/// frame; an extra row, a swapped pair, a feasibility against its
/// reason, a forged batch count and an empty auditor through the
/// forgery frame. Every edit is on a copy: the filed doc is read again
/// after the battery and must be byte for byte what it was. The battery
/// reads the newest generation whose review is filed.
#[test]
fn the_review_refuses_tampering() {
    let filed = GENERATIONS.iter().rev().find(|g| load(g.review).is_some());
    let Some(g) = filed else {
        println!("eval_merge_review: no review filed yet — skipped");
        return;
    };
    let pristine = load(g.review).expect("the filed review");
    let before = std::fs::read(repo_file(g.review)).expect("the filed review");
    let sample = load(g.sample).expect("sample");
    let check = |doc: &Value| verify_review(&sample, doc);
    let long = "x".repeat(NOTE_MAX + 1);
    let mutations = [
        ("reason", "maybe", "a reason outside the six"),
        ("params", "3", "a parameter count that is no integer"),
        ("note", long.as_str(), "a note past its bound"),
    ];
    assert_tampering_refused(&pristine, &mutations, &check);
    let forgeries: [Forgery; 5] = [
        (&duplicate_first, "an extra row"),
        (&|d| rows_mut(d).swap(0, 1), "a swapped pair"),
        (&flip_feasible, "a feasibility against its reason"),
        (&|d| d["batches"] = json!(3), "a forged batch count"),
        (&|d| d["auditor"] = json!(""), "an empty auditor"),
    ];
    assert_forgeries_refused(&pristine, &forgeries, &check);
    let after = std::fs::read(repo_file(g.review)).expect("the filed review");
    assert!(
        after == before,
        "{}: the battery left it byte for byte",
        g.review
    );
}

#[test]
#[ignore = "reads the audit batches' answer files"]
fn assemble() {
    assert!(crate::facts::blessing(), "CE_BLESS=1 files the review");
    let (dir, auditor) = (env("CE_MERGE_BATCH_DIR"), env("CE_MERGE_AUDITOR"));
    let sample = load(EXAM.sample).unwrap_or_else(|| panic!("{}: not filed", EXAM.sample));
    let manifest = load_path(&format!("{dir}/manifest.json"));
    let file = |n: usize| std::fs::read_to_string(answer_file(&dir, n)).ok();
    let doc = answers::assemble(&sample, &manifest, &file, &auditor, generated_from())
        .unwrap_or_else(|errs| panic!("{} refusals:\n{}", errs.len(), errs.join("\n")));
    verify_review(&sample, &doc);
    write(EXAM.review, &doc);
    println!("{}: {} rows", EXAM.review, rows(&doc).len());
}

#[test]
#[ignore]
fn regenerate() {
    assert!(crate::facts::blessing(), "CE_BLESS=1 regenerates");
    let g = &EXAM;
    let review = load(g.review).unwrap_or_else(|| panic!("{}: the audit comes first", g.review));
    let read = readings(
        g,
        &load(g.set).expect("set"),
        &load(g.sample).expect("sample"),
        &review,
    );
    println!("{}", read["readings"]);
    write(g.precision, &read);
}
