//! The dead-file exam's gate over its committed record (lane
//! v235_licence, part 3A): the record is the main session's bytes; every
//! field of the filed review docs follows from the record and the check
//! and tag facts the docs recorded (the clone-reading re-derive is
//! `derived::deadfile_reviews_rederive`); population B stands on the
//! frozen slices and on BARS §3's passing set; the registry page carries
//! every table the committed data renders.

use crate::common::{ledger::out_file, repo_root};
use crate::eval_deadfile_parts::audit::bars::evaluate;
use crate::eval_deadfile_parts::audit::conventions::evaluate as section3;
use crate::eval_deadfile_parts::audit::derived::{
    POPULATION_SCHEMA, REVIEW_SCHEMA, passing, population_path,
};
use crate::eval_deadfile_parts::audit::population::frozen_dead;
use crate::eval_deadfile_parts::audit::record::{ANSWERS, at, convention_docs, pinned, text};
use crate::eval_deadfile_parts::audit::tables::{
    REVIEW, bars_rows, passing_rows, per_language, recorded, reviewed,
};
use crate::eval_deadfile_parts::audit::{page, page_bars};
use crate::eval_deadfile_parts::{SAMPLES, corpora};
use crate::eval_support::{content_sha, load};
use serde_json::Value;
use std::collections::BTreeSet;

#[test]
fn the_record_is_the_main_sessions_bytes() {
    for (rel, sha) in pinned() {
        assert_eq!(content_sha(&text(rel)), sha, "{rel}: not the copied bytes");
    }
    for dir in [ANSWERS.0, ANSWERS.1] {
        let names: BTreeSet<String> = std::fs::read_dir(at(dir))
            .expect(dir)
            .flatten()
            .map(|e| format!("{dir}/{}", e.file_name().to_string_lossy()))
            .collect();
        let listed: BTreeSet<String> = pinned()
            .into_iter()
            .map(|(p, _)| p.to_string())
            .filter(|p| p.starts_with(&format!("{dir}/")))
            .collect();
        assert_eq!(
            names, listed,
            "{dir}: a file not pinned, or a pinned file missing"
        );
    }
}

#[test]
fn every_review_row_follows_from_the_record() {
    let filed = per_language(REVIEW);
    let (_, rebuilt) = reviewed(&recorded(&filed));
    for (doc, mine) in filed.iter().zip(&rebuilt) {
        assert_eq!(doc["schema"], REVIEW_SCHEMA);
        for key in ["lang", "agreement", "rows"] {
            assert_eq!(
                doc[key], mine[key],
                "{}: `{key}` does not follow from the record",
                doc["lang"]
            );
        }
    }
    let failing: Vec<String> = filed
        .iter()
        .flat_map(|d| d["rows"].as_array().cloned().unwrap_or_default())
        .flat_map(|r| [("first", r.clone()), ("second", r)])
        .filter(|(k, r)| {
            r[k]["check"]
                .as_str()
                .is_some_and(|c| c.starts_with("fail"))
        })
        .map(|(k, r)| format!("{} {} ({k}): {}", r["corpus"], r["path"], r[k]["check"]))
        .collect();
    assert!(
        failing.is_empty(),
        "answers that fail BARS section 1's check:\n{}",
        failing.join("\n")
    );
}

