//! The language exams' generators — `#[ignore]`, because they read the
//! pinned corpus clone under `.ce-eval/corpora/<name>` (EXAMS pins the
//! tip; corpus::pinned_root refuses any other tree). Each writes one
//! frozen doc and refuses to overwrite it: a re-freeze is a new
//! generation (Exam::generation) and a named entry in
//! docs/EVAL-SET-LANGS.md, never a silent re-run.
//!   CE_LANG_CORPUS=gson cargo test --test it -- --ignored eval_lang_parts::generate::lang_slice --nocapture
//!   CE_LANG=java cargo test --test it -- --ignored eval_lang_parts::generate::lang_sample --nocapture
//!   CE_LANG_CORPUS=gson cargo test --test it -- --ignored eval_lang_parts::generate::lang_precision --nocapture
//! The precision doc is the one frozen doc that depends on product
//! code: a ladder change that moves an answer re-scores it — delete,
//! regenerate, and name the change in the registry.

use super::review::verify_review;
use super::{
    AUDIT_TABLES, Exam, PRECISION_DOCS, SAMPLE_SCHEMA, SAMPLES, SLICE_SCHEMA, SLICES, score,
};
use crate::eval_support::{
    generated_from, git_in, lang_of, of_corpus, pinned_root, site_row, site_summary,
};
use codeeraser::graph::sites::detect;
use codeeraser::graph::store::is_resolver_config;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const SLICE_METHOD: &str = "site universe of one pinned corpus for one v2.30 language: every \
    file whose extension is in scope, inventoried with the sha256 of the text the detector saw \
    and its resolution-free per-kind site counts (graph::sites — grammar kind tables and \
    file-local facts only, no path ever consulted). Frozen before the language's ladder exists; \
    the falsification constants are pre-registered here, before any measurement.";

const SAMPLE_METHOD: &str = "hash-ranked stratified draw over the language's frozen site \
    universes, reconstructed file by file against their frozen rows. rank id = \
    sha256(domain|corpus|commit|path|line|nth|kind|spec), the M5-2 payload. Primaries: per \
    kind min(min_per_kind, pool), the seats left to total by largest remainder over each \
    kind's remaining pool, top of the site-domain rank per kind; rows stored in audit-domain \
    order. Backups: per kind, the audit-domain rank over the unpicked rest — the audit walks \
    them only for an unanswerable primary of the same kind.";

const PRECISION_METHOD: &str = "the frozen sample rows of one corpus, resolved by the shipped \
    ladder against its frozen universe (every file re-read at the pinned tip and reproduced \
    against its frozen row first; a drifted tree never scores), with the tree's resolver \
    configs and no declared root, judged against the frozen blind audit. An in-corpus answer \
    matches its truth exactly, or at file level when the ladder made no unit claim; a package \
    answer matches a package truth; External is an answer, so external on an in-corpus truth \
    is wrong, never missed. The universe ledger runs the ladder over every site of the frozen \
    universe: its resolution rate is a recall ceiling. Each audit site gap is answered with \
    every site the detector reads off its line. Pre-registered: the per-rung cut table and one \
    line per site kind are published; precision >= 0.90 overall and per corpus where the \
    in-corpus truths reach 5 (the M5-2 G2 contract).";

pub(super) fn env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{key} names what to freeze"))
}

pub(super) fn corpus_repo(name: &str, tip: &str) -> String {
    pinned_root(name, tip).to_string_lossy().into_owned()
}

pub(super) fn blob(repo: &str, tip: &str, path: &str) -> String {
    git_in(Some(repo), &["show", &format!("{tip}:{path}")])
}

/// Write a frozen doc once, at its file (Docs::file).
pub(super) fn freeze(file: &str, doc: &Value) {
    assert!(
        !std::path::Path::new(file).exists(),
        "{file} is frozen: a re-freeze is a new generation and a named ledger entry"
    );
    let text = serde_json::to_string_pretty(doc).expect("json") + "\n";
    std::fs::write(file, text).expect(file);
    println!("{file} written");
}

/// The pinned tree's paths. -z: unquoted non-ASCII paths;
/// --full-tree: root-relative paths whatever the cwd (the M5-2
/// walker's two lessons).
fn tree_paths(repo: &str, tip: &str) -> Vec<String> {
    let listing = git_in(
        Some(repo),
        &["ls-tree", "-r", "--full-tree", "--name-only", "-z", tip],
    );
    listing
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect()
}

/// The pinned tree's in-scope files as universe rows, plus the tally
/// of what the scope left out.
fn walk(exam: &Exam, repo: &str, tip: &str) -> (Vec<Value>, BTreeMap<&'static str, u64>) {
    let (mut files, mut excluded) = (Vec::new(), BTreeMap::new());
    for path in tree_paths(repo, tip) {
        let ext = path.rsplit_once('.').map_or("", |(_, e)| e);
        if exam.exts.contains(&ext) {
            files.push(site_row(&path, exam.lang, &blob(repo, tip, &path)));
        } else {
            *excluded.entry("other_extension").or_insert(0) += 1;
        }
    }
    files.sort_by(|a: &Value, b: &Value| a["path"].as_str().cmp(&b["path"].as_str()));
    (files, excluded)
}

