//! Changeset-level FPR replay for the update-supervision class
//! (plan v2.29 step 10, C-R-L2-4; the ledger is docs/FPR-L2.md).
//! The sample gate replays 600 single-file edits; the Stop audit's
//! four-class report judges a MULTI-FILE CHANGESET, and
//! that population had no ledger at all — which is why the cross-file
//! relocation and stacking report has never been allowed a deny path.
//!
//! One event = one first-parent commit of a frozen commit slice,
//! assembled the way the audit assembles a session (session pair
//! builder, one cat-file batch for the two blob sides) and judged
//! through the seam `Judge::classify` itself calls, so the ledger
//! measures the pipeline and not a copy of it. A changeset that is
//! not WHOLE is not an event: every skip is named and counted.
//!
//! Intercept = the counterfactual deny: `[guard] mode == "deny"` and
//! the core's `suspicions` non-empty (the M4 stacking conjunction,
//! constants in CE.FourClass.Verdict). Relocations are NOT the
//! predicate — a clean refactor produces them by the hundred — they
//! ride the row as evidence. No default tier is flipped here; the
//! gate pins that.
//!
//!   cargo test --release -j 8 --test it -- --ignored l2_fpr_replay --nocapture
//!
//! CE_BLESS=1 writes contracts/eval/fpr-l2-v1.json;
//! CE_L2FPR_CORPUS=<self|requests|ripgrep> runs one corpus;
//! CE_L2FPR_LIMIT=<n> takes the newest n commits (a smoke run — the
//! ledger's numbers are full runs).

use crate::common;
use crate::l2_fpr_replay_parts as parts;
use crate::l2_fpr_replay_parts::fixtures;
use crate::l2_fpr_replay_parts::{Label, Row, Tally};
use codeeraser::daemon::judge::Judge;
use codeeraser::fourclass::batch::BatchClassification;
use codeeraser::fourclass::session;
use codeeraser::fourclass::stacking::dup_spans;
use codeeraser::tombstone::texts::{self, Side};
use std::path::{Path, PathBuf};

/// The three frozen corpora, in ledger order.
pub const CORPORA: [&str; 3] = ["self", "requests", "ripgrep"];

/// One commit put in front of the judge, or a named reason why it
/// could not be: a partial changeset measures nothing, and reading it
/// as "clean" is the deflation the audit's own `skipped` key exists
/// to stop.
enum Judged {
    Measured(Box<Measured>),
    Skipped(&'static str),
}

/// One judged changeset: the pairs as the report names them, the
/// core's verdict, and the newly-duplicated span count per pair (the
/// evidence the aligner ships and the rule weighs).
struct Measured {
    files: Vec<String>,
    batch: BatchClassification,
    spans: Vec<usize>,
}

/// The slice/labels contract pair of one corpus.
fn docs_of(name: &str) -> (String, String) {
    let suffix = if name == "self" {
        String::new()
    } else {
        format!("-{name}")
    };
    (
        format!("../contracts/eval/commit-slice{suffix}-v1.json"),
        format!("../contracts/eval/commit-labels{suffix}-v1.json"),
    )
}

/// The checkout of one corpus: this repository, or the pinned
/// external clone every other external-corpus leg resolves through.
fn corpus_root(name: &str) -> PathBuf {
    if name == "self" {
        return common::repo_root();
    }
    let (_, tip) = crate::eval_support::PINNED_CORPORA
        .iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("{name}: not a pinned corpus"));
    crate::eval_support::pinned_root(name, tip)
}

/// One commit as the Stop audit would see it: the family's own pair
/// builder over `<sha>^ .. <sha>`, both blob sides in one batch, then
/// the audit's own judgment seam.
fn judge_commit(judge: &mut Judge, root: &Path, sha: &str) -> Judged {
    let parent = format!("{sha}^");
    if !common::revision_exists(root, &parent) {
        return Judged::Skipped("no_parent");
    }
    let Some(pairs) = session::commit_pairs(root, sha) else {
        return Judged::Skipped("no_pairs");
    };
    let Some((loaded, unread)) = texts::load(root, &pairs, Side::Rev(&parent), Side::Rev(sha))
    else {
        return Judged::Skipped("no_texts");
    };
    if loaded.is_empty() {
        return Judged::Skipped("no_judged_files");
    }
    if unread > 0 {
        return Judged::Skipped("partial");
    }
    let inputs = crate::eval_l2_edges_parts::pair_inputs(&loaded);
    let spans: Vec<usize> = inputs.iter().map(|i| dup_spans(i).len()).collect();
    let batch = judge.judge_changeset(&inputs);
    if batch.degraded.is_some() {
        return Judged::Skipped("degraded");
    }
    Judged::Measured(Box::new(Measured {
        files: loaded.iter().map(|l| l.rel.clone()).collect(),
        batch,
        spans,
    }))
}

