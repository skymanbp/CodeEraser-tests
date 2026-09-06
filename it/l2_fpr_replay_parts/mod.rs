//! The ledger half of the changeset FPR replay (l2_fpr_replay.rs):
//! the per-commit ground truth read off the frozen commit contracts,
//! the tally each corpus contributes, the Clopper-Pearson interval
//! those tallies carry, the frozen document, and the e2e fixture
//! texts. Split from the walk so neither file passes the suite's own
//! 300-line wall.
pub mod fixtures;
pub mod render;
pub mod tally;
pub use render::{document, report};
pub use tally::Tally;

use crate::eval_support::{by_sha, load};
use serde_json::json;
use std::collections::BTreeSet;
use std::path::Path;

/// The frozen contract this instrument writes.
pub const SCHEMA: &str = "ce.eval-fpr-l2/1.0.0";

/// The plan's §4.2 line, as a percentage of the wide denominator.
pub const GATE_MAX_PERCENT: u64 = 1;

/// What the reviewed commit contracts say one commit is.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Label {
    /// a slice row carrying a `copied` pair: the whole-file copy, the
    /// only duplication-shaped positive the three corpora hold
    Copy,
    /// reviewed line by line against the raw diff, and not a copy
    Normal,
    /// admitted by the slice, never individually reviewed for
    /// duplication — a WIDER denominator, never counted as reviewed
    Unreviewed,
}

/// The universe tip a slice froze against.
pub fn universe_tip(slice_path: &str) -> String {
    load(slice_path)["universe_tip"]
        .as_str()
        .expect("universe_tip")
        .to_string()
}

/// One corpus's ground truth: the slice's admitted shas in frozen
/// order, each with its label. Nothing is invented here — `copied`
/// comes from the slice, `normal` from the presence of a reviewed
/// labels row, and everything else says so by name. Every SHA must
/// belong to the frozen tip's first-parent chain; current HEAD is
/// not that anchor (the self corpus predates a history rewrite).
pub fn ground_truth(root: &Path, slice_path: &str, labels_path: &str) -> Vec<(String, Label)> {
    let labels = load(labels_path);
    let reviewed: BTreeSet<String> = by_sha(&labels).keys().map(|s| (*s).to_string()).collect();
    let tip = universe_tip(slice_path);
    let history: BTreeSet<_> = crate::common::first_parent(root, &tip)
        .into_iter()
        .collect();
    load(slice_path)["commits"]
        .as_array()
        .expect("slice commits")
        .iter()
        .map(|c| {
            let sha = c["sha"].as_str().expect("sha").to_string();
            assert!(
                history.contains(&sha),
                "{slice_path}: {sha} is outside {tip}'s first-parent chain"
            );
            let copied = c["pairs"]
                .as_array()
                .expect("pairs")
                .iter()
                .any(|p| p["copied"] == json!(true));
            let label = match (copied, reviewed.contains(&sha)) {
                (true, _) => Label::Copy,
                (false, true) => Label::Normal,
                (false, false) => Label::Unreviewed,
            };
            (sha, label)
        })
        .collect()
}

/// One intercepted (commit, firing pair): what a human reads to
/// arbitrate. `inDup` — the novel mass inside a duplicated span — is
/// the rule's own third arm and stays in CE.FourClass.Verdict; the
/// three integers here are the Rust-side measurements the aligner
/// already ships, which is what opens the diff.
#[derive(serde::Serialize)]
pub struct Row {
    pub corpus: String,
    pub sha: String,
    pub pairs: usize,
    pub file: String,
    pub moved_cross: usize,
    pub novel: usize,
    pub deleted: usize,
    pub dup_spans: usize,
    pub verdict: String,
    pub label: Label,
}
