//! The relocation channel of the duplicate-write replay (v2.35 guard
//! lane, GD4; REVIEW-1 Q7): an instrument that lacks a veto channel the
//! product has records the product's allowances as intercepts. Every
//! intercepted event carries two readings beside the standing one:
//!
//! - `relocated` — the product's own stage at the write's moment in the
//!   walk's file order: HEAD = the parent commit, the working tree = the
//!   shadow mid-commit (the commit's files already applied), the counts
//!   through the product's counter (`dedup::relocate::occurrences`) and
//!   the verdict the core's (relocate/1), asked per event exactly as the
//!   hook asks it: every novel match a relocation, or no credit.
//! - `count_neutral` — order-free: the commit as a whole did not raise
//!   the tree's count of the row's runs (child tree ≤ parent tree over
//!   the files it changed). Inside one commit the true write order is
//!   unknown; an author who removed the source first is this reading.
//!
//! The strict reading keeps counting write-first rows as intercepts (the
//! K-round ruling: both readings stand, recorded side by side).

use super::Intercept;
use crate::common::blob;
use codeeraser::corelink::{Link, judged};
use codeeraser::dedup::relocate::{self, CAP, KIND, occurrences};
use codeeraser::dedup::tokens;
use codeeraser::scan::lang::Lang;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// A content's token hashes in its language.
pub fn hashes(bytes: &[u8], lang: Lang) -> Vec<u64> {
    tokens::stream(bytes, lang)
        .expect("tokens")
        .iter()
        .map(|t| t.hash)
        .collect()
}

/// The commit's changed fingerprinting files as token streams: the
/// parent side (every path the parent had) and the child side.
pub struct CommitStreams {
    parent: BTreeMap<String, Vec<u64>>,
    child: BTreeMap<String, Vec<u64>>,
}

impl CommitStreams {
    pub fn load(repo: &Path, (parent, commit): (&str, &str), touched: &BTreeMap<&str, char>) -> Self {
        let mut s = CommitStreams {
            parent: BTreeMap::new(),
            child: BTreeMap::new(),
        };
        for (rel, status) in touched {
            let lang = Lang::judged_path(Path::new(rel)).expect("a judged file");
            if *status != 'A' {
                s.parent.insert(rel.to_string(), hashes(&blob(repo, parent, rel), lang));
            }
            if *status != 'D' {
                s.child.insert(rel.to_string(), hashes(&blob(repo, commit, rel), lang));
            }
        }
        s
    }

    fn occ(side: &BTreeMap<String, Vec<u64>>, files: impl Iterator<Item = String>, run: &[u64]) -> u64 {
        files
            .filter_map(|f| side.get(&f))
            .map(|s| occurrences(s, run))
            .sum()
    }

    /// Order-free: the whole commit did not raise the run's count.
    pub fn neutral(&self, run: &[u64]) -> bool {
        let child = Self::occ(&self.child, self.child.keys().cloned(), run);
        let parent = Self::occ(&self.parent, self.parent.keys().cloned(), run);
        child <= parent
    }

    /// At the write's moment: `[occHead, occNow]` over the files the
    /// commit applied before it (`before`) and the target `t` — the
    /// hook's formula with HEAD = the parent, the tree = the shadow.
    pub fn at_write(&self, run: &[u64], before: &BTreeSet<String>, t: &str) -> [u64; 2] {
        let with_t = before.iter().cloned().chain(std::iter::once(t.to_string()));
        let head = Self::occ(&self.parent, with_t, run);
        let others = before.iter().filter(|f| *f != t).cloned();
        let now = Self::occ(&self.child, others, run) + Self::occ(&self.child, std::iter::once(t.to_string()), run);
        [head, now]
    }
}

/// The two readings filled in for one commit's rows, once the commit is
/// whole: per row `count_neutral`, per event `relocated` (the core's
/// verdict over every run the event raised, as the hook asks it).
pub fn read(link: &mut Link, streams: &CommitStreams, rows: &mut [Intercept]) {
    for r in rows.iter_mut() {
        r.count_neutral = r.runs.iter().all(|run| streams.neutral(run));
    }
    let events: BTreeSet<String> = rows.iter().map(|r| r.rel.clone()).collect();
    for rel in events {
        let mine: Vec<&Intercept> = rows.iter().filter(|r| r.rel == rel).collect();
        let counts: Vec<[u64; 2]> = mine
            .iter()
            .flat_map(|r| r.runs.iter().map(|run| streams.at_write(run, &r.applied_before, &rel)))
            .collect();
        let reply = judged::ask(link, CAP, KIND, relocate::body(&counts)).expect("relocate/1");
        let credited = relocate::consume(&reply, counts.len()).expect("a verdict").all;
        for r in rows.iter_mut().filter(|r| r.rel == rel) {
            r.relocated = credited;
        }
    }
}
