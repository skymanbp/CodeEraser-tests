//! FPR replay of the duplicate-write class PER LANGUAGE, over each
//! language's own history — the release gate the design booklet
//! (docs/reference/language-expansion.md §14 item 9) demands of every
//! language plan v2.30 adds: a language ships its judged bit only on
//! its own ledger, read at or under the §4.2 line. The standing ledger
//! (fpr_replay.rs) walks this repository and requests, which hold no
//! file of the six; this instrument walks one corpus per language —
//! the first exam corpus at the exam's pinned tip, so the corpus and
//! the tip are the exam registry's and nothing else — over the WINDOW
//! newest first-parent commits, and counts only that language's files
//! as events. The walk is fpr_replay's own (`replay` with the language
//! named), so the two ledgers cannot disagree about what an event or
//! an intercept is.
//!
//!   CE_FPR_CORPUS=lua cargo test --release --test it -- --ignored fpr_lang_replay --nocapture
//!
//! CE_BLESS=1 merges the measured row into
//! contracts/eval/fpr-lang-v1.json, replacing the row of the same name
//! and keeping the other five as their own runs left them.
//!
//! The reading (docs/FPR-REPLAY.md, the per-language section):
//! WINDOW commits; an event is one changed file of the language in one
//! first-parent commit; an intercepted event is one the probe raised a
//! row for; it LANDED when a row still stands once the commit is
//! whole (a verified duplication the parent did not have — the
//! ledger's true-positive class); otherwise its twins were TOUCHED
//! when every twin it named was modified or deleted in the same
//! commit — the write-first split, fold or rename and the same-commit
//! reformat, the class the K round arbitrated: at the write instant a
//! copy and a move are the same bytes, the refusal teaches the safe
//! order, and the standing ledger records the class beside the false
//! count, never inside it; what remains is a FALSE intercept, and the
//! line is GATE_PPM false intercepts per million events, the graded
//! zone's. The strict reading — every mid-state counted as false, the
//! whole-write framing — is frozen and printed beside it in the
//! standing ledger's habit (双口径并记), never folded into one number.
//! HTML never fingerprints (lang.rs `fingerprints`), so its row reads
//! zero events by construction and the gate admits it by name.

use crate::common::ledger::{self, Ledger, corpus_at};
use crate::common::stats::{cp_upper_ppm, rate_ppm};
use crate::common::{chain_of, repo_root, tmp};
use crate::eval_lang_parts::exam_of_corpus;
use crate::eval_support::{lang_of, pinned_root};
use crate::fpr_replay::replay;
use crate::fpr_replay_parts::Intercept;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const DOC: &str = "contracts/eval/fpr-lang-v1.json";
pub const SCHEMA: &str = "ce.eval-fpr-lang/1.0.0";
/// The §4.2 line is one line: the graded zone's.
pub use crate::fpr_zone_replay_parts::GATE_PPM;
/// The newest first-parent commits each corpus is read over.
pub const WINDOW: usize = 400;

/// The six languages and the corpus each is read over — the first
/// exam corpus of each (eval_lang_parts::EXAMS), `<lang>=<corpus>` in
/// the ledger's row order, the language spelled as the registry
/// spells it. One literal, not a pair table: a pair table is this
/// repository's most self-rhyming token shape (the guard matched a
/// six-row one against similar_replay's five at the first write).
pub const CORPORA: &str = "c=lua cpp=fmt java=gson lua=luarocks r=stringr html=codeeraser";

/// The ledger's `(lang, corpus)` rows, in order.
pub fn corpora() -> Vec<(&'static str, &'static str)> {
    CORPORA
        .split_whitespace()
        .map(|p| p.split_once('=').expect("lang=corpus"))
        .collect()
}

/// The language a ledger corpus is read for, and its row's rank.
pub fn language_of(corpus: &str) -> (&'static str, usize) {
    corpora()
        .iter()
        .enumerate()
        .find(|(_, (_, n))| *n == corpus)
        .map(|(i, (l, _))| (*l, i))
        .unwrap_or_else(|| panic!("{corpus}: no ledger corpus (fpr_lang_replay::CORPORA)"))
}

pub const LEDGER: Ledger = Ledger {
    rel: DOC,
    schema: SCHEMA,
    generated_from: "cli/tests/it/fpr_lang_replay.rs",
    gate_ppm: GATE_PPM,
};

/// One row's event arithmetic, read alike by the instrument and its
/// gate — one owner, so the frozen row and the gate's recomputation
/// cannot drift apart.
#[derive(Debug, PartialEq, Eq)]
pub struct Fold {
    pub intercepted: usize,
    pub landed: usize,
    /// Mid-states whose every twin was touched in the same commit.
    pub touched: usize,
}

