//! FPR replay for the hard-budget class over a git history (v2.35 guard
//! lane, GD2) — the record the class never had: booklet 11's ledger
//! rows are duplicate blocks only, and the class shipped at deny on
//! them. Every changed in-scope file of every first-parent commit is
//! one write (common::policy: the hook's population, scope and fence):
//! `before` = the parent blob's line count (0 for an added file),
//! `after` = the child's, `cap` = the file's own hard line off its class
//! table, judged by `guard::zone::budget_kind` — the hook's own rule,
//! no second copy here. Every write past the line is frozen as a row
//! (`crossing` / `growth` refused, `carried` let through), so the gate
//! re-derives each number.
//!
//!   CE_FPR_CORPUS=self CE_FPR_TIP=<sha> cargo test --release --test it -- --ignored fpr_budget_replay --nocapture
//!
//! CE_FPR_CORPUS names the row: `self` (this repository, or CE_FPR_REPO,
//! over common::history's chain — CE_FPR_TIP / CE_FPR_LIMIT), `requests`
//! (CE_FPR_REPO, CE_FPR_TIP=1f6589ec, CE_FPR_LIMIT=400 — the zone
//! ledger's window), or one of the per-language ledger's corpora
//! (fpr_lang_replay::CORPORA), read at the exam's pinned tip over the
//! same WINDOW under CE_FPR_CLONE_ROOT (default `.ce-eval/corpora`).
//! CE_BLESS=1 merges the row into contracts/eval/fpr-budget-v1.json;
//! with CE_FPR_OUT the doc is read and written there instead.

use crate::common::ledger::WINDOW;
use crate::common::policy::{Blobs, Write, each_write};
use crate::common::{chain, chain_of, repo_root, tmp};
use crate::eval_lang_parts::exam_of_corpus;
use crate::eval_support::pinned_at;
use crate::fpr_budget_replay_parts::{Corpus, LEDGER, Over, table};
use crate::fpr_lang_replay::corpora;
use codeeraser::guard::zone::{self, Budget};
use std::path::{Path, PathBuf};

/// One write: its row when it lands past the line.
fn measure(shadow: &Path, w: &Write, blobs: &mut Blobs) -> Option<Over> {
    let cap = zone::table_for(shadow, &w.policy.cfg, w.rel).file_lines_fail;
    let after = blobs.lines(w.commit, w.rel);
    let before = if w.status == 'A' {
        0
    } else {
        blobs.lines(w.parent, w.rel)
    };
    let kind = match zone::budget_kind(Ok(before), after, cap)? {
        Budget::Crossing => "crossing",
        Budget::Growth => "growth",
        Budget::Carried => "carried",
        Budget::Unread => unreachable!("a blob's size is always read"),
    };
    Some(Over {
        sha: w.commit[..8].to_string(),
        rel: w.rel.to_string(),
        before,
        after,
        cap,
        kind,
    })
}

/// One corpus replayed end to end over `commits`.
pub fn replay(repo: &Path, shadow: &Path, name: &str, commits: &[String]) -> Corpus {
    let (mut events, mut rows) = (0, Vec::new());
    let unreadable = each_write(repo, shadow, commits, |w, b| {
        events += 1;
        rows.extend(measure(shadow, w, b));
    });
    let tip = commits.last().expect("a tip");
    Corpus {
        name: name.to_string(),
        tip: tip[..8].to_string(),
        commits: commits.len() - 1,
        events,
        unreadable,
        rows,
    }
}

/// The repository and the chain a corpus name selects.
pub fn corpus(name: &str) -> (PathBuf, Vec<String>) {
    if corpora().iter().any(|(_, n)| *n == name) {
        let (_, tip) = exam_of_corpus(name);
        let base = std::env::var_os("CE_FPR_CLONE_ROOT")
            .map_or_else(|| repo_root().join(".ce-eval/corpora"), PathBuf::from);
        let repo = pinned_at(&base, name, tip);
        let commits = chain_of(&repo, tip, Some(WINDOW));
        return (repo, commits);
    }
    let repo = std::env::var_os("CE_FPR_REPO").map_or_else(repo_root, PathBuf::from);
    let commits = chain(&repo);
    (repo, commits)
}

/// The ledger's row order: self, requests, then the per-language
/// corpora in their ledger's order.
pub fn rank(name: &str) -> usize {
    match name {
        "self" => 0,
        "requests" => 1,
        _ => 2 + corpora().iter().position(|(_, n)| *n == name).unwrap_or(99),
    }
}

#[test]
#[ignore = "history instrument: replays a repository's first-parent chain, minutes; run by hand"]
fn every_write_past_the_line_is_read_against_its_parent() {
    let name = std::env::var("CE_FPR_CORPUS").unwrap_or_else(|_| "self".into());
    let (repo, commits) = corpus(&name);
    let c = replay(&repo, &tmp(&format!("fpr-budget-{name}")), &name, &commits);
    let walked = c.summary(&repo);
    let row = c.json();
    let printed = table(std::slice::from_ref(&row));
    LEDGER.report(row, printed, walked, |c| {
        (rank(c["name"].as_str().unwrap_or_default()), String::new())
    });
}
