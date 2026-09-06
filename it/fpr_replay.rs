//! FPR replay for the duplicate-write class over a git history — the
//! M3 acceptance instrument (docs/FPR-REPLAY.md: ≤ 1 false intercept
//! per 500 real edits), standing again (plan v2.29 step 9, O49). The
//! root harness retired with M7.5 and was revived twice from history
//! with hand-applied shims; the K-round revival left 23 re-ignitions
//! of already-fired pairs classified "true positive by leaning", with
//! no measurement behind the leaning, and 32 mid-states arbitrated by
//! hand with two-file fixtures. This leg carries both measurements:
//! beside every intercept it probes the PARENT content too, so the
//! child's shared run with each twin is read against the parent's —
//! an edit that grew the duplication (extension) is told from one that
//! only moved it (drift) and from a pair's first fire (new) — and once
//! the commit is whole it probes the child AGAIN against the child
//! state, so a run gone by then (the write-first split, fold or rename
//! the guard teaches the safe ordering for) is told from duplication
//! that landed. The twin's fate in the same commit rides along.
//!
//! Every changed judged-language file of every first-parent commit is
//! one edit event: the child blob probed against the PARENT-state
//! index (the whole-content Write model), the guard's novelty
//! subtraction applied (matches the parent's own content already had
//! are carried, not written), then the shadow tree and the index
//! advance. A history instrument like tombstone_replay, so it stays an
//! ignored leg under the EVAL-SET retirement rule:
//!
//!   cargo test --release --test it -- --ignored fpr_replay --nocapture
//!
//! CE_FPR_REPO = the checkout to walk (default: this repo); the chain
//! and the two knobs that bound it live in common::history, shared
//! with the graded-zone ledger so the two instruments walk exactly
//! the same commits.

use crate::common::{blob, chain, changed, git_lines, repo_root, tmp};
use crate::fpr_replay_parts::{Class, Intercept, pair, report};
use codeeraser::dedup::pairs::{DEFAULT_MIN_DISTINCT, Filter};
use codeeraser::dedup::probe::{self, Match, Target};
use codeeraser::dedup::{Params, index::Index};
use codeeraser::scan::lang::Lang;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The files the T1/T2 probe can read, as the daemon's probe reply
/// decides it (daemon/server/replies.rs: a grammar-less language
/// answers no matches): judged languages, never the scan-only arm,
/// with a token stream — Markdown is judged but grammar-less, and its
/// duplication is docdup's, not this class's.
fn judged(rel: &str) -> Option<Lang> {
    Lang::judged_path(Path::new(rel)).filter(|l| l.grammar().is_some())
}

/// One file into the shadow tree and the index — the tree the next
/// probe reads its candidate streams from.
fn apply(shadow: &Path, idx: &mut Index, rel: &str, bytes: &[u8], lang: Lang, p: Params) {
    let dest = shadow.join(rel);
    std::fs::create_dir_all(dest.parent().expect("a parent")).expect("mkdir");
    std::fs::write(&dest, bytes).expect("write shadow");
    idx.refresh_file(rel, bytes, lang, p, false)
        .expect("refresh");
}

/// The seed: every judged file of the first commit's tree.
fn seed(repo: &Path, shadow: &Path, idx: &mut Index, first: &str, p: Params) -> BTreeSet<String> {
    let mut live = BTreeSet::new();
    for rel in git_lines(repo, &["ls-tree", "-r", "--name-only", first]) {
        if let Some(lang) = judged(&rel) {
            apply(shadow, idx, &rel, &blob(repo, first, &rel), lang, p);
            live.insert(rel);
        }
    }
    live
}

/// The guard's split (guard/probe.rs `carried`): a match whose twin
/// region overlaps one the parent content already had is carried
/// forward, not written anew.
fn novel(child: Vec<Match>, parent: &[Match]) -> Vec<Match> {
    child
        .into_iter()
        .filter(|m| {
            !parent.iter().any(|b| {
                b.file == m.file && b.start_line <= m.end_line && m.start_line <= b.end_line
            })
        })
        .collect()
}

/// Shared tokens per twin — the sum over every verified run with that
/// file, the quantity an extension grows and a move leaves alone.
fn shared(ms: &[Match]) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for m in ms {
        *out.entry(m.file.clone()).or_default() += m.tokens;
    }
    out
}

/// The replay's fixed context: the repository, the shadow tree and
/// the operating point every probe reads at.
struct Walk<'a> {
    repo: &'a Path,
    shadow: &'a Path,
    p: Params,
    f: Filter,
}

impl Walk<'_> {
    fn probe(&self, idx: &Index, rel: &str, content: &[u8], lang: Lang) -> Vec<Match> {
        let target = Target { rel, content, lang };
        probe::probe(idx, self.shadow, target, self.p, self.f).expect("probe")
    }
}

