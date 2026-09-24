//! Graph-instrument shared surface: the frozen corpus set, scope
//! constants and the audit/precision gate primitives — ONE binding
//! for the universe gate (eval_graph.rs) and the precision
//! instrument (eval_graph_precision.rs), so the two can never
//! drift apart. The scope classifier and corpus selection retired
//! with the one-shot generators (git history).

use codeeraser::graph::sites::RawSite;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// One site-universe row: content identity plus the per-kind site
/// counts the detector reads off the text — the row throat the self
/// drift walk re-derives and CE_BLESS=1 re-signs with (moved from
/// eval_graph.rs when the v2.30 language exams became its second
/// consumer).
pub fn site_row(path: &str, lang: &str, text: &str) -> Value {
    let sites = codeeraser::graph::sites::detect(text, super::lang_of(lang));
    json!({"lang": lang, "path": path, "sha256": super::content_sha(text),
           "sites": super::kind_counts(&sites)})
}

/// Every site the detector reads off `text`, each asserted within its
/// statement window. The site line is the STATEMENT HEAD: a multi-line
/// TS import carries its full specifier on a later line of the same
/// statement (2c/2d review F1 — 14 frozen zod sites), so the
/// anti-invention property is "spec within the statement window", not
/// "spec on the head line". ONE reading for the self drift walk
/// (eval_graph.rs) and the exam fixtures (eval_lang.rs), which carried
/// the loop twice until the clone gate paired them.
pub fn sites_within_windows(path: &str, lang: &str, text: &str) -> Vec<RawSite> {
    let sites = codeeraser::graph::sites::detect(text, super::lang_of(lang));
    let lines: Vec<&str> = text.lines().collect();
    for s in &sites {
        let end = (s.line + 15).min(lines.len());
        assert!(
            lines[s.line - 1..end].iter().any(|l| l.contains(&s.spec)),
            "{path}:{}: spec {:?} not within its statement window",
            s.line,
            s.spec
        );
    }
    sites
}

/// A site universe's summary, re-derivable from its rows alone — the
/// generator and every gate run this exact function (the G1
/// discipline: generator and gate share one scorer).
pub fn site_summary(files: &[Value]) -> Value {
    let mut by: BTreeMap<String, u64> = BTreeMap::new();
    let mut total = 0;
    for f in files {
        for (kind, n) in f["sites"].as_object().expect("sites") {
            let n = n.as_u64().expect("count");
            let key = format!("{}/{kind}", f["lang"].as_str().expect("lang"));
            *by.entry(key).or_insert(0) += n;
            total += n;
        }
    }
    json!({"files": files.len(), "total_sites": total, "sites_by": by})
}

/// Two string fields of every row under `key`, as a map — the shape
/// every graph doc keys its rows by (rank→truth, path→sha256).
pub fn str_pairs<'a>(doc: &'a Value, key: &str, k1: &str, k2: &str) -> BTreeMap<&'a str, &'a str> {
    doc[key]
        .as_array()
        .expect(key)
        .iter()
        .map(|r| (r[k1].as_str().expect(k1), r[k2].as_str().expect(k2)))
        .collect()
}

/// The frozen-corpus-set anchor (G10): the docs of one family must
/// cover exactly FROZEN_CORPORA — a deleted or renamed doc reddens
/// CI instead of blinding it. Shared by the slice and precision
/// gates.
pub fn assert_frozen_corpus_set(family: &str) -> Vec<String> {
    let expected: Vec<Option<String>> = FROZEN_CORPORA
        .iter()
        .map(|n| n.map(str::to_string))
        .collect();
    assert_corpus_set(family, &expected)
}

/// The same anchor over an explicit sorted corpus list (None = the
/// self doc) — the v2.30 language exams name their own corpora.
pub fn assert_corpus_set(family: &str, expected: &[Option<String>]) -> Vec<String> {
    let docs = super::frozen_docs(family);
    let mut names: Vec<Option<String>> =
        docs.iter().map(|p| super::doc_suffix(p, family)).collect();
    names.sort();
    assert_eq!(names, expected, "frozen {family} corpus set drifted (G10)");
    docs
}

/// Verbatim identity echo of one derived row against its sampled row
/// — the G4 discipline both the audit and precision gates enforce.
pub fn assert_identity_echo(corpus: &str, rank: &str, row: &Value, sampled: &Value) {
    for key in ["path", "line", "nth", "kind", "spec"] {
        assert_eq!(
            row[key], sampled[key],
            "{corpus}/{rank}: {key} echo drifted"
        );
    }
}

