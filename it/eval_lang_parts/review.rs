//! The frozen audit-table verifier of the v2.30 language exams: one
//! corpus's blind ground truth (`lang-review-<corpus>-v1.json`,
//! assembled verbatim from independent auditors who read the pinned
//! clone and their batch, never a parse) against the frozen sample it
//! answers and the frozen universe its truths must name. Split from
//! the gates like verify.rs: the tamper gate runs it on forged copies.

use crate::eval_lang_parts::Exam;
use crate::eval_support::{MIN_WHY, TRUTH_KEYWORDS, eval_doc, load};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const REVIEW_SCHEMA: &str = "ce.eval-lang-review/1.0.0";

/// The sampled identity a judged row echoes, field for field.
const ECHO: [&str; 6] = ["rank", "path", "line", "nth", "kind", "spec"];

/// One corpus's audit table, repo-relative — the ordering gate's git
/// facts and the verifier's load read the same file.
pub fn review_path(corpus: &str) -> String {
    format!("contracts/eval/lang-review-{corpus}-v1.json")
}

pub fn load_review(corpus: &str) -> Value {
    load(&eval_doc(&format!("lang-review-{corpus}")))
}

/// Whether an exam's audit is frozen: its EXAMS row says so, and the
/// tables on disk must agree corpus by corpus — a table that vanished
/// or one filed ahead of its flag is named, never read as "pending".
pub fn audited(exam: &Exam) -> bool {
    for (corpus, _) in exam.corpora {
        let present = crate::common::repo_root()
            .join(review_path(corpus))
            .exists();
        assert_eq!(
            present, exam.audited,
            "{corpus}: audit table present = {present}, but the exam's audited flag is {}",
            exam.audited
        );
    }
    exam.audited
}

/// What a truth names (the M5-2 vocabulary): a keyword; a frozen file
/// of the corpus, `#Name` naming a member declared in it; or — for an
/// on-demand import of a package only — the directory directly
/// holding frozen files of that package.
fn class_of(truth: &str, kind: &str, files: &BTreeSet<String>) -> &'static str {
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
    let dir = format!("{truth}/");
    let holds = files
        .iter()
        .any(|f| f.strip_prefix(&dir).is_some_and(|rest| !rest.contains('/')));
    assert!(
        kind == "import_star" && holds,
        "{truth}: no keyword, no frozen file, and a package directory answers an on-demand import only (this row: {kind})"
    );
    "package"
}

/// One corpus's table against its exam: the envelope (schema, corpus,
/// language, pinned tip), one judged row per sampled primary of the
/// corpus in sample order echoing its identity, each truth in the
/// vocabulary and bound to the corpus's frozen universe, each why past
/// the floor with its batch named, the summary re-derived from the
/// truths, and every site gap on a frozen file.
pub fn verify_review(exam: &Exam, corpus: &str, doc: &Value, sample: &Value) {
    assert_eq!(doc["schema"], json!(REVIEW_SCHEMA), "{corpus}: schema");
    assert_eq!(doc["corpus"], json!(corpus), "{corpus}: corpus");
    assert_eq!(doc["lang"], json!(exam.lang), "{corpus}: language");
    let tip = exam
        .tip(corpus)
        .unwrap_or_else(|| panic!("{corpus}: no corpus of the exam"));
    assert_eq!(doc["tip"], json!(tip), "{corpus}: not the pinned tip");
    let slice = load(&eval_doc(&format!("lang-slice-{corpus}")));
    let files: BTreeSet<String> = slice["files"]
        .as_array()
        .expect("files")
        .iter()
        .map(|f| f["path"].as_str().expect("path").to_string())
        .collect();
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
            .entry(class_of(truth, row["kind"].as_str().expect("kind"), &files))
            .or_default() += 1;
        assert!(
            row["why"].as_str().expect("why").chars().count() >= MIN_WHY,
            "{corpus}/{rank}: a why under the floor"
        );
        assert!(row["batch"].as_u64() >= Some(1), "{corpus}/{rank}: batch");
    }
    assert_eq!(doc["summary"], json!(summary), "{corpus}: summary drifted");
    for gap in doc["site_gaps"].as_array().expect("site_gaps") {
        let path = gap["path"].as_str().expect("path");
        assert!(
            files.contains(path) && gap["line"].as_u64() >= Some(1),
            "{corpus}: a site gap off the frozen universe ({path})"
        );
        assert!(
            gap["note"].as_str().is_some_and(|n| !n.is_empty()),
            "{corpus}: a site gap without its note"
        );
    }
}
