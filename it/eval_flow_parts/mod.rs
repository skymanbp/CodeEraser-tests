//! The v2.31 flow exams (design booklet analysis-track.md §5.5, §13
//! item 16; registry docs/EVAL-SET-FLOW.md): the exam table (exams.rs),
//! its doc families and the pre-registered constants, the candidate
//! pools read off a lowered unit's tables (pools.rs) and the per-cell
//! hash-ranked draw (draw.rs) — ONE binding for the `#[ignore]`
//! generators (generate.rs) and the CI gates (eval_flow.rs, through
//! the verifier in verify.rs and the tamper battery in tamper.rs). The
//! shape is the language exams' (eval_lang_parts), whose doc family
//! type, git and clone helpers and tamper frame it reads; what differs
//! is what a question is — a statement, a write or a declaration of
//! one unit, classified by kind and stratum from the lowering's own
//! rows, never from the core's verdict.

pub mod draw;
pub mod exams;
pub mod generate;
pub mod pools;
pub mod tamper;
pub mod verify;

use crate::eval_lang_parts::{Docs, Generated};
use codeeraser::scan::lang::Lang;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub use crate::eval_lang_parts::Stage;
pub use exams::{EXAMS, exam};

/// One language's flow exam: its corpora at their pinned tips, the
/// extensions its universe walks (the product's own row for the
/// language, eval_flow.rs holds them equal), how far it has come
/// (Stage), the generation every doc carries, and the lowering the
/// answers come from (`ladder`, pathspecs) with the reason it lands
/// before the sample (`ladder_first`).
pub struct FlowExam {
    pub lang: &'static str,
    pub corpora: Vec<(&'static str, &'static str)>,
    pub exts: Vec<&'static str>,
    pub stage: Stage,
    pub generation: u32,
    pub ladder: &'static [&'static str],
    pub ladder_first: &'static str,
}

impl Generated for FlowExam {
    fn generation(&self) -> u32 {
        self.generation
    }

    fn corpora(&self) -> &[(&'static str, &'static str)] {
        &self.corpora
    }
}

impl FlowExam {
    /// A slice doc's key: zod is sliced once per exam (typescript and
    /// tsx read one clone through two extension lists).
    pub fn key(&self, corpus: &str) -> String {
        format!("{}-{corpus}", self.lang)
    }

    /// Whether a path is of the exam's language by extension.
    pub fn in_scope(&self, path: &str) -> bool {
        let ext = path.rsplit_once('.').map_or("", |(_, e)| e);
        self.exts.contains(&ext)
    }

    /// The frozen scope of the exam's universe.
    pub fn scope(&self) -> Value {
        json!({"extensions": self.exts, "excludes": []})
    }

    /// The exam's frozen sample, its one per-language doc.
    pub fn sample(&self) -> Value {
        SAMPLES.load(self, self.lang)
    }

    /// The product's language of that report name.
    pub fn language(&self) -> Lang {
        Lang::with_grammar()
            .find(|l| l.name() == self.lang)
            .unwrap_or_else(|| panic!("{}: no grammar of that name", self.lang))
    }
}

/// The lowering and the wire — the code that answers a question. The
/// module table (`mod.rs`, the judged mask) is policy, not an answer,
/// so it is the one named exception under the directory.
pub const LOWERING: &[&str] = &["cli/src/flow/", ":!cli/src/flow/mod.rs"];

/// Why the lowering precedes the sample (booklet §13 item 16): a
/// question is drawn from the lowered tables, so there is no pool
/// before the lowering lands; the blindness rests on the process — the
/// auditors read the pinned clone and their batch, never a verdict.
pub const LADDER_FIRST: &str = "step 4 commit A landed the ten FlowSpec tables and the lowering \
    (5278e747, 2026-09-30) before any flow exam existed: the pools are read off the lowered \
    tables, so the lowering precedes the sample by construction and the blindness rests on the \
    process alone - the auditors read the pinned clone and their batch, never a verdict";

