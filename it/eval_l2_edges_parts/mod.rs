//! The ignored half of the edge-coverage instrument (O48): replay the
//! REAL pipeline over each commit the reviewed edge register names and
//! freeze what it relocated. A git-history instrument like
//! bench_backfill and tombstone_replay, so it stays a standing ignored
//! leg under the EVAL-SET retirement rule.
//!
//!   CE_CORE_BIN=… cargo test --release --test it -- --ignored eval_l2_edges --nocapture
//!
//! Only the GT-bearing commits are replayed: that is the scope of the
//! coverage question and of the edge_violations gate alike. Set
//! CE_EDGES_CORPUS to a single corpus name to narrow a rerun.

use crate::common::{git_out, repo_root};
use crate::eval_commit_review as review;
use crate::eval_support::corpus::{PINNED_CORPORA, pinned_root};
use codeeraser::corelink::Link;
use codeeraser::fourclass::batch::{PairInput, classify_batch};
use codeeraser::fourclass::session;
use codeeraser::tombstone::texts::{self, Side};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub const SCHEMA: &str = "ce.eval-commit-edges/1.0.0";
pub const CORPORA: [Option<&str>; 3] = [None, Some("requests"), Some("ripgrep")];

/// The corpora with an edge register, and where each one is checked
/// out (self = this repository).
fn corpus_root(corpus: Option<&str>) -> PathBuf {
    let Some(name) = corpus else {
        return repo_root();
    };
    let (_, tip) = PINNED_CORPORA
        .iter()
        .find(|(n, _)| *n == name)
        .expect("pinned corpus");
    pinned_root(name, tip)
}

/// One commit, replayed: its pairs' relocations in the session report
/// shape (from, from_unit, to, to_unit, lines) — the SAME projection
/// the product ships, so the ledger cannot drift from the product.
fn replay(root: &Path, sha: &str, link: &mut Link) -> Value {
    let parent = format!("{sha}^");
    let pairs = session::commit_pairs(root, sha).expect("commit pairs");
    let (loaded, unread) =
        texts::load(root, &pairs, Side::Rev(&parent), Side::Rev(sha)).expect("blob texts");
    assert_eq!(
        unread, 0,
        "{sha}: {unread} pairs unread — the changeset is not whole"
    );
    let inputs: Vec<PairInput> = loaded
        .iter()
        .map(|l| PairInput {
            before: &l.before,
            after: &l.after,
            lang: l.lang,
        })
        .collect();
    let seen: Vec<(Option<String>, Option<String>)> = loaded
        .iter()
        .map(|l| (Some(l.rel.clone()), Some(l.rel.clone())))
        .collect();
    let batch = classify_batch(&inputs, Some(link));
    assert!(
        batch.degraded.is_none(),
        "{sha}: degraded {:?}",
        batch.degraded
    );
    let report = session::report_json(&batch, &seen);
    json!({"sha": sha, "relocations": report["relocations"].clone()})
}

/// Full shas for the register's 9-character prefixes.
fn resolve(root: &Path, prefixes: &[String]) -> Vec<String> {
    prefixes
        .iter()
        .map(|p| {
            let (ok, out) = git_out(root, &["rev-parse", "--verify", "-q", p]);
            assert!(ok, "{p}: not in {}", root.display());
            out.trim().to_string()
        })
        .collect()
}

fn doc_path(corpus: Option<&str>) -> String {
    match corpus {
        None => "../contracts/eval/commit-edges-v1.json".into(),
        Some(c) => format!("../contracts/eval/commit-edges-{c}-v1.json"),
    }
}

#[test]
#[ignore] // needs CE_CORE_BIN, git history and the pinned external clones
fn generate_edge_documents() {
    let only = std::env::var("CE_EDGES_CORPUS").ok();
    assert!(
        only.as_deref()
            .is_none_or(|n| CORPORA.iter().any(|c| c.unwrap_or("self") == n)),
        "unknown CE_EDGES_CORPUS"
    );
    for corpus in CORPORA {
        if only
            .as_deref()
            .is_some_and(|o| o != corpus.unwrap_or("self"))
        {
            continue;
        }
        let root = corpus_root(corpus);
        let (mut link, _) = Link::open(&crate::common::core_bin()).expect("open core");
        let shas = resolve(&root, &review::edge_shas(corpus));
        let commits: Vec<Value> = shas.iter().map(|s| replay(&root, s, &mut link)).collect();
        let (ok, tip) = git_out(&root, &["rev-parse", "HEAD"]);
        assert!(ok, "tip of {}", root.display());
        let doc = json!({
            "schema": SCHEMA,
            "corpus": corpus,
            "generated_from": tip.trim(),
            "commits": commits,
        });
        let path = doc_path(corpus);
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&doc).expect("ser") + "\n",
        )
        .expect("write edge doc");
        println!("wrote {path} ({} commits)", commits.len());
    }
}