#[test]
fn population_b_stands_on_the_slices_and_the_passing_set() {
    let doc = load(&out_file("CE_DEADFILE_OUT", &population_path()));
    assert_eq!(doc["schema"], POPULATION_SCHEMA);
    let names: Vec<String> = passing(&per_language(REVIEW))
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        doc["passing"],
        serde_json::json!(names),
        "the passing set is BARS section 3's"
    );
    let set = |v: &Value| -> BTreeSet<String> {
        v.as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect()
    };
    let listed = doc["corpora"].as_array().expect("corpora");
    assert_eq!(
        listed.len(),
        corpora().len(),
        "population B names every corpus of the exam table"
    );
    for ((corpus, _), r) in corpora().into_iter().zip(listed) {
        let (slice, still) = (frozen_dead(corpus), set(&r["still_dead"]));
        assert_eq!(r["corpus"], corpus);
        assert_eq!(r["before"], slice.len(), "{corpus}: not the frozen slice");
        assert!(
            still.is_subset(&slice) && r["after"] == still.len(),
            "{corpus}: still-dead rows outside the slice"
        );
        let dropped: BTreeSet<String> = slice.difference(&still).cloned().collect();
        let moved: BTreeSet<String> = set(&r["matched_dead"])
            .union(&set(&r["reached_beyond"]))
            .cloned()
            .collect();
        assert_eq!(
            dropped, moved,
            "{corpus}: dropped rows are not the matched rows and their reach"
        );
    }
}

/// Every table the committed data renders, in page order.
fn rendered() -> Vec<String> {
    let reviews = per_language(REVIEW);
    let rows: Vec<Value> = reviews
        .iter()
        .flat_map(|r| r["rows"].as_array().cloned().unwrap_or_default())
        .collect();
    let (classes, _) = reviewed(&recorded(&reviews));
    let docs = convention_docs();
    let table = section3(&rows, &docs);
    let pop = load(&out_file("CE_DEADFILE_OUT", &population_path()));
    let still: BTreeSet<(String, String)> = pop["corpora"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|r| {
            let corpus = r["corpus"].as_str().unwrap_or("").to_string();
            r["still_dead"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(move |p| (corpus.clone(), p.to_string()))
                .collect::<Vec<_>>()
        })
        .collect();
    let bars = bars_rows(&reviews, &classes).expect("every row settled");
    let report = evaluate(&bars, &passing_rows(&rows, &table, &docs), Some(&still));
    let agreement = crate::eval_deadfile_parts::audit::review::agreement(&rows);
    vec![
        page::record_table(&pinned()),
        page::sample_table(&per_language(SAMPLES)),
        page::readings_table(&rows, &agreement),
        page::settled_table(&rows),
        page::section3_table(&table),
        page::population_table(&pop),
        page_bars::section4_table(&report),
        page_bars::flips_table(&report),
        page_bars::same_table(&report),
        page_bars::outcomes(&report),
        page_bars::outcome_sentence(&report, &passing(&reviews))
            .expect("an outcome the page has words for"),
    ]
}

#[test]
fn the_registry_page_states_the_committed_numbers() {
    let page = std::fs::read_to_string(repo_root().join("docs/EVAL-SET-DEADFILE.md"))
        .expect("registry page")
        .replace("\r\n", "\n");
    let missing: Vec<String> = rendered()
        .into_iter()
        .filter(|b| !page.contains(b.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "docs/EVAL-SET-DEADFILE.md lacks these rendered blocks:\n\n{}",
        missing.join("\n\n")
    );
    let record: Vec<String> = pinned()
        .into_iter()
        .filter(|(p, _)| p.ends_with(".md"))
        .map(|(p, _)| text(p))
        .collect();
    let quoted = page
        .split("## The bars, as frozen")
        .nth(1)
        .and_then(|t| t.split("## Who answered").next())
        .expect("the bars section");
    let cut: Vec<&str> = quoted
        .lines()
        .filter_map(|l| l.strip_prefix("> "))
        .filter(|l| !l.starts_with("<!-- ce:allow(docdup)"))
        .collect();
    assert!(!cut.is_empty(), "the bars section quotes nothing");
    for line in cut {
        assert!(
            record.iter().any(|r| r.lines().any(|l| l == line)),
            "quoted but not a line of the record: {line}"
        );
    }
}

/// The rendered blocks in page order, each after a `=== block` line —
/// what a page writer pastes (the page itself is never written by a leg).
#[test]
#[ignore = "prints the registry page's tables for the page writer"]
fn registry_tables() {
    for block in rendered() {
        println!("=== block\n{block}");
    }
}
