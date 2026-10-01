//! The flow exams' blind-audit batches (booklet §5.5 「盲判」): a
//! language's frozen sample cut into batches an independent auditor
//! reads — one corpus per batch (one clone to read), at most BATCH_MAX
//! questions, in the sample's audit order — each rendered from the
//! prompt (prompt.rs) with its questions. A question names the file, the
//! function and its line range, and the anchor (statement, write,
//! declaration or parameter); never the stratum, the rank or any
//! product answer. The plan is one pure function of the sample, read by
//! the renderer, the assembler (review.rs) and the verifier, so the
//! manifest written beside the batches is a convenience, not a source.
//!   CE_FLOW_LANG=python CE_FLOW_BATCH_DIR=<dir> [CE_FLOW_CLONE_ROOT=<corpora>] cargo test --test it -- --ignored eval_flow_parts::batches::flow_batches --nocapture

use super::draw::text;
pub use super::prompt::READINGS_TEXT;
use super::prompt::{PROMPT, readings};
use super::{FlowExam, exam};
use crate::eval_lang_parts::generate::env;
use crate::eval_support::pinned_at;
use serde_json::{Value, json};
use std::path::Path;

/// At most this many questions in one batch.
pub const BATCH_MAX: usize = 25;

/// Each kind's two answer words (the first is the finding's), one text;
/// `cannot_tell` is every kind's third.
const WORDS: &str = "0 unreachable reachable\n1 dead live\n2 unread read\n3 unread read";
pub const CANNOT_TELL: &str = "cannot_tell";

/// A kind's two answer words.
pub fn words(kind: u64) -> [&'static str; 2] {
    let line = WORDS
        .lines()
        .find(|l| l.starts_with(&format!("{kind} ")))
        .unwrap_or_else(|| panic!("kind {kind}: no answer words"));
    let w: Vec<&str> = line.split(' ').skip(1).collect();
    [w[0], w[1]]
}

/// Whether a truth is legal for a kind.
pub fn legal(kind: u64, truth: &str) -> bool {
    truth == CANNOT_TELL || words(kind).contains(&truth)
}

/// One batch: its number (1-based over the language), its corpus and
/// its sample rows in audit order.
pub struct Batch<'a> {
    pub n: u64,
    pub corpus: &'static str,
    pub rows: Vec<&'a Value>,
}

/// The sample cut into batches: corpus by corpus in the exam's order,
/// each corpus's rows in the sample's (audit) order, BATCH_MAX a batch.
pub fn plan<'a>(exam: &FlowExam, sample: &'a Value) -> Vec<Batch<'a>> {
    let rows = sample["rows"].as_array().expect("rows");
    let mut out: Vec<Batch> = Vec::new();
    for (corpus, _) in &exam.corpora {
        let mine: Vec<&Value> = rows.iter().filter(|r| r["corpus"] == *corpus).collect();
        for chunk in mine.chunks(BATCH_MAX) {
            let n = out.len() as u64 + 1;
            out.push(Batch {
                n,
                corpus,
                rows: chunk.to_vec(),
            });
        }
    }
    out
}

/// A question's sentence by kind. The frozen nth counts from 1; the
/// prompt counts from 0, so the sentence says nth - 1.
fn sentence(row: &Value) -> String {
    let (line, name) = (&row["line"], text(row, "name"));
    let n = row["nth"].as_u64().expect("nth") - 1;
    match row["kind"].as_u64().expect("kind") {
        0 => format!(
            "the statement starting on line {line}, the {n}-th statement starting on that line (from 0)"
        ),
        1 => format!(
            "the write to {name} on line {line}, the {n}-th write to {name} on that line (from 0)"
        ),
        2 => format!(
            "the local variable {name} declared on line {line}, the {n}-th declaration of {name} on that line (from 0)"
        ),
        3 => format!("the parameter {name}"),
        k => panic!("kind {k}: no question"),
    }
}