/// The frozen universes (per `<lang>-<corpus>`) and the samples (per
/// language).
pub const SLICES: Docs = Docs("flow-slice");
pub const SAMPLES: Docs = Docs("flow-sample");

/// (slice, sample)
pub const SCHEMAS: (&str, &str) = ("ce.eval-flow-slice/1.0.0", "ce.eval-flow-sample/1.0.0");

/// Pre-registered: each (kind, stratum) cell takes min(MIN_PER_STRATUM,
/// pool); no backups. The rank and the audit order use two unrelated
/// hash domains.
pub const MIN_PER_STRATUM: u64 = 15;
/// (site, audit)
pub const DOMAINS: (&str, &str) = ("ce-flow-site-v1", "ce-flow-audit-v1");

/// The rank payload, field for field; `name` last so the '|'-joined
/// encoding is injective (two declarations on one line share a
/// (line, nth) and differ in name alone).
pub const FIELDS: &str = "corpus commit path unit kind stratum line nth name";

pub fn fields() -> Vec<&'static str> {
    FIELDS.split(' ').collect()
}

/// The kinds and their strata — one text, read by everything that asks
/// which cells exist: `kind name A:why B:why [C:why]`.
pub const CLASSES: &str = "\
0 unreachable  A:after_terminal B:after_structure_with_exit C:after_structure
1 dead_store   A:next_is_write  B:last_access               C:other_write
2 unused_local A:no_read        B:read
3 unused_param A:no_read        B:read";

/// A kind's strata letters, in order; none for a kind outside the table.
pub fn strata(kind: u64) -> Vec<char> {
    CLASSES
        .lines()
        .filter(|l| l.starts_with(&format!("{kind} ")))
        .flat_map(|l| {
            l.split_whitespace()
                .skip(2)
                .map(|s| s.chars().next().expect("letter"))
        })
        .collect()
}

/// Every `kind/stratum` cell, in table order.
pub fn cells() -> Vec<String> {
    (0..4)
        .flat_map(|k| strata(k).into_iter().map(move |s| cell(k, s)))
        .collect()
}

pub fn cell(kind: u64, stratum: char) -> String {
    format!("{kind}/{stratum}")
}

pub fn slice_constants() -> Value {
    json!({"min_per_stratum": MIN_PER_STRATUM})
}

pub fn sample_constants() -> Value {
    json!({
        "min_per_stratum": MIN_PER_STRATUM,
        "domains": {"site": DOMAINS.0, "audit": DOMAINS.1},
    })
}

/// A slice's summary, re-derivable from its file rows alone (the G1
/// discipline): files, units, dynamic units, unlowered units and the
/// pool of every cell.
pub fn slice_summary(files: &[Value]) -> Value {
    let (mut units, mut dynamic, mut unlowered) = (0u64, 0u64, 0u64);
    let mut pools: BTreeMap<String, u64> = cells().into_iter().map(|c| (c, 0)).collect();
    for f in files {
        unlowered += f["unlowered"].as_array().expect("unlowered").len() as u64;
        for u in f["units"].as_array().expect("units") {
            units += 1;
            dynamic += u["dynamic"].as_bool().expect("dynamic") as u64;
            for (c, n) in u["pools"].as_object().expect("pools") {
                *pools.entry(c.clone()).or_insert(0) += n.as_u64().expect("n");
            }
        }
    }
    json!({
        "files": files.len(), "units": units, "dynamic": dynamic,
        "unlowered": unlowered, "pools": pools,
    })
}

/// Every key of `want` present in `doc` with the same value — the
/// envelope check the slice gate and the sample verifier share.
pub fn assert_keys(what: &str, doc: &Value, want: &Value) {
    for (key, value) in want.as_object().expect("object") {
        assert_eq!(&doc[key], value, "{what}: {key} drifted");
    }
}

/// One sample `sources` row: which frozen universe a pool came from.
pub fn source_row(name: &str, slice: &Value) -> Value {
    json!({
        "corpus": name, "tip": slice["corpus"]["tip"], "units": slice["summary"]["units"],
    })
}
