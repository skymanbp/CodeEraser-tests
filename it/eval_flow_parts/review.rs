//! The flow exams' blind review (booklet §5.5 「盲判」): one language's
//! answers, assembled verbatim from its batches' answer files
//! (answers.rs) into `flow-review-<lang>-v<g>.json`, and the verifier
//! the gate (eval_flow_review.rs) and the tamper battery run on it —
//! one row per sampled question in the sample's audit order, echoing
//! the sampled identity field for field, its batch the plan's
//! (batches.rs), its truth one of its kind's words or `cannot_tell`,
//! its why within the reason bounds, the summary re-derived.
//!   CE_FLOW_LANG=python CE_FLOW_BATCH_DIR=<dir> CE_FLOW_AUDITOR="<one sentence>" [CE_FLOW_REVIEW_DRY=1] cargo test --test it -- --ignored eval_flow_parts::review::flow_review_assemble --nocapture
//! A dry run reads, checks and prints the summary and writes nothing.

use super::answers::{self, reason_fits};
use super::batches::{CANNOT_TELL, answer_file, legal, plan, words};
use super::draw::text;
use super::prompt::readings;
use super::{FlowExam, exam};
use crate::eval_lang_parts::Docs;
use crate::eval_lang_parts::generate::{env, freeze};
use crate::eval_support::{MIN_WHY, generated_from, load};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const REVIEWS: Docs = Docs("flow-review");
pub const REVIEW_SCHEMA: &str = "ce.eval-flow-review/1.0.0";

/// The sampled identity a review row echoes, field for field; the row
/// adds `batch`, `truth` and `why`.
pub const IDENTITY: &str =
    "rank audit corpus commit path unit unit_name unit_lines kind stratum line nth name";

/// The exam's corpora as the review names them.
pub fn corpora(exam: &FlowExam) -> Value {
    let rows: Vec<Value> = exam
        .corpora
        .iter()
        .map(|(c, tip)| json!({"corpus": c, "tip": tip}))
        .collect();
    json!(rows)
}

/// Per kind (as a string), the count of each answer word.
pub fn summary(rows: &[Value]) -> Value {
    let mut out: BTreeMap<String, BTreeMap<&str, u64>> = BTreeMap::new();
    for row in rows {
        let kind = row["kind"].as_u64().expect("kind");
        let counts = out.entry(kind.to_string()).or_insert_with(|| {
            let [a, b] = words(kind);
            [a, b, CANNOT_TELL].into_iter().map(|w| (w, 0)).collect()
        });
        *counts.entry(text(row, "truth")).or_insert(0) += 1;
    }
    json!(out)
}

/// The manifest must be this sample's plan: language, generation, the
/// readings its batches were rendered under (absent at the first
/// generation) and every batch's number, corpus and ids.
fn manifest_agrees(exam: &FlowExam, sample: &Value, manifest: &Value) -> bool {
    let batches: Vec<Value> = plan(exam, sample)
        .iter()
        .map(|b| {
            let ids: Vec<&str> = b.rows.iter().map(|r| text(r, "audit")).collect();
            json!({"n": b.n, "corpus": b.corpus, "ids": ids})
        })
        .collect();
    let theirs: Vec<Value> = manifest["batches"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|b| json!({"n": b["n"], "corpus": b["corpus"], "ids": b["ids"]}))
                .collect()
        })
        .unwrap_or_default();
    manifest["lang"] == json!(exam.lang)
        && manifest["generation"] == json!(exam.generation)
        && manifest["readings"] == json!(readings(exam))
        && theirs == batches
}

/// A language's batch directory as read: its manifest, and `file(n)`
/// batch n's answer file (None when absent).
pub struct Filed<'a> {
    pub manifest: &'a Value,
    pub file: &'a dyn Fn(u64) -> Option<String>,
}

