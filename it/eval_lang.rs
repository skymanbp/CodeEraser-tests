//! The v2.30 language exams' CI gates (booklet language-expansion.md
//! §11, §14 item 14; registry docs/EVAL-SET-LANGS.md): per corpus a
//! frozen site universe (`lang-slice-<corpus>-v1.json`), per language a
//! hash-ranked sample (`lang-sample-<lang>-v1.json`), both committed
//! before the language's ladder (the ordering: lang_provenance.rs). The
//! five launch corpora keep their own frozen instrument (eval_graph.rs,
//! its scope frozen to the launch extensions); each v2.30 language adds
//! one row to eval_lang_parts::EXAMS. No git, no corpus clone.

use crate::eval_lang_parts::review;
use crate::eval_lang_parts::review::{load_review, verify_review};
use crate::eval_lang_parts::verify::verify_sample;
use crate::eval_lang_parts::{self as parts, EXAMS, Exam};
use crate::eval_support::{
    assert_corpus_set, assert_envelope_core, assert_tampering_refused, doc_refused, eval_doc,
    lang_of, load, site_row, sites_within_windows,
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
/// pair and a backup smuggled in as a primary.
#[test]
fn a_tampered_sample_is_refused() {
    let exam = &EXAMS[0];
    let pristine = sample_of(exam);
    let check = |doc: &Value| verify_sample(exam, doc);
    assert_tampering_refused(
        &pristine,
        &[
            ("path", "forged/Path.java", "a forged path"),
            ("spec", "forged.Spec", "a forged spec"),
            ("kind", "forged_kind", "a forged kind"),
            ("rank", "0000", "a forged rank"),
            ("audit", "0000", "a forged audit hash"),
        ],
        &check,
    );
    let mut swapped = pristine.clone();
    swapped["rows"].as_array_mut().expect("rows").swap(0, 1);
    assert!(
        doc_refused(&swapped, &check),
        "a reordered pair must refuse"
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
            verify_review(exam, corpus, &load_review(corpus), &sample);
        }
    }
}

/// G9 for the audit tables: a truth off the frozen universe or out
/// of the vocabulary, a drifted echo, a why under the floor, a
/// dropped row, a swapped pair, a package truth on a row that is no
/// on-demand import, and a cooked summary — each refuses. jsoup's
/// table is the one that holds a package truth to aim with.
#[test]
fn a_tampered_review_is_refused() {
    let exam = &EXAMS[0];
    let (corpus, _) = exam.corpora[1];
    let (sample, pristine) = (sample_of(exam), load_review(corpus));
    let check = |doc: &Value| verify_review(exam, corpus, doc, &sample);
    assert_tampering_refused(
        &pristine,
        &[
            ("truth", "forged/Path.java", "a truth off the universe"),
            ("truth", "elsewhere", "a truth out of the vocabulary"),
            ("path", "forged/Path.java", "a drifted path"),
            ("spec", "forged.Spec", "a drifted spec"),
            ("why", "a label, not a reason", "a why under the floor"),
        ],
        &check,
    );
    let package = pristine["rows"]
        .as_array()
        .expect("rows")
        .iter()
        // the one truth class spelled with a `/` and no `.java`
        .find(|r| {
            r["truth"]
                .as_str()
                .is_some_and(|t| t.contains('/') && !t.contains(".java"))
        })
        .expect("a package truth")["truth"]
        .clone();
    let mut misplaced = pristine.clone();
    let row = misplaced["rows"]
        .as_array_mut()
        .expect("rows")
        .iter_mut()
        .find(|r| r["kind"] != json!("import_star"))
        .expect("a row that is no on-demand import");
    row["truth"] = package;
    let mut swapped = pristine.clone();
    swapped["rows"].as_array_mut().expect("rows").swap(0, 1);
    let mut cooked = pristine.clone();
    cooked["summary"]["external"] = json!(0);
    for (label, doc) in [
        ("a misplaced package truth", &misplaced),
        ("a swapped pair", &swapped),
        ("a cooked summary", &cooked),
    ] {
        assert!(doc_refused(doc, &check), "{label} must refuse");
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
