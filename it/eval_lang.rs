//! The v2.30 language exams' CI gates (booklet language-expansion.md
//! §11, §14 item 14; registry docs/EVAL-SET-LANGS.md): per corpus a
//! frozen site universe (`lang-slice-<corpus>-v1.json`), per language a
//! hash-ranked sample (`lang-sample-<lang>-v1.json`), both committed
//! before the language's ladder (the ordering: lang_provenance.rs). The
//! five launch corpora keep their own frozen instrument (eval_graph.rs,
//! its scope frozen to the launch extensions); each v2.30 language adds
//! one row to eval_lang_parts::EXAMS. No git, no corpus clone.

use crate::eval_lang_parts::review;
use crate::eval_lang_parts::review::{PACKAGE_KINDS, verify_review};
use crate::eval_lang_parts::tamper::{assert_exam_tampering, tamper_battery};
use crate::eval_lang_parts::verify::verify_sample;
use crate::eval_lang_parts::{self as parts, AUDIT_TABLES, EXAMS, Exam};
use crate::eval_support::{
    TRUTH_KEYWORDS, assert_corpus_set, assert_envelope_core, doc_refused, eval_doc, lang_of, load,
    site_row, sites_within_windows,
};
use codeeraser::scan::lang::Lang;
use serde_json::{Value, json};

fn named(names: Vec<String>) -> Vec<Option<String>> {
    names.into_iter().map(Some).collect()
}

/// Every frozen universe: the frozen set is exactly the exam corpora
/// (G10), each doc passes the shared envelope core under its own
/// language's scope, and every exam language holds sites.
#[test]
fn lang_slices_consistent() {
    for path in assert_corpus_set(parts::SLICE.family, &named(parts::corpus_names())) {
        let doc = load(&path);
        let name =
            assert_envelope_core(&path, &doc, &parts::SLICE).expect("an exam corpus is named");
        let (exam, tip) = parts::exam_of_corpus(&name);
        assert_eq!(doc["schema"], json!(parts::SLICE_SCHEMA), "{path}: schema");
        assert_eq!(doc["scope"], parts::scope(exam), "{path}: scope");
        assert_eq!(
            doc["corpus"]["tip"],
            json!(tip),
            "{path}: not the pinned tip"
        );
        assert_eq!(doc["corpus"]["lang"], json!(exam.lang), "{path}: language");
        for row in doc["files"].as_array().expect("files") {
            assert_eq!(
                row["lang"],
                json!(exam.lang),
                "{path}: another language's file"
            );
        }
        assert!(
            doc["summary"]["total_sites"].as_u64() > Some(0),
            "{path}: no sites"
        );
    }
}

/// An exam's extensions are its language's, by the product's own path
/// table — the scope cannot drift from what `ce` walks as that language.
#[test]
fn exam_scopes_are_the_languages_extensions() {
    for exam in &EXAMS {
        for ext in exam.exts {
            let lang = Lang::from_path(std::path::Path::new(&format!("x.{ext}")));
            assert_eq!(lang, Some(lang_of(exam.lang)), "{}: .{ext}", exam.lang);
        }
    }
}

fn sample_of(exam: &Exam) -> Value {
    load(&eval_doc(&format!("lang-sample-{}", exam.lang)))
}

/// Every exam language's frozen sample verifies (the frozen set is
/// exactly the exam languages, G10).
#[test]
fn lang_samples_verify() {
    let mut langs: Vec<String> = EXAMS.iter().map(|e| e.lang.to_string()).collect();
    langs.sort();
    assert_corpus_set("lang-sample", &named(langs));
    for exam in &EXAMS {
        verify_sample(exam, &sample_of(exam));
    }
}

/// G9: the verifier refuses a forged row, a dropped row, a reordered
/// pair (the shared frame, tamper.rs) and a backup smuggled in as a
/// primary.
#[test]
fn a_tampered_sample_is_refused() {
    let exam = &EXAMS[0];
    let pristine = sample_of(exam);
    let check = |doc: &Value| verify_sample(exam, doc);
    assert_exam_tampering(
        &pristine,
        &[
            ("kind", "forged_kind", "a forged kind"),
            ("rank", "0000", "a forged rank"),
            ("audit", "0000", "a forged audit hash"),
        ],
        &check,
    );
    let mut smuggled = pristine.clone();
    let backup = smuggled["backups"][0].clone();
    smuggled["rows"].as_array_mut().expect("rows").push(backup);
    assert!(
        doc_refused(&smuggled, &check),
        "a smuggled backup must refuse"
    );
}

/// Every frozen audit verifies against its exam's sample: one judged
/// row per primary, truths bound to the frozen universe (review.rs).
/// An exam whose audit is pending has no tables, by its flag.
#[test]
fn lang_reviews_verify() {
    for exam in EXAMS.iter().filter(|e| review::audited(e)) {
        let sample = sample_of(exam);
        for (corpus, _) in exam.corpora {
            verify_review(exam, corpus, &AUDIT_TABLES.load(corpus), &sample);
        }
    }
}

/// G9 for the audit tables: a truth off the frozen universe or out
/// of the vocabulary, a drifted echo, a why under the floor, a
/// dropped row, a swapped pair (the shared frame, tamper.rs) and a
/// cooked summary — each refuses.
#[test]
fn a_tampered_review_is_refused() {
    let (pristine, refused) = tamper_battery(
        &AUDIT_TABLES,
        verify_review,
        &[
            ("truth", "forged/Path.java", "a truth off the universe"),
            ("truth", "elsewhere", "a truth out of the vocabulary"),
            ("why", "a label, not a reason", "a why under the floor"),
        ],
    );
    let mut cooked = pristine.clone();
    cooked["summary"]["external"] = json!(0);
    assert!(refused(&cooked), "a cooked summary must refuse");
}

