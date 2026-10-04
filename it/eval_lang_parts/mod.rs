//! The v2.30 language exams (booklet language-expansion.md §11, §14
//! item 14; registry docs/EVAL-SET-LANGS.md): the exam table, its doc
//! families and the pre-registered constants, with the per-language
//! hash-ranked draw in draw.rs — ONE binding for the `#[ignore]`
//! generators (generate.rs) and the CI gates (eval_lang.rs, through
//! the verifier in verify.rs), the G1
//! discipline of the M5-2 sample this re-instantiates per language.
//! No RNG, no clock: ranks are sha256 over domain-separated payloads
//! (eval_support::identity_hash, the M5-2 payload order), seats are
//! integer largest remainder (eval_support::largest_remainder).

pub mod answer;
pub mod draw;
pub mod exams;
pub mod freezing;
pub mod generate;
pub mod precision;
pub mod replay;
pub mod review;
pub mod score;
pub mod tamper;
pub mod tree;
pub mod verify;
pub mod walk;

use crate::eval_support::{UniverseFamily, eval_doc_path, eval_doc_v, load, site_summary};
use serde_json::{Value, json};

pub use exams::EXAMS;

/// One language's exam: its corpora at their pinned tips, the file
/// extensions its universe walks, what its truths may name (Reach),
/// the pathspecs of the rungs that may land only after its audit
/// (lang_provenance.rs), how far it has come (Stage — flipped by the
/// commit that files each doc family, so a doc that vanishes is named,
/// not read as pending), and the generation every doc of the exam
/// carries. A re-freeze (the detector reading more of the language, a
/// new tip) counts it up and retires the old generation by name in
/// docs/EVAL-SET-LANGS.md: the ordering gate reads a doc's first
/// commit (intro_commit), which a doc rewritten in place would keep.
pub struct Exam {
    pub lang: &'static str,
    pub corpora: &'static [(&'static str, &'static str)],
    pub exts: &'static [&'static str],
    pub reach: Reach,
    pub ladder: &'static [&'static str],
    /// Some(why) for an exam whose ladder landed before its sample — the
    /// C family, whose rungs step 2 committed before any exam existed:
    /// the ordering gate then checks the reversal as a fact and holds
    /// the ladder still inside the sample→audit blind window, instead
    /// of reading the ladder's first commit as a breach.
    pub ladder_first: Option<&'static str>,
    pub stage: Stage,
    pub generation: u32,
}

/// How far an exam has come, one step per doc family filed: sampled
/// (the universes and the sample), audited (the blind tables), scored
/// (the precision docs). Ordered, so scored implies audited by
/// construction — the two flags this replaces could disagree.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Stage {
    Sampled,
    Audited,
    Scored,
}

/// What an exam's truths may name: a file of its frozen universe or a
/// package directory holding some (Java, Lua, R load their own code),
/// or any path of the pinned tree, frozen per corpus (tree.rs) — a
/// page fetches whatever the site serves (HTML).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reach {
    Universe,
    Tree,
}

impl Exam {
    /// Whether one per-corpus doc family is filed, by the exam's stage —
    /// and the disk must agree corpus by corpus: a doc that vanished, or
    /// one filed ahead of its stage, is named here, never read as
    /// "pending" (the audit tables' and the precision docs' one reading).
    pub fn filed(&self, docs: &Docs, at: Stage) -> bool {
        let flag = self.stage >= at;
        for (corpus, _) in self.corpora {
            let path = docs.path(self, corpus);
            let present = crate::common::repo_root().join(&path).exists();
            assert_eq!(
                present, flag,
                "{path}: present = {present}, but the exam's flag is {flag}"
            );
        }
        flag
    }

    /// A per-corpus doc's envelope: its schema, its corpus named with the
    /// exam's pinned tip and language (the precision docs, the trees).
    pub fn assert_envelope(&self, corpus: &str, doc: &Value, schema: &str) {
        let tip = self
            .tip(corpus)
            .unwrap_or_else(|| panic!("{corpus}: no corpus of the exam"));
        assert_eq!(doc["schema"], json!(schema), "{corpus}: schema");
        let identity = json!({"name": corpus, "tip": tip, "lang": self.lang});
        assert_eq!(doc["corpus"], identity, "{corpus}: not the exam's corpus");
    }

    /// The exam's frozen sample, its one per-language doc.
    pub fn sample(&self) -> Value {
        SAMPLES.load(self, self.lang)
    }
}

/// A doc family of the exams, `<family>-<key>-v<generation>.json`: the
/// key a corpus (the universes, the audit tables, the precision docs)
/// or the language (the sample), the generation the exam's. Its
/// repo-relative path is what the ordering gate's git facts read, its
/// file what a gate opens and a generator freezes — one spelling
/// (eval_doc_v) for both.
pub struct Docs(pub &'static str);

/// What a doc family needs of an exam: its generation and its pinned
/// corpora. The flow exams (eval_flow_parts) file their docs through
/// the same family type and read their tips through the same lookup.
pub trait Generated {
    fn generation(&self) -> u32;
    fn corpora(&self) -> &[(&'static str, &'static str)];

    /// A corpus's pinned tip, or None when the exam holds no such
    /// corpus — the one lookup the table, the sample and the audit
    /// verifiers read.
    fn tip(&self, corpus: &str) -> Option<&'static str> {
        self.corpora()
            .iter()
            .find(|(c, _)| *c == corpus)
            .map(|(_, t)| *t)
    }
}

impl Generated for Exam {
    fn generation(&self) -> u32 {
        self.generation
    }

    fn corpora(&self) -> &[(&'static str, &'static str)] {
        self.corpora
    }
}

impl Docs {
    pub fn path(&self, exam: &impl Generated, key: &str) -> String {
        eval_doc_path(&self.stem(key), exam.generation())
    }

    pub fn file(&self, exam: &impl Generated, key: &str) -> String {
        eval_doc_v(&self.stem(key), exam.generation())
    }

    pub fn load(&self, exam: &impl Generated, key: &str) -> Value {
        load(&self.file(exam, key))
    }

    fn stem(&self, key: &str) -> String {
        format!("{}-{key}", self.0)
    }
}

/// The frozen universes, the samples, the blind audit tables
/// (review.rs) and the precision docs (score.rs).
pub const SLICES: Docs = Docs(SLICE.family);
pub const SAMPLES: Docs = Docs("lang-sample");
pub const AUDIT_TABLES: Docs = Docs("lang-review");
pub const PRECISION_DOCS: Docs = Docs("lang-precision");

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
        "corpus": name, "tip": slice["corpus"]["tip"],
        "total_sites": slice["summary"]["total_sites"],
    })
}
