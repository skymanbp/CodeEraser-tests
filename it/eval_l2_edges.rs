//! Edge-level relocation COVERAGE (plan v2.29 step 10, O48). The
//! reviewed edge register (eval_commit_review) says which
//! (commit, source file, destination file, unit) relocations exist;
//! this leg says, per pair, how the pipeline named it — by a line
//! station, by the declaration stage (proto 7.1.0), or not at all,
//! with the class of every remainder.
//!
//! The number is DERIVED here and read back out of docs/EVAL-SET-M5-3.md,
//! so a typed coverage number cannot survive: the gate compares the
//! sentence this leg builds against the sentence the document
//! carries (the fixture_contract triple's pattern).
//!
//! CI half: no git, no core, no external clone — it reads the frozen
//! contracts/eval/commit-edges*-v1.json the ignored generator wrote
//! (eval_l2_edges_parts). Deliberately a SEPARATE document: the seven
//! L2 gates keep reading commit-l2*-v1.json byte for byte.
//!
//! Regenerate:
//!   CE_CORE_BIN=$(cd core && cabal list-bin ce-core) \
//!     cargo test --release --test it -- --ignored eval_l2_edges --nocapture

use crate::eval_commit_review as review;
use crate::eval_l2_register::{names_unit, rel_bases};
use crate::eval_support;
use serde_json::Value;

/// How one reviewed edge-unit pair was named.
#[derive(PartialEq, Eq)]
pub enum How {
    Line,
    Decl,
    Uncovered,
}

pub struct Row {
    pub sha: String,
    pub from: String,
    pub to: String,
    pub unit: String,
    pub how: How,
}

/// The frozen edge documents, corpus name first ("" = self).
fn frozen() -> Vec<(Option<String>, Value)> {
    let mut docs: Vec<_> = eval_support::frozen_docs("commit-edges")
        .into_iter()
        .map(|p| {
            (
                eval_support::doc_suffix(&p, "commit-edges"),
                eval_support::load(&p),
            )
        })
        .collect();
    docs.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        docs.iter().map(|d| d.0.as_deref()).collect::<Vec<_>>(),
        crate::eval_l2_edges_parts::CORPORA,
        "every reviewed corpus must be frozen"
    );
    for (corpus, doc) in &docs {
        assert_eq!(doc["schema"], crate::eval_l2_edges_parts::SCHEMA);
        assert_eq!(doc["corpus"].as_str(), corpus.as_deref());
        let commits = doc["commits"].as_array().expect("commits");
        let shas = review::edge_shas(corpus.as_deref());
        assert_eq!(commits.len(), shas.len(), "one row per reviewed commit");
        for sha in shas {
            assert_eq!(
                commits
                    .iter()
                    .filter(|c| c["sha"].as_str().is_some_and(|s| s.starts_with(&sha)))
                    .count(),
                1,
                "{sha}"
            );
        }
    }
    docs
}

/// The relocations one document holds for `sha`.
fn relocations_of(doc: &Value, sha: &str) -> Vec<Value> {
    doc["commits"]
        .as_array()
        .expect("commits")
        .iter()
        .find(|c| c["sha"].as_str().is_some_and(|s| s.starts_with(sha)))
        .and_then(|c| c["relocations"].as_array().cloned())
        .unwrap_or_default()
}

/// Whether any relocation of `rels` names `unit` on this file edge,
/// and with how many moved lines — `lines == 0` IS the declaration
/// stage, which claims no line (fourclass/batch/edges.rs).
fn how(rels: &[Value], from: &str, to: &str, unit: &str) -> How {
    let mut seen = How::Uncovered;
    for r in rels {
        if r["from"] != from || r["to"] != to || !rel_bases(r).iter().any(|b| b == unit) {
            continue;
        }
        match r["lines"].as_u64().expect("relocation line count") {
            0 if seen == How::Uncovered => seen = How::Decl,
            0 => {}
            _ => return How::Line,
        }
    }
    seen
}

/// The per-pair ledger of one corpus, in register order.
pub fn ledger(corpus: Option<&str>, doc: &Value) -> Vec<Row> {
    let mut out = Vec::new();
    for sha in review::edge_shas(corpus) {
        let rels = relocations_of(doc, &sha);
        for e in review::edges_in(corpus, &sha) {
            let (from, to) = (
                e["from"].as_str().expect("from"),
                e["to"].as_str().expect("to"),
            );
            for u in e["units"].as_array().expect("units") {
                let unit = u.as_str().expect("unit").trim_start_matches('~');
                assert!(
                    names_unit(&e, unit),
                    "{sha}: {unit} is not its own row's unit"
                );
                out.push(Row {
                    sha: sha.clone(),
                    from: from.into(),
                    to: to.into(),
                    unit: unit.into(),
                    how: how(&rels, from, to, unit),
                });
            }
        }
    }
    out
}

fn display(corpus: Option<&str>) -> String {
    corpus.map_or_else(|| "自仓".to_string(), str::to_string)
}

/// The sentence docs/EVAL-SET-M5-3.md must carry, built from the frozen
/// documents — never typed.
pub fn coverage_sentence(measured: &[(Option<String>, usize, usize)]) -> String {
    let parts: Vec<String> = measured
        .iter()
        .map(|(c, hit, all)| format!("{} {hit}/{all}", display(c.as_deref())))
        .collect();
    format!("**{}**", parts.join("、"))
}

#[test]
fn edge_coverage_is_derived_and_every_pair_is_classed() {
    let mut measured = Vec::new();
    for (corpus, doc) in frozen() {
        let rows = ledger(corpus.as_deref(), &doc);
        assert!(!rows.is_empty(), "{corpus:?}: empty edge ledger");
        for r in &rows {
            let how = match r.how {
                How::Line => "line station",
                How::Decl => "declaration alignment",
                How::Uncovered => "UNCOVERED",
            };
            println!("{} {} {} -> {} [{how}]", &r.sha[..9], r.unit, r.from, r.to);
        }
        let hit = rows.iter().filter(|r| r.how != How::Uncovered).count();
        measured.push((corpus, hit, rows.len()));
    }
    let want = coverage_sentence(&measured);
    println!("{want}");
    let text = std::fs::read_to_string(crate::common::repo_root().join("docs/EVAL-SET-M5-3.md"))
        .expect("EVAL-SET-M5-3.md");
    assert!(text.contains(&want), "EVAL-SET-M5-3.md must carry {want}");
}

/// The reverse gate over the NEW edges: every relocation the frozen
/// edge document records is checked against the reviewed register by
/// the SAME predicate the L2 gate uses — a declaration edge outside
/// the ground truth for a tabled unit is an invention, and reddens
/// here exactly as a line edge would.
#[test]
fn no_edge_document_invents_a_tabled_relocation() {
    for (corpus, doc) in frozen() {
        let rows: Vec<Value> = doc["commits"].as_array().expect("commits").clone();
        let bad = crate::eval_l2_register::edge_violations(corpus.as_deref(), &rows);
        assert!(
            bad.is_empty(),
            "{corpus:?}: invented relocation edges: {bad:?}"
        );
    }
}