/// A package truth answers a package-level site only, its code where
/// the language keeps it (review.rs PACKAGE_KINDS): moved onto a row
/// of another kind it refuses, and so does the directory that holds
/// a package's code standing in for the package (R's `R/`). Every
/// audited table that holds a package truth aims one.
#[test]
fn a_misplaced_package_truth_is_refused() {
    let mut aimed = Vec::new();
    for exam in EXAMS.iter().filter(|e| review::audited(e)) {
        let sample = sample_of(exam);
        for (corpus, _) in exam.corpora {
            let pristine = AUDIT_TABLES.load(corpus);
            let rows = pristine["rows"].as_array().expect("rows");
            let Some(at) = rows.iter().position(|r| package_truth(r, exam)) else {
                continue;
            };
            let check = |doc: &Value| verify_review(exam, corpus, doc, &sample);
            assert!(
                !doc_refused(&pristine, &check),
                "{corpus}: pristine table must pass"
            );
            for forged in misplaced(&pristine, at) {
                assert!(
                    doc_refused(&forged, &check),
                    "{corpus}: a misplaced package truth must refuse"
                );
            }
            aimed.push(*corpus);
        }
    }
    assert_eq!(
        aimed,
        ["jsoup", "covid19model"],
        "the tables holding a package truth"
    );
}

/// Whether a row's truth is a package directory: no keyword, and no
/// file of the exam's extensions (`#Member` aside) — every frozen file
/// carries one.
fn package_truth(row: &Value, exam: &Exam) -> bool {
    let truth = row["truth"].as_str().expect("truth");
    let path = truth.split('#').next().unwrap_or(truth);
    !TRUTH_KEYWORDS.contains(&truth) && !exam.exts.iter().any(|e| path.ends_with(&format!(".{e}")))
}

/// The forgeries of row `at`'s package truth: the truth on the first
/// row of another kind, and — where the kind keeps its code in a
/// subdirectory — that code directory standing in for the package.
fn misplaced(pristine: &Value, at: usize) -> Vec<Value> {
    let row = &pristine["rows"][at];
    let (truth, kind) = (row["truth"].as_str().expect("truth"), &row["kind"]);
    let mut moved = pristine.clone();
    let other = moved["rows"]
        .as_array_mut()
        .expect("rows")
        .iter_mut()
        .find(|r| &r["kind"] != kind)
        .expect("a row of another kind");
    other["truth"] = json!(truth);
    let mut forged = vec![moved];
    let code = PACKAGE_KINDS
        .iter()
        .find(|(k, _)| json!(k) == *kind)
        .map_or("", |(_, c)| c.trim_end_matches('/'));
    if !code.is_empty() {
        let mut deeper = pristine.clone();
        deeper["rows"][at]["truth"] = json!(format!("{truth}/{code}"));
        forged.push(deeper);
    }
    forged
}

/// Gaps sit on the side of the frozen universe their list names
/// (review.rs check_gaps): a scope gap filed as a site gap refuses,
/// and so does a site gap filed as a scope gap. luarocks' table holds
/// both lists.
#[test]
fn a_gap_on_the_wrong_side_is_refused() {
    let exam = parts::exam("lua");
    let sample = sample_of(exam);
    let pristine = AUDIT_TABLES.load("luarocks");
    let check = |doc: &Value| verify_review(exam, "luarocks", doc, &sample);
    assert!(
        !doc_refused(&pristine, &check),
        "the pristine table verifies"
    );
    for (from, to) in [("scope_gaps", "site_gaps"), ("site_gaps", "scope_gaps")] {
        let mut moved = pristine.clone();
        let gap = moved[from].as_array_mut().expect(from).remove(0);
        moved[to].as_array_mut().expect(to).push(gap);
        assert!(
            doc_refused(&moved, &check),
            "a {from} entry filed under {to} must refuse"
        );
    }
}

/// The detector and the frozen universes must not drift apart
/// silently, and without the corpus clone: each language's committed
/// crosscheck fixtures are verbatim files of its first corpus at the
/// same tip (contracts/fixtures/crosscheck/SOURCES.md, `__` = `/`), so
/// each is re-detected against its frozen row — and every detected
/// spec must sit inside its statement window (the M5-2 anti-invention
/// property).
#[test]
fn crosscheck_fixtures_track_the_frozen_universe() {
    for exam in &EXAMS {
        let (name, _) = exam.corpora[0];
        let slice = load(&eval_doc(&format!("lang-slice-{name}")));
        let dir = format!("../contracts/fixtures/crosscheck/{}", exam.lang);
        let mut verified = 0;
        for entry in std::fs::read_dir(&dir).expect(&dir) {
            let entry = entry.expect("entry");
            let path = entry.file_name().to_string_lossy().replace("__", "/");
            let text = std::fs::read_to_string(entry.path()).expect("fixture");
            let frozen = slice["files"]
                .as_array()
                .expect("files")
                .iter()
                .find(|f| f["path"] == json!(path))
                .unwrap_or_else(|| panic!("{path}: fixture outside the frozen universe"));
            assert_eq!(
                &site_row(&path, exam.lang, &text),
                frozen,
                "{path}: drifted"
            );
            sites_within_windows(&path, exam.lang, &text);
            verified += 1;
        }
        assert!(verified >= 5, "{}: {verified} fixtures verified", exam.lang);
    }
}
