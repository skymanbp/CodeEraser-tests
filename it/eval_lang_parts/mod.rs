//! The v2.30 language exams (booklet language-expansion.md §11, §14
//! item 14; registry docs/EVAL-SET-LANGS.md): the exam table and the
//! per-language hash-ranked draw — ONE binding for the `#[ignore]`
//! generators (generate.rs) and the CI gates (eval_lang.rs, through
//! the verifier in verify.rs), the G1
//! discipline of the M5-2 sample this re-instantiates per language.
//! No RNG, no clock: ranks are sha256 over domain-separated payloads
//! (eval_support::identity_hash, the M5-2 payload order), seats are
//! integer largest remainder (eval_support::largest_remainder).

pub mod generate;
pub mod review;
pub mod verify;

use crate::eval_support::{UniverseFamily, identity_hash, largest_remainder, site_summary};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// One language's exam: its corpora at their pinned tips, the file
/// extensions its universe walks, the pathspec of the rungs that
/// may land only after its audit (lang_provenance.rs), and whether
/// that audit is frozen — flipped by the commit that files the
/// tables, so a table that vanishes is named, not read as pending.
pub struct Exam {
    pub lang: &'static str,
    pub corpora: &'static [(&'static str, &'static str)],
    pub exts: &'static [&'static str],
    pub ladder: &'static str,
    pub audited: bool,
}

impl Exam {
    /// A corpus's pinned tip, or None when the exam holds no such
    /// corpus — the one lookup the table, the sample and the audit
    /// verifiers read.
    pub fn tip(&self, corpus: &str) -> Option<&'static str> {
        self.corpora
            .iter()
            .find(|(c, _)| *c == corpus)
            .map(|(_, t)| *t)
    }
}

/// Every language whose exam is frozen, in landing order; a language
/// joins with its step (booklet §13) and never leaves. A second corpus
/// joins when the first holds none of a site kind: gson has no wildcard
/// import (its style guide forbids them), jsoup brings them.
pub const EXAMS: [Exam; 1] = [Exam {
    lang: "java",
    corpora: &[
        ("gson", "854c8255b625cf1e13c701a83ea9ccb4caaa576a"),
        ("jsoup", "093e2f58492c531667e551e8793513a41b22443e"),
    ],
    exts: &["java"],
    ladder: "cli/src/graph/ladder/java*",
    audited: true,
}];

pub const SLICE_SCHEMA: &str = "ce.eval-lang-slice/1.0.0";
pub const SAMPLE_SCHEMA: &str = "ce.eval-lang-sample/1.0.0";

/// Pre-registered sample constants: TOTAL primaries per language, a
/// floor of MIN_PER_KIND per site kind before the largest-remainder
/// seats (a kind with fewer sites is taken whole), BACKUP_PER_KIND
/// replacements per kind for an unanswerable primary — replenishment
/// stays inside the kind, or one bad row would sink its floor.
pub const TOTAL: u64 = 100;
pub const MIN_PER_KIND: u64 = 15;
pub const BACKUP_PER_KIND: u64 = 20;
pub const SITE_DOMAIN: &str = "ce-lang-site-v1";
pub const AUDIT_DOMAIN: &str = "ce-lang-audit-v1";

/// The M5-2 rank payload, field for field: spec last, so the
/// '|'-joined encoding is injective.
pub const FIELDS: [&str; 7] = ["corpus", "commit", "path", "line", "nth", "kind", "spec"];

pub fn slice_constants() -> Value {
    json!({"min_per_kind": MIN_PER_KIND, "r0_share_trigger": 0.80})
}

/// The exam slices as one universe family — the frozen constants above
/// and the shared site scorer — for the envelope core (eval_lang.rs).
pub const SLICE: UniverseFamily = UniverseFamily {
    family: "lang-slice",
    constants: slice_constants,
    summarize: site_summary,
};

pub fn sample_constants() -> Value {
    json!({
        "total": TOTAL, "min_per_kind": MIN_PER_KIND, "backup_per_kind": BACKUP_PER_KIND,
        "domains": {"site": SITE_DOMAIN, "audit": AUDIT_DOMAIN},
    })
}

