//! The replay's docs (G14): one per corpus,
//! `<OUT>/deadfile-gen2-replay-v1.<corpus>.json`, written when that
//! corpus's replay ends, so a run resumes; each carries the corpus's tip
//! and the product record (G6). Each event says where its fate was
//! decided (G15); each unjudged tree is listed with its commit, exit code
//! and reason (G3, G5).

use super::model::{Event, Followed, History, Sides, window};
use super::words::{KINDS, STATES, fate_name, word};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub const SCHEMA: &str = "ce.eval-deadfile-gen2-replay/2.0.0";

pub fn doc_path(out: &Path, corpus: &str) -> PathBuf {
    out.join(format!("deadfile-gen2-replay-v1.{corpus}.json"))
}

/// Whether the corpus's doc is already written for this tip and product.
pub fn done(out: &Path, corpus: &str, tip: &str, product: &Value) -> bool {
    let path = doc_path(out, corpus);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return false;
    };
    let doc: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    doc["tip"] == tip && doc["product"] == *product
}

fn event(h: &History, e: &Event, f: &Followed) -> Value {
    json!({"commit": h.shas[e.at], "path": e.path,
           "state": word(&STATES, e.state), "kind": word(&KINDS, e.kind),
           "fate": fate_name(f.fate),
           "decided_at": f.decided_at.map(|k| h.shas[k].clone()),
           "path_then": f.path_then})
}

/// One corpus's doc from its history, events and side numbers.
pub fn corpus_doc(
    (corpus, tip, product): (&str, &str, &Value),
    h: &History,
    events: &[(Event, Followed)],
    s: &Sides,
) -> Value {
    let w = window(h.trees.len());
    let unjudged: Vec<Value> = h
        .trees
        .iter()
        .zip(&h.shas)
        .filter(|(t, _)| !t.judged)
        .map(|(t, sha)| json!({"commit": sha, "reason": t.why}))
        .collect();
    json!({"schema": SCHEMA, "corpus": corpus, "tip": tip, "product": product,
           "commits": h.shas, "window": [w.start, w.end],
           "unjudged_trees": unjudged,
           "unjudged_pairs": s.unjudged_pairs, "unjudged_after_window": s.unjudged_after,
           "met_unjudged": s.met_unjudged, "dependents": s.dependents,
           "symlink_rows": s.symlink_rows,
           "events": events.iter().map(|(e, f)| event(h, e, f)).collect::<Vec<_>>()})
}

pub fn write(out: &Path, doc: &Value) {
    let corpus = doc["corpus"].as_str().expect("corpus");
    let text = serde_json::to_string_pretty(doc).expect("doc") + "\n";
    std::fs::write(doc_path(out, corpus), text).expect("write");
}

/// The per-corpus docs of a run, checked against the registry's chosen
/// rows: each present exactly once, every product record the same.
pub fn checked(docs: &[Value], chosen: &[&str]) -> Result<(), String> {
    for name in chosen {
        let n = docs.iter().filter(|d| d["corpus"] == *name).count();
        if n != 1 {
            return Err(format!("{name}: {n} docs, want exactly one"));
        }
    }
    if let Some(d) = docs
        .iter()
        .find(|d| !chosen.contains(&d["corpus"].as_str().unwrap_or("")))
    {
        return Err(format!("{}: a doc for no chosen row", d["corpus"]));
    }
    let first = docs.first().map(|d| (&d["corpus"], &d["product"]));
    match first.and_then(|(c, p)| docs.iter().find(|d| d["product"] != *p).map(|d| (c, d))) {
        Some((c, d)) => Err(format!("{}: another product record than {c}", d["corpus"])),
        None => Ok(()),
    }
}

/// Every per-corpus doc under `out`.
pub fn read_all(out: &Path) -> Vec<Value> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(out)
        .unwrap_or_else(|e| panic!("{}: {e}", out.display()))
        .map(|e| e.expect("entry").path())
        .filter(|p| {
            let n = p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            n.starts_with("deadfile-gen2-replay-v1.") && n.ends_with(".json")
        })
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|p| crate::eval_support::load(&p.to_string_lossy()))
        .collect()
}
