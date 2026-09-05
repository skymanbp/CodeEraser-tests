//! Explain byte drift without admitting normalized text to the scored cohort.
use super::data::Oracle;
use crate::{eval_support::content_sha, similar_replay::Measured};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub fn drift(m: &Measured, oracle: &Oracle) -> Value {
    let mut files = BTreeMap::new();
    for r in oracle.rows.iter().filter(|r| r.corpus == m.name) {
        for id in std::iter::once(&r.id).chain(r.candidates.iter().map(|c| &c.id)) {
            files.entry(id.path.as_str()).or_insert(id.sha.as_str());
        }
    }
    let mut counts = BTreeMap::<&str, usize>::new();
    let rows: Vec<_> = files
        .iter()
        .map(|(path, expected)| {
            let text = m.texts.get(*path);
            let actual = text.map(|s| content_sha(s));
            let kind = if actual.as_deref() == Some(*expected) {
                "exact"
            } else if text.is_some_and(|s| content_sha(&s.replace("\r\n", "\n")) == *expected) {
                "eol_only"
            } else {
                "content_or_missing"
            };
            *counts.entry(kind).or_default() += 1;
            json!({"path": path, "kind": kind, "expected": expected, "actual": actual})
        })
        .collect();
    json!({"corpus": m.name, "counts": counts, "files": rows})
}