/// The frozen scope of one exam's universe.
pub fn scope(exam: &Exam) -> Value {
    json!({"extensions": exam.exts, "excludes": []})
}

pub fn exam(lang: &str) -> &'static Exam {
    EXAMS
        .iter()
        .find(|e| e.lang == lang)
        .unwrap_or_else(|| panic!("{lang}: no exam"))
}

/// The exam a corpus belongs to, and its pinned tip.
pub fn exam_of_corpus(name: &str) -> (&'static Exam, &'static str) {
    EXAMS
        .iter()
        .find_map(|e| e.tip(name).map(|t| (e, t)))
        .unwrap_or_else(|| panic!("{name}: no exam holds this corpus"))
}

/// Every exam corpus name, sorted — the frozen-set anchor (G10).
pub fn corpus_names() -> Vec<String> {
    let mut names: Vec<String> = EXAMS
        .iter()
        .flat_map(|e| e.corpora.iter().map(|(c, _)| c.to_string()))
        .collect();
    names.sort();
    names
}

/// One sample `sources` row: which frozen universe a pool came from.
pub fn source_row(name: &str, slice: &Value) -> Value {
    json!({
        "corpus": name,
        "tip": slice["corpus"]["tip"],
        "total_sites": slice["summary"]["total_sites"],
    })
}

/// Kind quotas from the pool's per-kind counts: each kind first takes
/// min(MIN_PER_KIND, pool), the seats left go by largest remainder
/// over what each kind has left, so no quota exceeds its pool.
pub fn quotas(pool: &BTreeMap<String, u64>) -> BTreeMap<String, u64> {
    let floor: BTreeMap<String, u64> = pool
        .iter()
        .map(|(k, n)| (k.clone(), (*n).min(MIN_PER_KIND)))
        .collect();
    let rest: BTreeMap<String, u64> = pool
        .iter()
        .map(|(k, n)| (k.clone(), n - floor[k]))
        .collect();
    let seats = TOTAL
        .saturating_sub(floor.values().sum())
        .min(rest.values().sum());
    let extra = largest_remainder(&rest, seats);
    floor
        .into_iter()
        .map(|(k, f)| {
            let e = extra[&k];
            (k, f + e)
        })
        .collect()
}

/// One sample row: the pool row plus both domain hashes.
pub fn ranked(site: &Value) -> Value {
    let mut row = site.clone();
    row["rank"] = json!(identity_hash(SITE_DOMAIN, site, &FIELDS));
    row["audit"] = json!(identity_hash(AUDIT_DOMAIN, site, &FIELDS));
    row
}

/// The frozen draw.
pub struct Draw {
    pub allocation: BTreeMap<String, u64>,
    pub primary: Vec<Value>,
    pub backups: Vec<Value>,
}

fn by_hash<'a>(row: &'a Value, domain: &str) -> &'a str {
    row[domain].as_str().expect("hash")
}

/// Within each kind the site-domain rank picks the quota; the primaries
/// are then stored in audit-domain order (the auditor never sees rank
/// order), and each kind's backups are the audit-domain rank over its
/// unpicked rest, stored kind by kind.
pub fn draw(pool: &[Value]) -> Draw {
    let mut by_kind: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for site in pool {
        let kind = site["kind"].as_str().expect("kind").to_string();
        by_kind.entry(kind).or_default().push(ranked(site));
    }
    let counts = by_kind
        .iter()
        .map(|(k, v)| (k.clone(), v.len() as u64))
        .collect();
    let allocation = quotas(&counts);
    let (mut primary, mut backups) = (Vec::new(), Vec::new());
    for (kind, mut rows) in by_kind {
        rows.sort_by(|a, b| by_hash(a, "rank").cmp(by_hash(b, "rank")));
        let mut rest = rows.split_off(allocation[&kind] as usize);
        rest.sort_by(|a, b| by_hash(a, "audit").cmp(by_hash(b, "audit")));
        primary.extend(rows);
        backups.extend(rest.into_iter().take(BACKUP_PER_KIND as usize));
    }
    primary.sort_by(|a, b| by_hash(a, "audit").cmp(by_hash(b, "audit")));
    Draw {
        allocation,
        primary,
        backups,
    }
}