/// One question as the batch shows it.
fn question(row: &Value) -> String {
    let kind = row["kind"].as_u64().expect("kind");
    let [a, b] = words(kind);
    let lines = &row["unit_lines"];
    format!(
        "### {}\nfile: {}   function: {}   lines: {}-{}\nquestion: {}\nanswer with: {a} / {b} or {CANNOT_TELL}",
        text(row, "audit"),
        text(row, "path"),
        text(row, "unit_name"),
        lines[0],
        lines[1],
        sentence(row)
    )
}

/// Where a batch's answers go: `<dir>/<lang>/answers-<n>.jsonl`.
pub fn answer_file(dir: &str, lang: &str, n: u64) -> String {
    format!("{dir}/{lang}/answers-{n}.jsonl")
}

fn clone_root(base: &str, corpus: &str) -> String {
    format!("{base}/{corpus}")
}

/// Every file of a language's batches, `(name under <dir>/<lang>, text)`:
/// `batch-<n>.md` each and `manifest.json` — a pure function of the
/// sample, the clone base and the batch directory.
pub fn render(exam: &FlowExam, sample: &Value, base: &str, dir: &str) -> Vec<(String, String)> {
    let lang = exam.lang;
    let mut files = Vec::new();
    let mut manifest = Vec::new();
    let readings_section = match readings(exam) {
        Some(_) => format!("{READINGS_TEXT}\n"),
        None => String::new(),
    };
    for b in plan(exam, sample) {
        let questions: Vec<String> = b.rows.iter().map(|r| question(r)).collect();
        let ids: Vec<&str> = b.rows.iter().map(|r| text(r, "audit")).collect();
        let body = PROMPT
            .replace("{READINGS}", &readings_section)
            .replace("{BATCH_ID}", &format!("{lang}-{}", b.n))
            .replace("{LANG}", lang)
            .replace("{CLONE_ROOT}", &clone_root(base, b.corpus))
            .replace("{ANSWER_FILE}", &answer_file(dir, lang, b.n))
            .replace("{QUESTIONS}", &questions.join("\n\n"));
        files.push((format!("batch-{}.md", b.n), body));
        manifest.push(json!({
            "n": b.n, "corpus": b.corpus, "clone_root": clone_root(base, b.corpus), "ids": ids,
        }));
    }
    let mut doc = json!({"lang": lang, "generation": exam.generation, "batches": manifest});
    if let Some(r) = readings(exam) {
        doc["readings"] = json!(r);
    }
    let text = serde_json::to_string_pretty(&doc).expect("json") + "\n";
    files.push(("manifest.json".to_string(), text));
    files
}

/// The clone base: CE_FLOW_CLONE_ROOT, else the repository's corpora.
pub fn clone_base() -> String {
    std::env::var("CE_FLOW_CLONE_ROOT").unwrap_or_else(|_| {
        let root = crate::common::repo_root().join(".ce-eval/corpora");
        root.to_string_lossy().replace('\\', "/")
    })
}

/// Write one file, or find it already there byte for byte: a batch an
/// auditor may be reading is never rewritten under them.
pub(crate) fn put(path: &Path, text: &str) {
    match std::fs::read_to_string(path) {
        Ok(have) => assert_eq!(
            have,
            text,
            "{}: differs from this rendering",
            path.display()
        ),
        Err(_) => std::fs::write(path, text).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
    }
}

#[test]
#[ignore = "writes the audit batches beside the pinned clones"]
fn flow_batches() {
    let exam = exam(&env("CE_FLOW_LANG"));
    let (dir, base) = (env("CE_FLOW_BATCH_DIR"), clone_base());
    for (corpus, tip) in &exam.corpora {
        pinned_at(Path::new(&base), corpus, tip);
    }
    let out = Path::new(&dir).join(exam.lang);
    std::fs::create_dir_all(&out).expect("batch dir");
    for (name, text) in render(exam, &exam.sample(), &base, &dir) {
        put(&out.join(&name), &text);
        let questions = text.matches("\n### ").count();
        println!("{}/{name}: {questions} questions", out.display());
    }
}
