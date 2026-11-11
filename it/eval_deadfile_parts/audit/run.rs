//! Phase 2B part 1's run over the real answers (lane v235_licence): the
//! mechanical evidence check of BARS §1 over every answer of both
//! readings, each failure named by (file, row, why); the review tables
//! built in memory with the main session's settlement; the totals the
//! main session checks against its own. Writes nothing.
//!   CE_DEADFILE_ROOT=<clones> CE_DEADFILE_ANSWERS=<answers/> CE_DEADFILE_ANSWERS2=<answers2/> CE_DEADFILE_SETTLED=<settlement.json>
//!   cargo test --test it -- --ignored eval_deadfile_parts::audit::run::deadfile_answers --nocapture

use super::review::{Line, Tip, agreement, check, clone_reader};
use super::tables::{ByRow, opened};
use crate::eval_lang_parts::generate::env;
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn key(v: &Value) -> (String, String) {
    let s = |k: &str| v[k].as_str().unwrap_or("").to_string();
    (s("corpus"), s("path"))
}

/// The evidence check over one reading: every failure and every
/// convention-only `in_use`, named by (file, row, corpus, path, check).
fn checked(tip: Tip, rows: &ByRow, answers: &[Line]) -> (Vec<String>, Vec<String>) {
    let (mut fail, mut convention) = (Vec::new(), Vec::new());
    for (file, line, a) in answers {
        let (corpus, path) = key(a);
        let got = match rows.get(&(corpus.clone(), path.clone())) {
            Some(row) => check(tip, row, a),
            None => "fail: not a row of any sample".to_string(),
        };
        let at = format!("{file}:{line} {corpus} {path}: {got}");
        match got.as_str() {
            "convention" => convention.push(at),
            g if g.starts_with("fail") => fail.push(at),
            _ => {}
        }
    }
    (fail, convention)
}

/// The totals the main session compares with its own.
fn totals(rows: &[Value]) -> Value {
    let count = |at: &str, word: &str| {
        rows.iter()
            .filter(|r| r.pointer(at) == Some(&json!(word)))
            .count()
    };
    let tally = |at: &str| json!({"in_use": count(at, "in_use"), "unused": count(at, "unused"), "unsure": count(at, "unsure")});
    let differ: Vec<String> = rows
        .iter()
        .filter(|r| !r["second"].is_null() && r["first"]["answer"] != r["second"]["answer"])
        .map(|r| {
            format!(
                "{} {} ({} / {}) -> {} [{}]",
                r["corpus"],
                r["path"],
                r["first"]["answer"],
                r["second"]["answer"],
                r["answer"],
                r["status"]
            )
        })
        .collect();
    json!({
        "first_rows": rows.len(),
        "first": tally("/first/answer"),
        "first_tags": {"named": count("/first/tag", "named"), "generic": count("/first/tag", "generic")},
        "twice": rows.iter().filter(|r| !r["second"].is_null()).count(),
        "agreement": agreement(rows),
        "two_different_answers": differ,
        "unsettled": rows.iter().filter(|r| r["answer"].is_null()).count(),
        "substitutes": rows.iter().filter(|r| r["substitute"] == true).count(),
        "final": tally("/answer"),
    })
}

/// Rows read twice that the BARS §1 rule did not predict, and rows it
/// predicted that were not read twice.
fn twice_set(rows: &[Value]) -> Value {
    let ids = |pick: &dyn Fn(&Value) -> bool| -> BTreeSet<String> {
        let mine = rows.iter().filter(|r| pick(r));
        mine.map(|r| format!("{} {}", r["corpus"], r["path"]))
            .collect()
    };
    let read = ids(&|r| !r["second"].is_null());
    let predicted = ids(&|r| r["twice"].as_array().is_some_and(|t| !t.is_empty()));
    json!({"read_not_predicted": read.difference(&predicted).collect::<Vec<_>>(), "predicted_not_read": predicted.difference(&read).collect::<Vec<_>>()})
}

#[test]
#[ignore = "reads the auditors' answers (phase 2B part 1, on the main session's word)"]
fn deadfile_answers() {
    let tip = clone_reader(env("CE_DEADFILE_ROOT"));
    let ([one, two], classes, reviews) = opened(&tip);
    let sample: ByRow = classes
        .iter()
        .flat_map(|c| c["rows"].as_array().cloned().unwrap_or_default())
        .map(|r| (key(&r), r))
        .collect();
    for (name, answers) in [("first reading", &one), ("second reading", &two)] {
        let (fail, convention) = checked(&tip, &sample, answers);
        let (n, f, c) = (answers.len(), fail.len(), convention.len());
        println!(
            "{name}: {n} answers, {f} fail the check, {c} cite no existing path:line (convention)"
        );
        fail.iter()
            .chain(&convention)
            .for_each(|f| println!("   {f}"));
    }
    let rows: Vec<Value> = reviews
        .iter()
        .flat_map(|r| r["rows"].as_array().cloned().unwrap_or_default())
        .collect();
    println!(
        "totals: {}",
        serde_json::to_string_pretty(&totals(&rows)).expect("json")
    );
    println!(
        "second-reading set against the BARS rule: {}",
        twice_set(&rows)
    );
}