/// One commit under replay: its parent, itself, and what it did to
/// every judged file (the twin's fate is read off this table).
struct Commit<'a> {
    parent: &'a str,
    commit: &'a str,
    touched: BTreeMap<&'a str, char>,
}

/// One edit event's rows: the child probed against the parent state,
/// the parent's own matches subtracted, one row per twin — `landed`
/// waits for the commit to be whole (settle).
fn read_event(
    w: &Walk,
    idx: &Index,
    c: &Commit,
    (rel, bytes, lang, status): (&str, &[u8], Lang, char),
    fired: &mut BTreeSet<(String, String)>,
    rows: &mut Vec<Intercept>,
) {
    let child = w.probe(idx, rel, bytes, lang);
    let base = if status == 'M' && !child.is_empty() {
        w.probe(idx, rel, &blob(w.repo, c.parent, rel), lang)
    } else {
        Vec::new()
    };
    let (before, after) = (shared(&base), shared(&child));
    for (twin, novel_tokens) in shared(&novel(child, &base)) {
        let (pt, ct) = (before.get(&twin).copied().unwrap_or(0), after[&twin]);
        rows.push(Intercept {
            commit: c.commit[..8].to_string(),
            rel: rel.to_string(),
            twin: twin.clone(),
            parent: pt,
            child: ct,
            novel: novel_tokens,
            class: Class::of(pt, ct),
            refire: !fired.insert(pair(rel, &twin)),
            twin_status: c.touched.get(twin.as_str()).copied(),
            landed: false,
        });
    }
}

/// The commit whole: every row it raised probed again against the
/// child state. The run is either still there (landed) or gone — a
/// write-first mid-state, the twin trimmed, folded or deleted.
fn settle(
    w: &Walk,
    idx: &Index,
    children: &BTreeMap<String, (Vec<u8>, Lang)>,
    rows: &mut [Intercept],
) {
    for r in rows {
        let (bytes, lang) = &children[&r.rel];
        r.landed = w
            .probe(idx, &r.rel, bytes, *lang)
            .iter()
            .any(|m| m.file == r.twin);
    }
}

/// One commit's events: probe, read against the parent, advance, then
/// settle once the commit is whole.
fn replay_commit(
    w: &Walk,
    idx: &mut Index,
    (parent, commit): (&str, &str),
    live: &mut BTreeSet<String>,
    fired: &mut BTreeSet<(String, String)>,
    rows: &mut Vec<Intercept>,
) -> usize {
    let ch: Vec<(char, String)> = changed(w.repo, parent, commit)
        .into_iter()
        .filter(|(_, rel)| judged(rel).is_some())
        .collect();
    let c = Commit {
        parent,
        commit,
        touched: ch.iter().map(|(s, r)| (r.as_str(), *s)).collect(),
    };
    let (first, mut events) = (rows.len(), 0);
    let mut children = BTreeMap::new();
    for (status, rel) in &ch {
        if *status == 'D' {
            live.remove(rel);
            let _ = std::fs::remove_file(w.shadow.join(rel));
            continue;
        }
        let lang = judged(rel).expect("judged");
        let bytes = blob(w.repo, commit, rel);
        events += 1;
        let before = rows.len();
        read_event(w, idx, &c, (rel, &bytes, lang, *status), fired, rows);
        if rows.len() > before {
            children.insert(rel.clone(), (bytes.clone(), lang));
        }
        apply(w.shadow, idx, rel, &bytes, lang, w.p);
        live.insert(rel.clone());
    }
    let seen = idx.indexed_paths().expect("indexed paths");
    idx.remove_missing(live, &seen).expect("reap");
    settle(w, idx, &children, &mut rows[first..]);
    events
}

#[test]
#[ignore = "history instrument: replays a repository's first-parent chain, minutes; run by hand"]
fn every_intercept_is_read_against_its_parent_baseline() {
    let repo = std::env::var_os("CE_FPR_REPO").map_or_else(repo_root, PathBuf::from);
    let shadow = tmp("fpr-shadow");
    let p = Params::default();
    let w = Walk {
        repo: &repo,
        shadow: &shadow,
        p,
        f: Filter {
            min_tokens: p.guarantee(),
            min_distinct: DEFAULT_MIN_DISTINCT,
        },
    };
    let mut idx = Index::open(&shadow.join(".ce/index.db"), p).expect("open index");
    let commits = chain(&repo);
    let mut live = seed(&repo, &shadow, &mut idx, &commits[0], p);
    let (mut fired, mut rows, mut events) = (BTreeSet::new(), Vec::new(), 0usize);
    for step in commits.windows(2) {
        events += replay_commit(
            &w,
            &mut idx,
            (&step[0], &step[1]),
            &mut live,
            &mut fired,
            &mut rows,
        );
    }
    report(events, &rows);
}
