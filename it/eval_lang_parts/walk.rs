//! The product's own walk over a frozen universe, as the language
//! exams score it (plan v2.30 step 5): which universe files the walk
//! refuses, how a truth naming one is scored, and the doc's record of
//! them — one reading for the scorer (score.rs), the verifier
//! (precision.rs), the generator (generate.rs) and the tamper frame's
//! oracle (tamper.rs).

use super::Exam;
use crate::eval_support::{TRUTH_KEYWORDS, site_summary};
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// The product's own walk over one frozen universe: the universe files
/// it refuses at the pinned tip — vendored code, a build tool's output,
/// a path the tree's ignore files fence off (scan/walk.rs) — and the
/// ones it reads. An exam measures the product on what it reads: the
/// ladder resolves against the walked files only, the ledger counts
/// their sites only, and an audit truth naming a refused target is one
/// the product has no node for, so it is scored as outside the corpus
/// ("external": an External answer or a refusal is right, an in-corpus
/// answer wrong), the audit's own word kept beside it.
pub struct Walk {
    pub refused: BTreeSet<String>,
    pub walked: BTreeSet<String>,
}

impl Walk {
    pub fn new<'a>(universe: impl IntoIterator<Item = &'a str>, refused: BTreeSet<String>) -> Self {
        let walked = universe
            .into_iter()
            .filter(|p| !refused.contains(*p))
            .map(str::to_string)
            .collect();
        Self { refused, walked }
    }

    /// Whether a truth names a target the walk refuses: a refused file
    /// (or a unit inside one), or a directory holding refused files and
    /// no walked one.
    pub fn unwalked(&self, truth: &str) -> bool {
        if TRUTH_KEYWORDS.contains(&truth) {
            return false;
        }
        let file = truth.split('#').next().unwrap_or(truth);
        let dir = format!("{file}/");
        let under = |set: &BTreeSet<String>| set.iter().any(|f| f.starts_with(&dir));
        self.refused.contains(file) || (under(&self.refused) && !under(&self.walked))
    }

    /// A truth as scored: an unwalked target reads as "external".
    pub fn scored<'t>(&self, truth: &'t str) -> &'t str {
        if self.unwalked(truth) {
            "external"
        } else {
            truth
        }
    }
}

/// The doc's record of the walk: the refused files and their sites per
/// (lang, kind) — the gate re-derives the tally from the frozen slice's
/// own rows, so the ledger and the refused sites add up to the universe.
pub fn walk_record(slice: &Value, refused: &BTreeSet<String>) -> Value {
    let rows: Vec<Value> = slice["files"]
        .as_array()
        .expect("files")
        .iter()
        .filter(|r| refused.contains(r["path"].as_str().expect("path")))
        .cloned()
        .collect();
    json!({"refused": refused, "refused_sites_by": site_summary(&rows)["sites_by"]})
}

/// Whether a path is of the exam's language by extension — the
/// universe walk's first test (generate.rs walk) and the tree gate's
/// partition of what a slice left out (tree.rs).
pub fn in_scope(exam: &Exam, path: &str) -> bool {
    let ext = path.rsplit_once('.').map_or("", |(_, e)| e);
    exam.exts.contains(&ext)
}

/// The frozen universe's files, in the slice's order — what a walk
/// record's refused files must be drawn from.
pub fn universe_files(slice: &Value) -> Vec<&str> {
    slice["files"]
        .as_array()
        .expect("files")
        .iter()
        .map(|r| r["path"].as_str().expect("path"))
        .collect()
}
