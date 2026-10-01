//! The clone-merge audit's batches (plan v2.31 step 7, booklet 18 §6
//! "The audit"): the frozen sample cut into BATCHES batches of
//! BATCH_SIZE an independent judge reads alone. The rows are ordered by
//! a second hash of their audit id (the question's identity) under the
//! batches' own domain and cut in that order: the draw took each half's
//! lowest ids, so the ids alone cluster the feasible half at the high
//! end (ordered by id, one batch read 25 of 25 feasible) — rehashed,
//! no batch's place says which half a row came from. It lives beside
//! eval_merge_parts, not inside it: a child that reads its parent sits in
//! the parent's import cycle. A question shows the corpus, the family,
//! whether it is a fragment, every member's identity, run and source,
//! and the core's parameterisation (`param_texts`, which the judge may
//! reject); never the feasibility, the reason, the savings or the member
//! kept. The plan is one pure function of the sample, so the manifest
//! written beside the batches is a convenience, not a source.
//!   CE_MERGE_BATCH_DIR=<dir> cargo test --test it -- --ignored eval_merge_batches::merge_batches --nocapture

mod prompt;

use crate::eval_flow_parts::batches::put;
use crate::eval_lang_parts::generate::env;
use crate::eval_merge_parts::load;
use crate::eval_merge_parts::sample::SAMPLE;
use crate::eval_support::identity_hash;
use prompt::{PROMPT_ANSWER, PROMPT_HEAD};
use serde_json::{Value, json};
use std::path::Path;

/// The batches' own hash domain (the order a judge reads in).
const DOMAIN: &str = "merge-batches-v1";

/// Four judges, twenty-five questions each.
pub const BATCHES: usize = 4;
pub const BATCH_SIZE: usize = 25;

/// The sample's rows in batch order (rehashed id), cut into batches.
pub fn plan(sample: &Value) -> Vec<Vec<&Value>> {
    let mut rows: Vec<&Value> = sample["rows"].as_array().expect("rows").iter().collect();
    assert_eq!(
        rows.len(),
        BATCHES * BATCH_SIZE,
        "{SAMPLE}: the sample is the batches' size"
    );
    rows.sort_by_key(|r| identity_hash(DOMAIN, r, &["id"]));
    rows.chunks(BATCH_SIZE).map(<[&Value]>::to_vec).collect()
}

/// A fence longer than any run of backticks in the text.
fn fence(text: &str) -> String {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    "`".repeat(longest.max(2) + 1)
}

/// One member as a question shows it: identity, run, source.
fn member(m: usize, row: &Value) -> String {
    let src = row["source"].as_str().expect("source");
    let f = fence(src);
    let run = &row["run"];
    format!(
        "member {m}: {}   run: lines {}-{}\n{f}\n{src}\n{f}",
        row["at"].as_str().expect("at"),
        run[0],
        run[1]
    )
}

/// The core's parameters, each with every member's text.
fn params(texts: &[Value]) -> String {
    if texts.is_empty() {
        return "parameters the tool found: none".to_string();
    }
    let lines: Vec<String> = texts
        .iter()
        .enumerate()
        .map(|(p, t)| {
            let each: Vec<String> = t
                .as_array()
                .expect("texts")
                .iter()
                .enumerate()
                .map(|(m, s)| format!("member {m}: {s}"))
                .collect();
            format!("  p{p}: {}", each.join(" · "))
        })
        .collect();
    format!(
        "parameters the tool found: {}\n{}",
        texts.len(),
        lines.join("\n")
    )
}

/// One question: the group's head, its members, the parameters.
fn question(row: &Value) -> String {
    let members: Vec<String> = row["members"]
        .as_array()
        .expect("members")
        .iter()
        .enumerate()
        .map(|(m, r)| member(m, r))
        .collect();
    let field = |k: &str| row[k].as_str().unwrap_or_else(|| panic!("{k}"));
    let [id, corpus, family] = ["id", "corpus", "family"].map(field);
    let texts = row["param_texts"].as_array().expect("param_texts");
    format!(
        "### {id}\ncorpus: {corpus}   family: {family}   fragment: {}\n{}\n{}",
        row["fragment"],
        members.join("\n"),
        params(texts)
    )
}

/// Where a batch's answers go.
pub fn answer_file(dir: &str, n: usize) -> String {
    format!("{dir}/answers-{n}.jsonl")
}

/// Every file of the batches, `(name under <dir>, text)`: `batch-<n>.md`
/// each and `manifest.json`.
pub fn render(sample: &Value, dir: &str) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let mut manifest = Vec::new();
    for (i, rows) in plan(sample).into_iter().enumerate() {
        let n = i + 1;
        let questions: Vec<String> = rows.iter().map(|r| question(r)).collect();
        let head = PROMPT_HEAD.replace("{BATCH_ID}", &format!("merge-{n}"));
        let answer = PROMPT_ANSWER
            .replace("{ANSWER_FILE}", &answer_file(dir, n))
            .replace("{QUESTIONS}", &questions.join("\n\n"));
        files.push((format!("batch-{n}.md"), format!("{head}\n{answer}")));
        let ids: Vec<&Value> = rows.iter().map(|r| &r["id"]).collect();
        manifest.push(json!({"n": n, "ids": ids, "answers": answer_file(dir, n)}));
    }
    let doc = json!({"sample": SAMPLE, "batches": manifest});
    let text = serde_json::to_string_pretty(&doc).expect("json") + "\n";
    files.push(("manifest.json".to_string(), text));
    files
}

#[test]
#[ignore = "writes the audit batches outside the tree"]
fn merge_batches() {
    let dir = env("CE_MERGE_BATCH_DIR");
    let sample = load(SAMPLE).unwrap_or_else(|| panic!("{SAMPLE}: not filed"));
    std::fs::create_dir_all(&dir).expect("batch dir");
    for (name, text) in render(&sample, &dir) {
        put(&Path::new(&dir).join(&name), &text);
        let questions = text.matches("\n### ").count();
        println!("{dir}/{name}: {questions} questions");
    }
}

/// The plan is total and blind: every sampled row in exactly one batch,
/// and no rendered batch names a verdict field.
#[test]
fn the_batches_cover_the_sample_blind() {
    let sample = load(SAMPLE).unwrap_or_else(|| panic!("{SAMPLE}: not filed"));
    let mut seen: Vec<&str> = plan(&sample)
        .iter()
        .flatten()
        .map(|r| r["id"].as_str().expect("id"))
        .collect();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), BATCHES * BATCH_SIZE, "each row once");
    for (name, text) in render(&sample, "<dir>") {
        let blind = ["\"savings\"", "\"kept\"", "savings:", "kept:"];
        assert!(
            blind.iter().all(|w| !text.contains(w)),
            "{name}: a batch shows no verdict"
        );
    }
}
