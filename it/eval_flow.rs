//! The v2.31 flow exams' CI gates (design booklet analysis-track.md
//! §5.5; registry docs/EVAL-SET-FLOW.md): per `<lang>-<corpus>` a
//! frozen unit universe (`flow-slice-<lang>-<corpus>-v<g>.json`), per
//! language a hash-ranked per-cell sample (`flow-sample-<lang>-v<g>.json`),
//! g the exam's generation, both committed after the lowering they
//! draw from (the `ladder_first` ordering, flow_provenance.rs at commit
//! C) and before the blind audit. Each exam adds one row to
//! eval_flow_parts::EXAMS. No git, no corpus clone.

use crate::eval_flow_parts::{self as parts, EXAMS, FlowExam, Stage, verify::verify_sample};
use crate::eval_lang::named;
use crate::eval_lang_parts::Docs;
use crate::eval_support::{MIN_WHY, UniverseFamily, assert_corpus_set, assert_envelope_core, load};
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// The frozen set of a family is exactly `keys` (G10).
fn frozen_set(family: &Docs, keys: BTreeSet<String>) {
    assert_corpus_set(family.0, &named(keys.into_iter().collect()));
}

/// The slice family of one language, as the shared envelope core reads
/// it: the doc suffix after `flow-slice-<lang>` is the corpus name.
fn family(lang: &str) -> UniverseFamily {
    UniverseFamily {
        family: Box::leak(format!("flow-slice-{lang}").into_boxed_str()),
        constants: parts::slice_constants,
        summarize: parts::slice_summary,
    }
}

/// Every frozen universe with its exam, corpus, tip and path.
fn each_slice(mut f: impl FnMut(&FlowExam, &str, &str, &str, &Value)) {
    for exam in EXAMS.iter() {
        for (corpus, tip) in &exam.corpora {
            let path = parts::SLICES.file(exam, &exam.key(corpus));
            f(exam, corpus, tip, &path, &load(&path));
        }
    }
}

/// Every unit row of a universe, with where it sits.
fn units(path: &str, doc: &Value) -> Vec<(String, Value)> {
    let files = doc["files"].as_array().expect("files");
    let rows = files.iter().flat_map(|f| {
        let units = f["units"].as_array().expect("units");
        units.iter().map(move |u| {
            (
                format!("{path}: {} unit {}", f["path"], u["nth"]),
                u.clone(),
            )
        })
    });
    rows.collect()
}

/// Every unit row: three non-negative table sizes, an ordered line
/// pair, pools keyed by the cells the class table names.
fn units_well_formed(path: &str, doc: &Value) {
    let cells = parts::cells();
    for (at, u) in units(path, doc) {
        let rows = u["rows"].as_array().expect("rows");
        assert!(
            rows.len() == 3 && rows.iter().all(|n| n.as_u64().is_some()),
            "{at}: rows"
        );
        let lines = &u["lines"];
        assert!(lines[0].as_u64() <= lines[1].as_u64(), "{at}: lines");
        for cell in u["pools"].as_object().expect("pools").keys() {
            assert!(cells.contains(cell), "{at}: pool cell {cell}");
        }
    }
}

/// Every frozen universe: the frozen set is exactly the exam keys, each
/// doc passes the shared envelope core under its own language's family
/// (summary re-derived, constants, sorted rows, pinned tip), names the
/// exam's corpus and scope, excludes only by the two walk reasons, and
/// every unit row is well formed.
#[test]
fn flow_slices_consistent() {
    frozen_set(&parts::SLICES, parts::exams::keys());
    each_slice(|exam, corpus, tip, path, doc| {
        assert!(exam.stage >= Stage::Sampled, "{}: unsampled", exam.lang);
        let name = assert_envelope_core(path, doc, &family(exam.lang));
        assert_eq!(name.as_deref(), Some(corpus), "{path}: corpus");
        let want = json!({
            "schema": parts::SCHEMAS.0,
            "corpus": {"name": corpus, "tip": tip, "lang": exam.lang},
            "scope": exam.scope(),
        });
        parts::assert_keys(path, doc, &want);
        for why in doc["excluded"].as_object().expect("excluded").keys() {
            let known = ["other_extension", "walk_refused"].contains(&why.as_str());
            assert!(known, "{path}: excluded by {why}");
        }
        units_well_formed(path, doc);
    });
}

/// A dynamic unit (the core judges none of it) offers no question:
/// its row keeps the dynamic bit and its table sizes, every pool 0.
#[test]
fn dynamic_units_enter_no_pool() {
    each_slice(|_, _, _, path, doc| {
        for (at, u) in units(path, doc) {
            if u["dynamic"] == json!(true) {
                let pools = u["pools"].as_object().expect("pools");
                let empty = pools.values().all(|n| n.as_u64() == Some(0));
                assert!(empty, "{at}: a dynamic unit with a pool");
            }
        }
    });
}

/// An exam's extensions are its language's, by the product's own row
/// (Lang::extensions) — the scope cannot drift from what `ce` walks as
/// that language, and the exam's name is the language's report name.
#[test]
fn exam_scopes_are_the_languages_extensions() {
    for exam in EXAMS.iter() {
        let exts = exam.language().extensions();
        assert_eq!(exam.exts, exts, "{}: extensions", exam.lang);
    }
}

/// The ladder every exam names is the lowering on disk — a directory
/// and one named exception — and its `ladder_first` is a reason, not
/// a label.
#[test]
fn the_ladder_is_the_lowering() {
    let root = crate::common::repo_root();
    for exam in EXAMS.iter() {
        assert!(exam.ladder_first.len() >= MIN_WHY, "{}: why", exam.lang);
        for spec in exam.ladder {
            let path = spec.trim_start_matches(":!");
            assert!(
                root.join(path).exists(),
                "{}: {spec} names nothing",
                exam.lang
            );
        }
        assert!(
            exam.ladder.iter().any(|s| s.starts_with(":!")),
            "{}: the module table is not excepted",
            exam.lang
        );
    }
}

/// Every exam language's frozen sample verifies; the frozen set is
/// exactly the exam languages.
#[test]
fn flow_samples_verify() {
    frozen_set(&parts::SAMPLES, parts::exams::langs());
    for exam in EXAMS.iter() {
        verify_sample(exam, &exam.sample());
    }
}

/// G9: the verifier refuses every forgery of the battery.
#[test]
fn a_tampered_flow_sample_is_refused() {
    parts::tamper::assert_flow_sample_tampering();
}
