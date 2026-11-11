//! The real run over the auditors' answers (phase 2B part 2, on the main
//! session's word): the review tables, BARS §3 under the citation input,
//! population B from the passing position rules (population.rs) over every
//! exam corpus, and BARS §4 under every registered reading with the
//! per-class "same under all readings?" table. Writes nothing.
//!   CE_DEADFILE_ROOT=<clones> CE_DEADFILE_COPIES=<scratch dir> CE_DEADFILE_ANSWERS=<answers/>
//!   CE_DEADFILE_ANSWERS2=<answers2/> CE_DEADFILE_SETTLED=<settlement.json> CE_DEADFILE_CONVENTION_DOCS=<convention_docs.json>
//!   cargo test --test it -- --ignored eval_deadfile_parts::audit::real::deadfile_reviews --nocapture

use super::bars::{evaluate, render, same_under_all};
use super::conventions::{Docs, candidates, carries, evaluate as section3};
use super::population::{line, population_b};
use super::review::clone_reader;
use super::tables::{bars_rows, input, opened, passing_rows};
use crate::eval_deadfile_parts::corpora;
use crate::eval_lang_parts::generate::env;
use serde_json::Value;
use std::collections::BTreeSet;

/// Why a §3 judgment does or does not pass.
fn outcome(j: &Value) -> String {
    let rules: Vec<String> = j["refused_by"]
        .as_array()
        .into_iter()
        .flatten()
        .map(Value::to_string)
        .collect();
    match (j["passes"] == true, j["kind"] == "project") {
        (true, _) => "PASSES".into(),
        (false, true) => "stays out by rule 1 (a project's convention)".into(),
        (false, false) => format!("refused by rule {}", rules.join(" and ")),
    }
}

/// A §3 judgment's counts and its three rule facts.
fn counts(j: &Value) -> String {
    let (n, i, u, s) = (&j["rows"], &j["in_use"], &j["unused"], &j["unsure"]);
    let (t, d, p) = (&j["names_tool"], &j["documented"], &j["position"]);
    format!(
        "rows {n} (in_use {i} / unused {u} / unsure {s}); names the tool {t}; documented {d}; position {p}"
    )
}

/// One §3 candidate as printed; the `beside` reading, `gap` and
/// `print_rows` beside it.
fn printed(c: &Value) -> Vec<String> {
    let gap = c["gap"]
        .as_str()
        .map(|g| format!(" | gap: {g}"))
        .unwrap_or_default();
    let mut out = vec![format!(
        "{:<22} {} -> {}{gap}",
        c["name"].as_str().unwrap_or(""),
        counts(c),
        outcome(c)
    )];
    if let Some(b) = c.get("beside").filter(|b| b.is_object()) {
        out.push(format!(
            "{:<22} beside (not evaluated — printed for comparison): {} -> {}",
            "",
            counts(b),
            outcome(b)
        ));
    }
    let rows = c["listed"].as_array().into_iter().flatten();
    out.extend(rows.map(|r| format!("{:<22}   row: {}", "", r.as_str().unwrap_or(""))));
    out
}

#[test]
#[ignore = "reads the auditors' answers and copies every corpus: run only on the main session's word"]
fn deadfile_reviews() {
    let tip = clone_reader(env("CE_DEADFILE_ROOT"));
    let (_, classes, reviews) = opened(&tip);
    let all: Vec<Value> = reviews
        .iter()
        .flat_map(|r| r["rows"].as_array().cloned().unwrap_or_default())
        .collect();
    let docs: Docs = serde_json::from_value(
        input("CE_DEADFILE_CONVENTION_DOCS").expect("CE_DEADFILE_CONVENTION_DOCS"),
    )
    .expect("CE_DEADFILE_CONVENTION_DOCS: {candidate: object}");
    let table = section3(&all, &docs);
    println!("== section 3");
    table.iter().flat_map(printed).for_each(|l| println!("{l}"));
    let passing: Vec<_> = candidates()
        .into_iter()
        .filter(|c| {
            table
                .iter()
                .any(|t| t["name"] == c.name && t["passes"] == true)
        })
        .collect();
    println!(
        "passing set: {:?}",
        passing.iter().map(|c| c.name).collect::<Vec<_>>()
    );
    println!("== population B");
    let mut still = BTreeSet::new();
    for (corpus, _) in corpora() {
        let m = population_b(corpus, &|p| {
            passing.iter().any(|c| carries(c, &docs, corpus, p))
        });
        println!("{}", line(corpus, &m));
        still.extend(m.after.into_iter().map(|p| (corpus.to_string(), p)));
    }
    let rows = bars_rows(&reviews, &classes)
        .unwrap_or_else(|b| panic!("{} rows block the evaluation:\n{}", b.len(), b.join("\n")));
    let report = evaluate(&rows, &passing_rows(&all, &table, &docs), Some(&still));
    println!("== section 4\n{}", render(&report));
    println!("== same under all readings?");
    same_under_all(&report).iter().for_each(|l| println!("{l}"));
}