/// The review doc of a language, or every refusal; `stamp` is the
/// provenance stamp.
pub fn assemble(
    exam: &FlowExam,
    sample: &Value,
    filed: Filed,
    auditor: &str,
    stamp: Value,
) -> Result<Value, Vec<String>> {
    let Filed { manifest, file } = filed;
    if !manifest_agrees(exam, sample, manifest) {
        return Err(vec!["manifest: not this sample's batch plan".to_string()]);
    }
    let mut errs = Vec::new();
    if auditor.chars().count() < MIN_WHY {
        errs.push(format!("auditor: under {MIN_WHY} characters"));
    }
    let mut judged: BTreeMap<String, Value> = BTreeMap::new();
    for b in plan(exam, sample) {
        for (id, a) in answers::read(&b, file(b.n).as_deref(), &mut errs) {
            let extra = json!({"batch": b.n, "truth": a.truth, "why": a.reason});
            judged.insert(id, extra);
        }
    }
    if !errs.is_empty() {
        return Err(errs);
    }
    let rows: Vec<Value> = sample["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|s| review_row(s, &judged[text(s, "audit")]))
        .collect();
    let mut doc = json!({
        "schema": REVIEW_SCHEMA, "lang": exam.lang, "generation": exam.generation,
        "corpora": corpora(exam), "auditor": auditor, "summary": summary(&rows),
        "rows": rows, "notes": [], "generated_from": stamp,
    });
    if let Some(r) = readings(exam) {
        doc["readings"] = json!(r);
    }
    Ok(doc)
}

/// A sample row's identity with the answer's three fields.
fn review_row(sampled: &Value, answer: &Value) -> Value {
    let mut row = Map::new();
    for f in IDENTITY.split(' ').chain(["batch", "truth", "why"]) {
        let from = if sampled.get(f).is_some() {
            sampled
        } else {
            answer
        };
        row.insert(f.to_string(), from[f].clone());
    }
    Value::Object(row)
}

/// The envelope: schema, language, generation, the exam's corpora at
/// their tips, the readings its questions were judged under (2 from
/// the second generation on; absent or 1 at the first, whose prompt
/// had no readings section), an auditor sentence, notes as sentences,
/// a stamp.
fn check_envelope(exam: &FlowExam, doc: &Value) {
    let lang = exam.lang;
    let want = json!({
        "schema": REVIEW_SCHEMA, "lang": lang, "generation": exam.generation,
        "corpora": corpora(exam),
    });
    super::assert_keys(lang, doc, &want);
    let read = &doc["readings"];
    let fits = match readings(exam) {
        Some(r) => *read == json!(r),
        None => read.is_null() || *read == json!(1),
    };
    assert!(fits, "{lang}: readings {read} is not the generation's");
    let auditor = doc["auditor"].as_str().map_or(0, |a| a.chars().count());
    assert!(auditor >= MIN_WHY, "{lang}: no auditor sentence");
    let notes = doc["notes"].as_array().expect("notes");
    assert!(
        notes.iter().all(Value::is_string),
        "{lang}: a note that is no sentence"
    );
    assert!(
        doc["generated_from"]["ce"].is_string(),
        "{lang}: no provenance stamp"
    );
}

pub fn row_list(doc: &Value) -> &Vec<Value> {
    doc["rows"].as_array().expect("rows")
}

/// One review against its exam and frozen sample.
pub fn verify_review(exam: &FlowExam, sample: &Value, doc: &Value) {
    check_envelope(exam, doc);
    let lang = exam.lang;
    let (sampled, rows) = (row_list(sample), row_list(doc));
    assert_eq!(rows.len(), sampled.len(), "{lang}: review row count");
    let batch: BTreeMap<&str, u64> = plan(exam, sample)
        .iter()
        .flat_map(|b| b.rows.iter().map(move |r| (text(r, "audit"), b.n)))
        .collect();
    let keys: BTreeSet<&str> = IDENTITY
        .split(' ')
        .chain(["batch", "truth", "why"])
        .collect();
    for (row, s) in rows.iter().zip(sampled) {
        let at = format!("{lang}/{}", text(s, "audit"));
        let have: BTreeSet<&str> = row
            .as_object()
            .expect("row")
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(have, keys, "{at}: fields");
        for f in IDENTITY.split(' ') {
            assert_eq!(row[f], s[f], "{at}: {f} is not the sampled one");
        }
        assert_eq!(
            row["batch"],
            json!(batch[text(s, "audit")]),
            "{at}: not its batch"
        );
        let kind = row["kind"].as_u64().expect("kind");
        assert!(
            legal(kind, text(row, "truth")),
            "{at}: truth out of the kind's words"
        );
        assert!(
            reason_fits(text(row, "why")),
            "{at}: why outside the reason bounds"
        );
    }
    assert_eq!(doc["summary"], summary(rows), "{lang}: summary drifted");
}

#[test]
#[ignore = "reads the audit batches' answer files"]
fn flow_review_assemble() {
    let exam = exam(&env("CE_FLOW_LANG"));
    let (dir, auditor) = (env("CE_FLOW_BATCH_DIR"), env("CE_FLOW_AUDITOR"));
    let sample = exam.sample();
    let manifest = load(&format!("{dir}/{}/manifest.json", exam.lang));
    let file = |n: u64| std::fs::read_to_string(answer_file(&dir, exam.lang, n)).ok();
    let filed = Filed {
        manifest: &manifest,
        file: &file,
    };
    let doc = assemble(exam, &sample, filed, &auditor, generated_from())
        .unwrap_or_else(|errs| panic!("{} refusals:\n{}", errs.len(), errs.join("\n")));
    verify_review(exam, &sample, &doc);
    println!("{}: summary {}", exam.lang, doc["summary"]);
    if std::env::var_os("CE_FLOW_REVIEW_DRY").is_some() {
        println!("dry run: nothing written");
        return;
    }
    freeze(&REVIEWS.file(exam, exam.lang), &doc);
}
