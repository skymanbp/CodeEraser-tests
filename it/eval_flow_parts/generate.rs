//! The flow exams' generators — `#[ignore]`, because they read the
//! pinned corpus clone under `.ce-eval/corpora/<name>` (EXAMS pins the
//! tip; corpus::pinned_root refuses any other tree). Each writes one
//! frozen doc and refuses to overwrite it: a re-freeze is a new
//! generation and a named entry in docs/EVAL-SET-FLOW.md.
//!   CE_FLOW_LANG=python CE_FLOW_CORPUS=requests cargo test --test it -- --ignored eval_flow_parts::generate::flow_slice --nocapture
//!   CE_FLOW_LANG=python cargo test --test it -- --ignored eval_flow_parts::generate::flow_sample --nocapture

use super::{FlowExam, SAMPLES, SCHEMAS, SLICES, draw, exam, pools, slice_summary, source_row};
use crate::eval_lang_parts::Generated;
use crate::eval_lang_parts::freezing::{frozen, walked};
use crate::eval_lang_parts::generate::{blob, corpus_repo, env, freeze};
use crate::eval_support::content_sha;
use codeeraser::flow::lower::{Lowered, Unit, lower_file};
use codeeraser::scan::lang::Lang;
use serde_json::{Value, json};

/// One file lowered — a flow language by construction (the exam's
/// extensions are the language's).
pub fn lowered(text: &str, lang: Lang) -> Lowered {
    lower_file(text, lang).expect("a flow language")
}

/// One universe row: the file's content identity, each unit's place,
/// size and pools, and each unit the lowering left out with its
/// reason.
pub fn file_row(path: &str, text: &str, done: &Lowered) -> Value {
    let units: Vec<Value> = done.units.iter().map(unit_row).collect();
    let unlowered: Vec<Value> = done
        .unlowered
        .iter()
        .map(|u| json!({"nth": u.nth, "name": u.name, "reason": u.reason}))
        .collect();
    json!({
        "path": path, "sha256": content_sha(text), "units": units, "unlowered": unlowered,
    })
}

fn unit_row(u: &Unit) -> Value {
    json!({
        "nth": u.nth, "name": u.name, "lines": [u.start_line, u.end_line],
        "params": u.params, "dynamic": u.dynamic,
        "rows": [u.stmts.len(), u.vars.len(), u.uses.len()],
        "pools": pools::counts(u),
    })
}

const SLICE_METHOD: &str = "unit universe of one pinned corpus for one v2.31 flow language: \
    every file whose extension is in scope and that the product's own walk reads at the pinned \
    tip (a tracked file the walk refuses is tallied, never asked about), each unit scan extracts \
    lowered into the flow family's four tables (flow::lower - grammar tables and file-local \
    facts only, the core never asked), inventoried with the sha256 of the text, the unit's \
    place and size, and its candidate pool per (kind, stratum) cell read off those tables \
    alone (pools.rs); a unit the lowering could not shape is listed with its reason. Frozen \
    before any verdict is read; the constants are pre-registered here.";

#[test]
#[ignore = "reads the pinned corpus clone"]
fn flow_slice() {
    let exam = exam(&env("CE_FLOW_LANG"));
    let name = env("CE_FLOW_CORPUS");
    let tip = exam
        .tip(&name)
        .unwrap_or_else(|| panic!("{name}: no corpus of the {} exam", exam.lang));
    let lang = exam.language();
    let row = |path: &str, text: &str| file_row(path, text, &lowered(text, lang));
    let (files, excluded) = walked(&corpus_repo(&name, tip), tip, &|p| exam.in_scope(p), &row);
    let summary = slice_summary(&files);
    println!("{}: summary {summary}", exam.key(&name));
    let doc = json!({
        "corpus": {"name": name, "tip": tip, "lang": exam.lang},
        "scope": exam.scope(),
        "summary": summary,
        "excluded": excluded,
        "files": files,
    });
    let doc = frozen(SCHEMAS.0, SLICE_METHOD, super::slice_constants(), doc);
    freeze(&SLICES.file(exam, &exam.key(&name)), &doc);
}

/// Every pool item of one frozen universe, re-lowered at the pinned
/// tip, each file reproducing its frozen row first — the pool equals
/// the frozen universe by checked reconstruction, not by trust.
fn corpus_pool(exam: &FlowExam, name: &str, tip: &str, slice: &Value) -> Vec<Value> {
    let (repo, lang) = (corpus_repo(name, tip), exam.language());
    let mut pool = Vec::new();
    for row in slice["files"].as_array().expect("files") {
        let path = row["path"].as_str().expect("path");
        let text = blob(&repo, tip, path);
        let done = lowered(&text, lang);
        assert_eq!(
            &file_row(path, &text, &done),
            row,
            "{name}/{path}: not the frozen row"
        );
        for u in &done.units {
            for item in pools::items(u) {
                pool.push(json!({
                    "corpus": name, "commit": tip, "path": path, "unit": u.nth,
                    "unit_name": u.name, "unit_lines": [u.start_line, u.end_line],
                    "kind": item.kind, "stratum": item.stratum.to_string(),
                    "line": item.line, "nth": item.nth, "name": item.name,
                }));
            }
        }
    }
    pool
}

const SAMPLE_METHOD: &str = "hash-ranked stratified draw over the language's frozen unit \
    universes, reconstructed file by file against their frozen rows. rank id = \
    sha256(domain|corpus|commit|path|unit|kind|stratum|line|nth|name). Per (kind, stratum) \
    cell the min(min_per_stratum, pool) lowest site-domain ranks; rows stored in audit-domain \
    order; no backups. The stratum stays in this doc for the gate to re-rank by - it is a \
    source fact, not an answer - and the batch handed to an auditor carries neither the \
    stratum nor any product answer.";

#[test]
#[ignore = "reads every pinned corpus clone of the language"]
fn flow_sample() {
    let exam = exam(&env("CE_FLOW_LANG"));
    let (mut pool, mut sources) = (Vec::new(), Vec::new());
    for (name, tip) in &exam.corpora {
        let slice = SLICES.load(exam, &exam.key(name));
        pool.extend(corpus_pool(exam, name, tip, &slice));
        sources.push(source_row(name, &slice));
    }
    let (allocation, rows) = draw::draw(&pool);
    println!(
        "{}: pool {} rows {} allocation {allocation:?}",
        exam.lang,
        pool.len(),
        rows.len()
    );
    let doc = json!({
        "lang": exam.lang,
        "sources": sources,
        "allocation": allocation,
        "rows": rows,
    });
    let doc = frozen(SCHEMAS.1, SAMPLE_METHOD, super::sample_constants(), doc);
    freeze(&SAMPLES.file(exam, exam.lang), &doc);
}