#[test]
#[ignore = "reads the pinned corpus clone"]
fn lang_slice() {
    let name = env("CE_LANG_CORPUS");
    let (exam, tip) = super::exam_of_corpus(&name);
    let (files, excluded) = walk(exam, &corpus_repo(&name, tip), tip);
    freeze(
        &SLICES.file(exam, &name),
        &json!({
            "schema": SLICE_SCHEMA,
            "corpus": {"name": name, "tip": tip, "lang": exam.lang},
            "scope": super::scope(exam),
            "constants": super::slice_constants(),
            "generated_from": generated_from(),
            "method": SLICE_METHOD,
            "summary": site_summary(&files),
            "excluded": excluded,
            "files": files,
        }),
    );
}

/// Every file of one frozen universe with its text at the pinned tip,
/// each reproducing its frozen row first — the tree equals the frozen
/// universe by checked reconstruction, not by trust (the sample's pool
/// and the scorer's ladder both read it).
pub(super) fn frozen_texts(
    exam: &Exam,
    name: &str,
    tip: &str,
    slice: &Value,
) -> Vec<(String, String)> {
    let repo = corpus_repo(name, tip);
    let rows = slice["files"].as_array().expect("files");
    rows.iter()
        .map(|row| {
            let path = row["path"].as_str().expect("path");
            let text = blob(&repo, tip, path);
            assert_eq!(
                &site_row(path, exam.lang, &text),
                row,
                "{name}/{path}: not the frozen row"
            );
            (path.to_string(), text)
        })
        .collect()
}

/// Every site of one frozen universe, re-detected at the pinned tip.
fn corpus_pool(exam: &Exam, name: &str, tip: &str, slice: &Value) -> Vec<Value> {
    let mut pool = Vec::new();
    for (path, text) in frozen_texts(exam, name, tip, slice) {
        for s in detect(&text, lang_of(exam.lang)) {
            pool.push(json!({
                "corpus": name, "commit": tip, "path": path, "line": s.line,
                "nth": s.nth, "kind": s.kind, "lang": exam.lang, "spec": s.spec,
            }));
        }
    }
    pool
}

#[test]
#[ignore = "reads every pinned corpus clone of the language"]
fn lang_sample() {
    let exam = super::exam(&env("CE_LANG"));
    let (mut pool, mut sources) = (Vec::new(), Vec::new());
    for (name, tip) in exam.corpora {
        let slice = SLICES.load(exam, name);
        pool.extend(corpus_pool(exam, name, tip, &slice));
        sources.push(super::source_row(name, &slice));
    }
    let draw = super::draw::draw(&pool);
    freeze(
        &SAMPLES.file(exam, exam.lang),
        &json!({
            "schema": SAMPLE_SCHEMA,
            "lang": exam.lang,
            "constants": super::sample_constants(),
            "generated_from": generated_from(),
            "method": SAMPLE_METHOD,
            "sources": sources,
            "allocation": draw.allocation,
            "rows": draw.primary,
            "backups": draw.backups,
        }),
    );
}

/// One corpus scored: the audit verified against its sample first,
/// then each sampled row judged (score.rs), the universe ledger and
/// the site gaps answered, and the doc run through the gate's own
/// verifier before it is frozen. CE_R0_DISPOSITION carries the written
/// disposition the RG1 trigger asks for, when it fires.
#[test]
#[ignore = "reads the pinned corpus clone"]
fn lang_precision() {
    let name = env("CE_LANG_CORPUS");
    let (exam, tip) = super::exam_of_corpus(&name);
    let slice = SLICES.load(exam, &name);
    let sample = exam.sample();
    let review = AUDIT_TABLES.load(exam, &name);
    verify_review(exam, &name, &review, &sample);
    let repo = corpus_repo(&name, tip);
    let texts = frozen_texts(exam, &name, tip, &slice);
    let configs = tree_paths(&repo, tip)
        .into_iter()
        .filter(|p| is_resolver_config(std::path::Path::new(p)))
        .collect();
    let tree = score::tree(std::path::Path::new(&repo), &texts, configs);
    let scope = tree.scope();
    let sampled = of_corpus(sample["rows"].as_array().expect("rows"), &name);
    let truths = review["rows"].as_array().expect("rows");
    let rows: Vec<Value> = sampled
        .iter()
        .zip(truths)
        .map(|(s, a)| score::judge(s, a["truth"].as_str().expect("truth"), &scope))
        .collect();
    let mut doc = json!({
        "schema": score::PRECISION_SCHEMA,
        "corpus": {"name": name, "tip": tip, "lang": exam.lang},
        "generated_from": generated_from(),
        "method": PRECISION_METHOD,
        "summary": score::summary(&rows),
        "universe": score::universe(&texts, exam.lang, &scope),
        "site_gaps": score::gaps(&review, &texts, exam.lang, &scope),
        "rows": rows,
    });
    if let Ok(why) = std::env::var("CE_R0_DISPOSITION") {
        doc["r0_disposition"] = json!(why);
    }
    super::precision::verify_precision(exam, &name, &doc, &sample);
    freeze(&PRECISION_DOCS.file(exam, &name), &doc);
}