impl Fold {
    /// Intercepted events with no row standing once the commit is
    /// whole — the strict reading's false count.
    pub fn mid_state(&self) -> usize {
        self.intercepted - self.landed
    }

    /// Mid-states with a twin the commit left alone: the false
    /// intercepts.
    pub fn false_events(&self) -> usize {
        self.mid_state() - self.touched
    }
}

/// Over `(commit, file, landed, twin touched)` rows: the distinct
/// events intercepted, those with a row standing once the commit was
/// whole, and those with none standing whose every twin was touched
/// in the same commit.
pub fn fold(rows: impl Iterator<Item = (String, String, bool, bool)>) -> Fold {
    let mut events: BTreeMap<(String, String), (bool, bool)> = BTreeMap::new();
    for (commit, rel, landed, touched) in rows {
        let e = events.entry((commit, rel)).or_insert((false, true));
        e.0 |= landed;
        e.1 &= touched;
    }
    Fold {
        intercepted: events.len(),
        landed: events.values().filter(|(l, _)| *l).count(),
        touched: events.values().filter(|(l, t)| !l && *t).count(),
    }
}

/// One corpus as the walk finished it.
pub struct Reading {
    pub events: usize,
    pub rows: Vec<Intercept>,
}

impl Reading {
    pub fn folded(&self) -> Fold {
        fold(self.rows.iter().map(|r| {
            (
                r.commit.clone(),
                r.rel.clone(),
                r.landed,
                r.twin_status.is_some(),
            )
        }))
    }

    /// The frozen row. Every derived number is recomputed by the gate
    /// from the rows, so a hand-edited doc reddens.
    pub fn json(&self, lang: &str, name: &str, tip: &str, commits: usize) -> Value {
        let f = self.folded();
        let (k, strict, n) = (f.false_events(), f.mid_state(), self.events);
        json!({
            "name": name,
            "lang": lang,
            "tip": &tip[..8],
            "commits": commits,
            "events": n,
            "intercepted": f.intercepted,
            "landed": f.landed,
            "touched": f.touched,
            "false_events": k,
            "rate_ppm": rate_ppm(k, n),
            "cp_upper_ppm": cp_upper_ppm(k, n),
            "strict_ppm": rate_ppm(strict, n),
            "strict_cp_upper_ppm": cp_upper_ppm(strict, n),
            "intercepts": self.rows,
        })
    }
}

/// The ledger table docs/FPR-REPLAY.md carries — one row per
/// language, printed by the instrument for the maintainer to paste.
pub fn table(corpora: &[Value]) -> String {
    ledger::table(
        "| 语言 | 语料 | 提交 | 事件 | 被拦事件 | 落地 | 孪生被动 | 误拦 | 率 | CP 95 % 上界 | 严格率 | 严格 CP 95 % 上界 |\n|---|---|---|---|---|---|---|---|---|---|---|---|\n",
        |c| format!("{} | {}", c["lang"].as_str().unwrap_or("?"), corpus_at(c)),
        "commits events intercepted landed touched false_events %rate_ppm %cp_upper_ppm %strict_ppm %strict_cp_upper_ppm",
        corpora,
    )
}

#[test]
#[ignore = "history instrument: replays one exam corpus's newest first-parent commits, minutes; run by hand"]
fn every_language_is_read_over_its_own_history() {
    let name = std::env::var("CE_FPR_CORPUS").expect("CE_FPR_CORPUS names the corpus (CORPORA)");
    let (code, _) = language_of(&name);
    let (_, tip) = exam_of_corpus(&name);
    let repo = pinned_root(&name, tip);
    let commits = chain_of(&repo, tip, Some(WINDOW));
    let shadow = tmp(&format!("fpr-lang-{name}"));
    let (events, rows) = replay(&repo, &shadow, Some(lang_of(code)), &commits);
    let r = Reading { events, rows };
    for row in &r.rows {
        println!("  INTERCEPT {}", row.row());
    }
    let row = r.json(code, &name, tip, commits.len() - 1);
    println!("{}", table(std::slice::from_ref(&row)));
    let f = r.folded();
    println!(
        "walked {} commits of {name} at {}: {} {code} events, {} intercepted, {} landed, {} twin-touched, {} false",
        commits.len() - 1,
        repo.display(),
        r.events,
        f.intercepted,
        f.landed,
        f.touched,
        f.false_events()
    );
    if crate::facts::blessing() {
        LEDGER.merge(&repo_root(), row, &[("window", json!(WINDOW))], |c| {
            (
                language_of(c["name"].as_str().unwrap_or("?")).1,
                String::new(),
            )
        });
    }
}
