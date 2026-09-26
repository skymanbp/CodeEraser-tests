//! The v2.30 language exams' CI gates (booklet language-expansion.md
//! §11, §14 item 14; registry docs/EVAL-SET-LANGS.md): per corpus a
//! frozen site universe (`lang-slice-<corpus>-v<g>.json`), per language
//! a hash-ranked sample (`lang-sample-<lang>-v<g>.json`), g the exam's
//! generation, both committed before the language's ladder (the
//! ordering: lang_provenance.rs) and, where an exam's truths reach any
//! tracked file, per corpus the pinned tree (`lang-tree-<corpus>-v<g>.json`,
//! tree.rs); the audit tables' gates are
//! eval_lang_review.rs. The five launch corpora keep their own frozen
//! instrument (eval_graph.rs, its scope frozen to the launch
//! extensions); each v2.30 language adds one row to
//! eval_lang_parts::EXAMS. No git, no corpus clone.

use crate::eval_lang_parts::tamper::assert_exam_tampering;
use crate::eval_lang_parts::tree::{TREES, verify_tree};
use crate::eval_lang_parts::verify::verify_sample;
use crate::eval_lang_parts::{self as parts, EXAMS, Reach, Stage};
use crate::eval_support::{
    assert_corpus_set, assert_envelope_core, doc_refused, lang_of, load, site_row,
    sites_within_windows,
};
use codeeraser::scan::lang::Lang;
use serde_json::{Value, json};

fn named(names: Vec<String>) -> Vec<Option<String>> {
    names.into_iter().map(Some).collect()
}

/// Every frozen universe: the frozen set is exactly the exam corpora
/// (G10), each at its exam's generation, each doc passes the shared
/// envelope core under its own language's scope, and every exam
/// language holds sites.
#[test]
fn lang_slices_consistent() {
    for path in assert_corpus_set(parts::SLICE.family, &named(parts::corpus_names())) {
        let doc = load(&path);
        let name =
            assert_envelope_core(&path, &doc, &parts::SLICE).expect("an exam corpus is named");
        let (exam, tip) = parts::exam_of_corpus(&name);
        assert_eq!(
            path,
            parts::SLICES.file(exam, &name),
            "{path}: not the exam's generation"
        );
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

/// Every exam language's frozen sample verifies (the frozen set is
/// exactly the exam languages, G10).
#[test]
fn lang_samples_verify() {
    let mut langs: Vec<String> = EXAMS.iter().map(|e| e.lang.to_string()).collect();
    langs.sort();
    assert_corpus_set(parts::SAMPLES.0, &named(langs));
    for exam in &EXAMS {
        verify_sample(exam, &exam.sample());
    }
}

/// G9: the verifier refuses a forged row, a dropped row, a reordered
/// pair (the shared frame, tamper.rs) and a backup smuggled in as a
/// primary.
#[test]
fn a_tampered_sample_is_refused() {
    let exam = &EXAMS[0];
    let pristine = exam.sample();
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

/// The stages are ordered as the doc families are filed — sampled,
/// audited, scored — so Exam::filed reads "reached" and a scored exam
/// is an audited one by construction.
#[test]
fn stages_are_filed_in_order() {
    assert!(Stage::Sampled < Stage::Audited && Stage::Audited < Stage::Scored);
}

/// Every frozen tree verifies against its exam and its frozen universe
/// (tree.rs); the frozen set is exactly the corpora of the exams whose
/// truths reach the tree (G10).
#[test]
fn lang_trees_verify() {
    let mut corpora = Vec::new();
    for exam in EXAMS.iter().filter(|e| e.reach == Reach::Tree) {
        for (corpus, _) in exam.corpora {
            verify_tree(exam, corpus, &TREES.load(exam, corpus));
            corpora.push(corpus.to_string());
        }
    }
    corpora.sort();
    assert_corpus_set(TREES.0, &named(corpora));
}

/// G9 for the trees: a universe file dropped, a page of the exam's own
/// extension smuggled in, a pair out of order, a rewritten method and
/// another tip each refuse; the pristine tree passes.
#[test]
fn a_tampered_tree_is_refused() {
    let exam = parts::exam("html");
    let (corpus, _) = exam.corpora[0];
    let pristine = TREES.load(exam, corpus);
    let check = |doc: &Value| verify_tree(exam, corpus, doc);
    assert!(
        !doc_refused(&pristine, &check),
        "the pristine tree verifies"
    );
    let page = parts::SLICES.load(exam, corpus)["files"][0]["path"].clone();
    fn paths(doc: &mut Value) -> &mut Vec<Value> {
        doc["paths"].as_array_mut().expect("paths")
    }
    type Forgery<'a> = (&'a dyn Fn(&mut Value), &'a str);
    let forgeries: [Forgery; 5] = [
        (
            &|d| paths(d).retain(|p| *p != page),
            "a universe file dropped",
        ),
        (
            &|d| paths(d).push(json!("zz/forged.html")),
            "a page smuggled in",
        ),
        (&|d| paths(d).swap(0, 1), "a pair out of order"),
        (
            &|d| d["method"] = json!("another reading"),
            "a rewritten method",
        ),
        (&|d| d["corpus"]["tip"] = json!("0000"), "another tip"),
    ];
    for (forge, what) in forgeries {
        let mut forged = pristine.clone();
        forge(&mut forged);
        assert!(doc_refused(&forged, &check), "{what} must refuse");
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
        let slice = parts::SLICES.load(exam, name);
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
