//! The frozen-sample verifier of the v2.30 language exams (split from
//! eval_lang.rs, whose gates call it — the tamper gate runs it on
//! forged copies, so it lives apart from the tests that own the
//! verdicts): a sample re-derived from its frozen slices, row by row.

use crate::eval_lang_parts::{self as parts, Exam};
use crate::eval_support::{eval_doc, identity_hash, load};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// The frozen slices of one exam, in EXAMS corpus order.
fn slices(exam: &Exam) -> Vec<(&'static str, Value)> {
    exam.corpora
        .iter()
        .map(|(name, _)| (*name, load(&eval_doc(&format!("lang-slice-{name}")))))
        .collect()
}

/// Per-kind pool counts re-derived from the frozen slice summaries —
/// the gate-side twin of the generator's live pool.
fn pool_counts(exam: &Exam) -> BTreeMap<String, u64> {
    let mut pool = BTreeMap::new();
    for (_, slice) in slices(exam) {
        for (cell, n) in slice["summary"]["sites_by"].as_object().expect("sites_by") {
            let kind = cell
                .strip_prefix(&format!("{}/", exam.lang))
                .expect("lang/kind");
            *pool.entry(kind.to_string()).or_insert(0) += n.as_u64().expect("n");
        }
    }
    pool
}

fn kind_of(row: &Value) -> &str {
    row["kind"].as_str().expect("kind")
}

/// One frozen row: both hashes re-derive from its own fields, its
/// identity is the exam's (language, corpus, pinned commit), no rank
/// repeats, and its site is bound to a frozen file that holds its
/// kind — never more sampled sites of a kind than the file has.
fn check_row(
    exam: &Exam,
    files: &BTreeMap<(String, String), Value>,
    row: &Value,
    seen: &mut BTreeSet<String>,
    bound: &mut BTreeMap<(String, String, String), u64>,
) {
    let rank = row["rank"].as_str().expect("rank");
    assert_eq!(
        rank,
        identity_hash(parts::SITE_DOMAIN, row, &parts::FIELDS),
        "rank forged"
    );
    assert_eq!(
        row["audit"].as_str().expect("audit"),
        identity_hash(parts::AUDIT_DOMAIN, row, &parts::FIELDS),
        "{rank}: audit hash forged"
    );
    assert!(seen.insert(rank.to_string()), "{rank}: sampled twice");
    assert_eq!(row["lang"], json!(exam.lang), "{rank}: language");
    let corpus = row["corpus"].as_str().expect("corpus");
    let tip = exam
        .corpora
        .iter()
        .find(|(c, _)| *c == corpus)
        .map(|(_, t)| *t)
        .unwrap_or_else(|| panic!("{rank}: {corpus} is no corpus of the exam"));
    assert_eq!(row["commit"], json!(tip), "{rank}: not the pinned commit");
    let path = row["path"].as_str().expect("path").to_string();
    let file = &files[&(corpus.to_string(), path.clone())];
    let key = (corpus.to_string(), path, kind_of(row).to_string());
    let n = bound.entry(key).or_insert(0);
    *n += 1;
    assert!(
        file["sites"][kind_of(row)]
            .as_u64()
            .is_some_and(|have| *n <= have),
        "{rank}: more {} sites sampled than its frozen file holds",
        kind_of(row)
    );
}

/// The per-kind tally of a row list.
fn per_kind(rows: &[Value]) -> BTreeMap<String, u64> {
    let mut by = BTreeMap::new();
    for row in rows {
        *by.entry(kind_of(row).to_string()).or_insert(0) += 1;
    }
    by
}

/// One frozen sample against its frozen slices: envelope, sources,
/// the allocation re-run from the slices' per-kind pools, the rows
/// (check_row), per-kind counts equal to the allocation, primaries in
/// audit order, each kind's backups sized min(BACKUP_PER_KIND, pool −
/// quota) in (kind, audit) order and every one ranked below every
/// primary of its kind — the primaries were the top of the rank.
pub fn verify_sample(exam: &Exam, doc: &Value) {
    let lang = exam.lang;
    assert_eq!(doc["schema"], json!(parts::SAMPLE_SCHEMA), "{lang}: schema");
    assert_eq!(doc["lang"], json!(lang), "{lang}: language");
    assert_eq!(
        doc["constants"],
        parts::sample_constants(),
        "{lang}: constants"
    );
    let slices = slices(exam);
    let sources: Vec<Value> = slices
        .iter()
        .map(|(n, s)| parts::source_row(n, s))
        .collect();
    assert_eq!(doc["sources"], json!(sources), "{lang}: sources drifted");
    let pool = pool_counts(exam);
    let allocation = parts::quotas(&pool);
    assert_eq!(
        doc["allocation"],
        json!(allocation),
        "{lang}: allocation drifted"
    );
    let mut files = BTreeMap::new();
    for (name, slice) in &slices {
        for f in slice["files"].as_array().expect("files") {
            let path = f["path"].as_str().expect("path").to_string();
            files.insert((name.to_string(), path), f.clone());
        }
    }
    let rows = doc["rows"].as_array().expect("rows");
    let backups = doc["backups"].as_array().expect("backups");
    let (mut seen, mut bound) = (BTreeSet::new(), BTreeMap::new());
    for row in rows.iter().chain(backups) {
        check_row(exam, &files, row, &mut seen, &mut bound);
    }
    assert_eq!(per_kind(rows), allocation, "{lang}: per-kind counts");
    let audit = |r: &Value| r["audit"].as_str().expect("audit").to_string();
    let rank = |r: &Value| r["rank"].as_str().expect("rank").to_string();
    assert!(
        rows.is_sorted_by_key(audit),
        "{lang}: primaries not in audit order"
    );
    let backup_key = |r: &Value| (kind_of(r).to_string(), audit(r));
    assert!(
        backups.is_sorted_by_key(backup_key),
        "{lang}: backups out of order"
    );
    let bk = per_kind(backups);
    for (kind, n) in &pool {
        let want = (n - allocation[kind]).min(parts::BACKUP_PER_KIND);
        assert_eq!(
            bk.get(kind).copied().unwrap_or(0),
            want,
            "{lang}/{kind}: backups"
        );
        let top = rows.iter().filter(|r| kind_of(r) == kind).map(rank).max();
        let next = backups
            .iter()
            .filter(|r| kind_of(r) == kind)
            .map(rank)
            .min();
        if let (Some(top), Some(next)) = (top, next) {
            assert!(top < next, "{lang}/{kind}: a backup outranks a primary");
        }
    }
}