/// The rank-bijection lookup (G4/G7): a duplicate row and a phantom
/// row are equally loud — shared by the audit and precision gates.
pub fn bijective_row<'a>(
    corpus: &str,
    row: &'a Value,
    sampled: &BTreeMap<&'a str, &'a Value>,
    seen: &mut BTreeSet<&'a str>,
) -> (&'a str, &'a Value) {
    let rank = row["rank"].as_str().expect("rank");
    assert!(seen.insert(rank), "{corpus}: duplicate row {rank} (G7)");
    let s = sampled
        .get(rank)
        .unwrap_or_else(|| panic!("{corpus}: phantom row {rank} (G4)"));
    (rank, s)
}

/// Frozen graph-universe scope: canonical extensions only (variant
/// suffixes stay out on every corpus: one frozen scope keeps
/// corpora comparable), minus machine-local
/// memory/. The crosscheck fixture islands are deliberately IN scope
/// even though ce.toml excludes them from the product walk: their
/// imports have no in-corpus target, so they are the designed-in
/// negative control (design §5, judge defect D2).
pub const SCOPE_EXTS: [&str; 5] = ["go", "md", "py", "rs", "ts"];
pub const SCOPE_EXCLUDES: [&str; 2] = ["memory/", "cli/memory/"];

/// The frozen corpus set — a deleted or renamed doc reddens CI
/// instead of blinding it (design G10; Opus review: the first gate
/// held no per-corpus anchor at all). Sorted; None = the self doc.
pub const FROZEN_CORPORA: [Option<&str>; 5] = [
    None,
    Some("cobra"),
    Some("requests"),
    Some("ripgrep"),
    Some("zod"),
];

/// Non-path truth verdicts (design §5 vocabulary). Everything else
/// must be a repo-relative "path" or "path#unit".
pub const TRUTH_KEYWORDS: [&str; 4] = ["external", "dynamic", "ambiguous", "none"];

/// The graph family's mount into the ONE review registry
/// (auditgen::REVIEWS — the M5-1d C3 lesson: a gate that resolves
/// via the active corpus reads the wrong book and stays green).
pub fn review_doc(corpus: &str) -> serde_json::Value {
    super::review_of("graph", corpus)
}

/// The rows of one corpus, in frozen order — the ONE sample filter
/// every audit/precision family selects rows with (three families
/// re-grew it independently before the ratchet paired them).
pub fn of_corpus<'a>(rows: &'a [Value], corpus: &str) -> Vec<&'a Value> {
    rows.iter()
        .filter(|r| r["corpus"].as_str() == Some(corpus))
        .collect()
}

/// Every promised seat must hold a nonzero count — a vocabulary
/// class or coverage lane that silently empties is how a gate goes
/// blind while staying green. Shared by the slice coverage gate and
/// the audit vocabulary-seat gate.
pub fn assert_nonzero_seats(counts: &BTreeMap<String, u64>, seats: &[&str], what: &str) {
    for seat in seats {
        assert!(
            counts.get(*seat).copied().unwrap_or(0) > 0,
            "{seat}: {what}"
        );
    }
}

/// Does `check` panic on this doc? The G9 counterfactual primitive
/// every tamper battery runs on.
pub fn doc_refused(doc: &Value, check: &dyn Fn(&Value)) -> bool {
    let doc = doc.clone();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || check(&doc))).is_err()
}

/// The table-driven half of every audit tamper battery: pristine
/// passes, each single-field mutation of row 0 refuses, a dropped
/// row refuses. Families add their bespoke cases via doc_refused.
pub fn assert_tampering_refused(
    pristine: &Value,
    mutations: &[(&str, &str, &str)],
    check: &dyn Fn(&Value),
) {
    assert!(!doc_refused(pristine, check), "pristine table must pass");
    for (field, value, label) in mutations {
        let mut doc = pristine.clone();
        doc["rows"][0][*field] = Value::from(*value);
        assert!(doc_refused(&doc, check), "{label} must refuse");
    }
    let mut missing = pristine.clone();
    missing["rows"].as_array_mut().expect("rows").remove(0);
    assert!(doc_refused(&missing, check), "missing row must refuse");
}

/// The per-corpus audited↔sampled bijection walk (G3/G4/G7): count
/// equality, then every audited row resolved against its sampled row
/// and deduped; row CONTENT stays with the calling family.
pub fn each_audited_row<'a>(
    corpus: &str,
    audited: &'a [Value],
    sample_rows: &[&'a Value],
    mut f: impl FnMut(&'a str, &'a Value, &'a Value),
) {
    let sampled: BTreeMap<&str, &Value> = sample_rows
        .iter()
        .map(|r| (r["rank"].as_str().expect("rank"), *r))
        .collect();
    assert_eq!(audited.len(), sampled.len(), "{corpus}: audited row count");
    let mut seen = BTreeSet::new();
    for row in audited {
        let (rank, s) = bijective_row(corpus, row, &sampled, &mut seen);
        f(rank, row, s);
    }
}