/// Every intercepted (commit, firing pair) as one ledger row.
fn rows_of(corpus: &str, sha: &str, m: &Measured, label: Label) -> Vec<Row> {
    let moved_cross: usize = m.batch.relocations.iter().map(|r| r.lines).sum();
    m.batch
        .suspicions
        .iter()
        .map(|(i, rule)| Row {
            corpus: corpus.to_string(),
            sha: sha.to_string(),
            pairs: m.files.len(),
            file: m.files.get(*i).cloned().unwrap_or_default(),
            moved_cross,
            novel: m.batch.pairs[*i].counts.added_novel,
            deleted: m.batch.pairs[*i].counts.removed_deleted,
            dup_spans: m.spans.get(*i).copied().unwrap_or(0),
            verdict: rule.clone(),
            label,
        })
        .collect()
}

/// One corpus replayed in frozen slice order.
fn replay(name: &str, judge: &mut Judge, rows: &mut Vec<Row>) -> Tally {
    let (slice, labels) = docs_of(name);
    let root = corpus_root(name);
    let tip = parts::universe_tip(&slice);
    let mut gt = parts::ground_truth(&root, &slice, &labels);
    if let Some(n) = std::env::var("CE_L2FPR_LIMIT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n < gt.len())
    {
        gt = gt.split_off(gt.len() - n);
    }
    let mut t = Tally::new(name, &tip, gt.len());
    for (sha, label) in &gt {
        match judge_commit(judge, &root, sha) {
            Judged::Skipped(why) => *t.skipped.entry(why).or_default() += 1,
            Judged::Measured(m) => {
                let fired = !m.batch.suspicions.is_empty();
                t.count(*label, fired);
                if fired {
                    rows.extend(rows_of(name, sha, &m, *label));
                }
            }
        }
    }
    t
}

#[test]
#[ignore = "history instrument: replays three frozen commit slices through a live core, minutes; run by hand"]
fn l2_fpr_replay() {
    let _ = common::core_bin(); // name CE_CORE_BIN before a degraded run
    let only = std::env::var("CE_L2FPR_CORPUS").ok();
    let names: Vec<&str> = CORPORA
        .iter()
        .copied()
        .filter(|n| only.as_deref().is_none_or(|o| o == *n))
        .collect();
    assert!(!names.is_empty(), "CE_L2FPR_CORPUS names no frozen corpus");
    if crate::facts::blessing() {
        assert_eq!(names, CORPORA, "freeze all three corpora in one sitting");
        assert!(
            std::env::var_os("CE_L2FPR_LIMIT").is_none(),
            "a smoke run cannot freeze the ledger"
        );
    }
    let mut judge = Judge::default();
    let (mut tallies, mut rows) = (Vec::new(), Vec::new());
    for name in &names {
        tallies.push(replay(name, &mut judge, &mut rows));
    }
    parts::report(&tallies, &rows);
    if crate::facts::blessing() {
        let doc = parts::document(&tallies, &rows);
        let path = crate::eval_support::eval_doc("fpr-l2");
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&doc).expect("json") + "\n",
        )
        .expect("write the l2 fpr contract");
        println!("wrote {path}");
    }
}

/// The three synthetic commits: (message, alpha.rs, beta.rs, an
/// intercept is expected, a cross-file relocation is expected). The
/// fixture is TEXT, never compiled — commit C writes a second
/// top-level `compute_total`, which is the very shape the stacking
/// rule exists for and which rustc would of course refuse.
const E2E: [(&str, &str, &str, bool, bool); 3] = [
    ("seed", fixtures::ALPHA_A, fixtures::BETA_A, false, false),
    ("relocate", fixtures::ALPHA_B, fixtures::BETA_B, false, true),
    ("stack", fixtures::ALPHA_C, fixtures::BETA_B, true, false),
];

/// The predicate, end to end through the seam the audit calls: moving
/// a function across files is NOT an intercept, writing a second copy
/// of a unit beside the first IS.
#[test]
fn a_relocation_is_not_an_intercept_and_a_stacked_copy_is() {
    let _ = common::core_bin();
    let dir = common::tmp("l2-fpr-e2e");
    common::git(&dir, &["init", "-q"]);
    let mut judge = Judge::default();
    let mut shas = Vec::new();
    for (msg, alpha, beta, _, _) in E2E {
        std::fs::write(dir.join("alpha.rs"), alpha).expect("alpha.rs");
        std::fs::write(dir.join("beta.rs"), beta).expect("beta.rs");
        common::commit_all(&dir, msg);
        shas.push(common::git_lines(&dir, &["rev-parse", "HEAD"])[0].clone());
    }
    for (i, (msg, .., want_fire, want_move)) in E2E.iter().enumerate().skip(1) {
        let Judged::Measured(m) = judge_commit(&mut judge, &dir, &shas[i]) else {
            panic!("{msg}: the changeset must be judged — export CE_CORE_BIN");
        };
        let got = (
            !m.batch.suspicions.is_empty(),
            !m.batch.relocations.is_empty(),
        );
        assert_eq!(got, (*want_fire, *want_move), "{msg}: (intercept, moved)");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
