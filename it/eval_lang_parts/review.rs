//! The frozen audit-table verifier of the v2.30 language exams: one
//! corpus's blind ground truth (`lang-review-<corpus>-v<g>.json`,
//! assembled verbatim from independent auditors who read the pinned
//! clone and their batch, never a parse) against the frozen sample it
//! answers and the frozen files its truths must name — the universe,
//! or the pinned tree where the exam's references reach any file
//! (Reach, tree.rs). Split from
//! the gates like verify.rs: the tamper gate runs it on forged copies.

use super::tree::{targets, universe};
use crate::eval_lang_parts::{AUDIT_TABLES, Exam, Reach, SLICES, Stage};
use crate::eval_support::{MIN_WHY, TRUTH_KEYWORDS};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const REVIEW_SCHEMA: &str = "ce.eval-lang-review/1.0.0";

/// The sampled identity a judged row echoes, field for field — the
/// audit tables' rows and the precision docs' alike.
pub const ECHO: [&str; 6] = ["rank", "path", "line", "nth", "kind", "spec"];

/// Whether an exam's audit is frozen: its EXAMS row has reached the
/// stage, and the tables on disk agree (Exam::filed).
pub fn audited(exam: &Exam) -> bool {
    exam.filed(&AUDIT_TABLES, Stage::Audited)
}

/// The site kinds a package directory answers, each with where the
/// package keeps its frozen code under that directory: Java's
/// on-demand import names the directory holding the package's files,
/// R's package load the package root, whose code sits in `R/`.
pub const PACKAGE_KINDS: [(&str, &str); 2] = [("import_star", ""), ("library", "R/")];

/// What a truth names (the M5-2 vocabulary): a keyword; a file the
/// exam's truths may name (tree.rs targets), `#Name` naming a member
/// declared in it (a document language's section: the element id, the
/// heading's slug); or — for a package-level site only — the package's
/// directory (`.` is the corpus root), its code frozen where
/// PACKAGE_KINDS says.
pub fn class_of(truth: &str, kind: &str, files: &BTreeSet<String>) -> &'static str {
    if let Some(k) = TRUTH_KEYWORDS.iter().find(|k| **k == truth) {
        return k;
    }
    let (path, member) = match truth.split_once('#') {
        Some((p, m)) => (p, Some(m)),
        None => (truth, None),
    };
    if files.contains(path) {
        return match member {
            None => "file",
            Some(m) if !m.is_empty() && !m.contains(' ') => "member",
            Some(_) => panic!("{truth}: no member name after `#`"),
        };
    }
    let holds = PACKAGE_KINDS
        .iter()
        .find(|(k, _)| *k == kind)
        .is_some_and(|(_, code)| {
            let dir = match truth {
                "." => code.to_string(),
                _ => format!("{truth}/{code}"),
            };
            files
                .iter()
                .any(|f| f.strip_prefix(&dir).is_some_and(|rest| !rest.contains('/')))
        });
    assert!(
        holds,
        "{truth}: no keyword, no frozen file, and no package directory holding frozen code for this row's kind ({kind})"
    );
    "package"
}

/// One corpus's table against its exam: the envelope (schema, corpus,
/// language, pinned tip), one judged row per sampled primary of the
/// corpus in sample order echoing its identity, each truth in the
/// vocabulary and bound to the files the exam's truths may name, each
/// why past the floor with its batch named, the summary re-derived
/// from the truths, and every gap on its side of the frozen universe.
pub fn verify_review(exam: &Exam, corpus: &str, doc: &Value, sample: &Value) {
    assert_eq!(doc["schema"], json!(REVIEW_SCHEMA), "{corpus}: schema");
    assert_eq!(doc["corpus"], json!(corpus), "{corpus}: corpus");
    assert_eq!(doc["lang"], json!(exam.lang), "{corpus}: language");
    let tip = exam
        .tip(corpus)
        .unwrap_or_else(|| panic!("{corpus}: no corpus of the exam"));
    assert_eq!(doc["tip"], json!(tip), "{corpus}: not the pinned tip");
    let files = universe(&SLICES.load(exam, corpus));
    let reach = targets(exam, corpus, &files);
    let sampled: Vec<&Value> = sample["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .filter(|r| r["corpus"] == json!(corpus))
        .collect();
    let rows = doc["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), sampled.len(), "{corpus}: judged row count");
    let mut summary: BTreeMap<&str, u64> = BTreeMap::new();
    for (row, s) in rows.iter().zip(sampled) {
        let rank = s["rank"].as_str().expect("rank");
        for f in ECHO {
            assert_eq!(row[f], s[f], "{corpus}/{rank}: {f} is not the sampled one");
        }
        let truth = row["truth"].as_str().expect("truth");
        *summary
            .entry(class_of(truth, row["kind"].as_str().expect("kind"), &reach))
            .or_default() += 1;
        assert!(
            row["why"].as_str().expect("why").chars().count() >= MIN_WHY,
            "{corpus}/{rank}: a why under the floor"
        );
        assert!(row["batch"].as_u64() >= Some(1), "{corpus}/{rank}: batch");
    }
    assert_eq!(doc["summary"], json!(summary), "{corpus}: summary drifted");
    check_gaps(exam, corpus, doc, &files, &reach);
}

/// Every gap sits at a real line and carries its note, on the side of
/// the frozen universe its list names: a site gap on a frozen file, a
/// scope gap on a file the universe does not walk (luarocks' launcher
/// `src/bin/luarocks` is Lua without the extension) — and, where the
/// exam's truths reach the pinned tree, on a path of it (learning-
/// area's gallery script builds the `src` its page swaps in). Java's
/// tables were filed before scope gaps existed and hold none.
fn check_gaps(
    exam: &Exam,
    corpus: &str,
    doc: &Value,
    files: &BTreeSet<String>,
    reach: &BTreeSet<String>,
) {
    let none = Vec::new();
    for (field, frozen) in [("site_gaps", true), ("scope_gaps", false)] {
        let gaps = match doc.get(field) {
            Some(g) => g.as_array().unwrap_or_else(|| panic!("{corpus}: {field}")),
            None if !frozen => &none,
            None => panic!("{corpus}: no {field}"),
        };
        for gap in gaps {
            let path = gap["path"].as_str().expect("path");
            let known = frozen || exam.reach == Reach::Universe || reach.contains(path);
            assert!(
                files.contains(path) == frozen && known && gap["line"].as_u64() >= Some(1),
                "{corpus}: a {field} entry on the wrong side of the frozen universe ({path})"
            );
            assert!(
                gap["note"].as_str().is_some_and(|n| !n.is_empty()),
                "{corpus}: a {field} entry without its note"
            );
        }
    }
}
