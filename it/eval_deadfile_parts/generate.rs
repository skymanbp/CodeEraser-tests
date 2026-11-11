//! The dead-file exam's generators — `#[ignore]`, because they read a
//! corpus checkout at its pinned tip and run the product's deadcode
//! family on it (`CE_CORE_BIN` names the core). Each writes frozen docs
//! once and refuses to overwrite one: a re-freeze is a new generation.
//!   CE_DEADFILE_ROOT=<dir of checkouts> CE_DEADFILE_CORPUS=requests [CE_DEADFILE_OUT=<dir>] cargo test --test it -- --ignored eval_deadfile_parts::generate::deadfile_slice --nocapture
//!   [CE_DEADFILE_OUT=<dir>] cargo test --test it -- --ignored eval_deadfile_parts::generate::deadfile_samples --nocapture
//! `<dir of checkouts>/<corpus>` is a clean clone at its exam-table tip; the
//! deadcode run writes its index under the clone's own `.ce/`.

use super::{SAMPLES, SCHEMAS, SLICE, constants, corpora, doc_path, draw, features, summary};
use crate::common::ledger::out_file;
use crate::eval_lang_parts::freezing::frozen;
use crate::eval_lang_parts::generate::{env, freeze};
use crate::eval_support::{content_sha, git_in, load};
use codeeraser::graph::deadcode::{self, Advisory};
use codeeraser::scan::lang::Lang;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

const SLICE_METHOD: &str = "file-tier dead universe of one pinned corpus: every dead row of \
    the product's deadcode family (graph wire without the symbol advisory, the core's verdict \
    and confidence) at the pinned tip of a clean checkout, one row per file, with the measured \
    features of features.rs - top-level node kinds through the product's parser, kept \
    out-edges off the same graph wire, other files of the pinned tree spelling the basename \
    (git grep -F) and the stem as a word (git grep -F -w), convention path segments, document \
    class - and the sha256 of the text. Frozen before any audit is read and before any rule \
    reads these features.";

const SAMPLE_METHOD: &str = "per language over the frozen slices: rank id = \
    sha256(domain|corpus|commit|path); at most `total` primary rows by the language exams' \
    quota rule over corpora (each corpus first min(15, pool), the seats left by largest \
    remainder), each corpus taking its lowest rank-domain hashes; the reserve is the \
    audit-domain rank over the unpicked rest, at most `reserve` rows; a language with fewer \
    than `whole_below` rows is taken whole with no reserve. Rows stored in audit-domain order.";

fn tip_of(name: &str) -> &'static str {
    corpora()
        .into_iter()
        .find(|(n, _)| *n == name)
        .map(|(_, t)| t)
        .unwrap_or_else(|| panic!("{name}: no exam corpus"))
}

/// One dead file's frozen row.
fn row(repo: &str, tip: &str, d: &deadcode::DeadRow, out_edges: u64) -> Value {
    let path = d.path.as_str();
    let text = git_in(Some(repo), &["show", &format!("{tip}:{path}")]);
    let lang = Lang::from_path(Path::new(path));
    let (base, stem) = features::names(path);
    json!({
        "path": path,
        "lang": lang.map_or("unknown", |l| l.name()),
        "verdict": d.verdict, "confidence": d.conf,
        "kinds": features::top_kinds(&text, lang),
        "out_edges": out_edges,
        "basename_spelled": features::spelled_by(repo, tip, path, base, false),
        "stem_spelled": features::spelled_by(repo, tip, path, stem, true),
        "conventions": features::conventions(path),
        "doc_class": features::doc_class(path),
        "sha256": content_sha(&text),
    })
}

/// The dead rows of one checkout with each file's kept out-edges.
fn dead_rows(repo: &str, tip: &str, core: &str) -> Vec<Value> {
    let root = Path::new(repo);
    let (idx, db) = codeeraser::dedup::refreshed_index(root, None).expect("index");
    let w = deadcode::wire_of(root, &idx, &db, Advisory::No).expect("wire");
    drop(idx);
    let report = deadcode::judge_report(root, core, &w).expect("deadcode");
    assert!(
        report.degraded.is_none(),
        "{repo}: degraded {:?}",
        report.degraded
    );
    let node: BTreeMap<&str, i64> = deadcode::file_nodes(&w)
        .into_iter()
        .map(|(i, p)| (p, i))
        .collect();
    let mut rows: Vec<Value> = report
        .dead
        .iter()
        .map(|d| {
            let at = node.get(d.path.as_str()).copied();
            let out = w
                .edges
                .iter()
                .filter(|e| Some(e[0]) == at && e[2] != 3 && e[2] != 5)
                .count();
            row(repo, tip, d, out as u64)
        })
        .collect();
    rows.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    rows
}

#[test]
#[ignore = "reads a corpus checkout and runs the deadcode family on it"]
fn deadfile_slice() {
    let name = env("CE_DEADFILE_CORPUS");
    let tip = tip_of(&name);
    let repo = format!("{}/{name}", env("CE_DEADFILE_ROOT"));
    assert_eq!(
        git_in(Some(&repo), &["rev-parse", "HEAD"]).trim(),
        tip,
        "{name}: not at its tip"
    );
    let core = env("CE_CORE_BIN");
    let rows = dead_rows(&repo, tip, &core);
    let doc = json!({
        "corpus": {"name": name, "tip": tip},
        "scope": {"walk": "the product's own walk of the checkout (ce.toml, ignore files, built-ins)"},
        "summary": summary(&rows),
        "excluded": {},
        "files": rows,
    });
    println!("{name}: {}", doc["summary"]);
    let doc = frozen(SCHEMAS.0, SLICE_METHOD, constants(), doc);
    freeze(
        &out_file("CE_DEADFILE_OUT", &doc_path(SLICE.family, &name)),
        &doc,
    );
}

/// Every frozen slice's rows, each with its corpus and commit, by language.
pub fn pools() -> (BTreeMap<String, Vec<Value>>, Vec<Value>) {
    let (mut by, mut sources) = (BTreeMap::<String, Vec<Value>>::new(), Vec::new());
    for (name, tip) in corpora() {
        let slice = load(&out_file("CE_DEADFILE_OUT", &doc_path(SLICE.family, name)));
        sources.push(json!({"corpus": name, "tip": tip, "rows": slice["summary"]["rows"]}));
        for r in slice["files"].as_array().expect("files") {
            let mut item = r.clone();
            item["corpus"] = json!(name);
            item["commit"] = json!(tip);
            by.entry(r["lang"].as_str().expect("lang").to_string())
                .or_default()
                .push(item);
        }
    }
    (by, sources)
}

/// One language's sample doc, as the generator writes it and the gate
/// re-derives it.
pub fn sample_doc(lang: &str, pool: &[Value], sources: &[Value]) -> Value {
    let (allocation, rows, reserve) = draw(pool);
    json!({"lang": lang, "pool": pool.len(), "sources": sources, "allocation": allocation, "rows": rows, "reserve": reserve})
}

#[test]
#[ignore = "reads every frozen slice and writes the samples"]
fn deadfile_samples() {
    let (by, sources) = pools();
    for (lang, pool) in &by {
        let doc = sample_doc(lang, pool, &sources);
        println!(
            "{lang}: pool {} rows {} reserve {}",
            pool.len(),
            doc["rows"].as_array().map_or(0, Vec::len),
            doc["reserve"].as_array().map_or(0, Vec::len)
        );
        let doc = frozen(SCHEMAS.1, SAMPLE_METHOD, constants(), doc);
        freeze(&out_file("CE_DEADFILE_OUT", &doc_path(SAMPLES, lang)), &doc);
    }
}
