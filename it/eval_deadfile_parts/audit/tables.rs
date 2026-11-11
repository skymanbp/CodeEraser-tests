//! The review table builder (lane v235_licence, phase 2A (b)): one
//! language's `deadfile-review-<lang>` body from its class doc, the
//! first and second readings and the main session's settlements; the
//! evaluator's rows from the review tables and the class docs. The real
//! run is real.rs; the builder's tests use a synthetic fixture
//! (eval_deadfile_bars_four.rs).

use super::bars::Row;
use super::conventions::{Docs, candidates, carries, drawn_for_second_reading};
use super::review::{Line, Readings, Tip, agreement, answer_lines, by_row, review_row};
use crate::eval_deadfile_parts::{classes::FAMILY, doc_path, generate::pools};
use crate::{eval_lang_parts::generate::env, eval_support::load};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Answers or settlements by (corpus, path).
pub type ByRow = BTreeMap<(String, String), Value>;
type Id = (String, String);

fn id(r: &Value) -> Id {
    let s = |k: &str| r[k].as_str().unwrap_or("").to_string();
    (s("corpus"), s("path"))
}

/// One language's review table body: the primary rows of its class doc
/// in stored order, a reserve row only where its first reading exists
/// (it replaced a primary row the auditor could not find).
pub fn review_doc(
    tip: Tip,
    class_doc: &Value,
    first: &ByRow,
    second: &ByRow,
    settled: &ByRow,
) -> Value {
    let rows: Vec<Value> = class_doc["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .filter(|r| {
            r["sample"] == "primary" || (r["sample"] == "reserve" && first.contains_key(&id(r)))
        })
        .map(|r| {
            let k = id(r);
            let given = Readings {
                first: first.get(&k),
                second: second.get(&k),
                settled: settled.get(&k).and_then(|s| s["answer"].as_str()),
                candidate: drawn_for_second_reading(&k.1),
            };
            let mut row = review_row(tip, r, given);
            row["substitute"] = json!(r["sample"] == "reserve");
            row
        })
        .collect();
    json!({"lang": class_doc["lang"], "agreement": agreement(&rows), "rows": rows})
}

/// Every language's class doc and its review table, languages in order.
pub fn reviewed(
    tip: Tip,
    first: &ByRow,
    second: &ByRow,
    settled: &ByRow,
) -> (Vec<Value>, Vec<Value>) {
    let classes: Vec<Value> = pools()
        .0
        .keys()
        .map(|lang| load(&doc_path(FAMILY, lang)))
        .collect();
    let reviews = classes
        .iter()
        .map(|c| review_doc(tip, c, first, second, settled))
        .collect();
    (classes, reviews)
}

/// The real answers of both readings (`CE_DEADFILE_ANSWERS`,
/// `CE_DEADFILE_ANSWERS2`) and the review tables they make with the
/// settlement: (both readings' lines, class docs, review tables).
pub fn opened(tip: Tip) -> ([Vec<Line>; 2], Vec<Value>, Vec<Value>) {
    let lines = [
        answer_lines(&env("CE_DEADFILE_ANSWERS")),
        answer_lines(&env("CE_DEADFILE_ANSWERS2")),
    ];
    let (classes, reviews) = reviewed(tip, &by_row(&lines[0]), &by_row(&lines[1]), &settlements());
    (lines, classes, reviews)
}

/// Per row its (language, k1, k1_strict); per (language, corpus) its
/// BARS §1 weight, pool / primary.
type Index = (BTreeMap<Id, (String, String, String)>, BTreeMap<Id, f64>);

fn index(classes: &[Value]) -> Index {
    let (mut class_of, mut weight) = (BTreeMap::new(), BTreeMap::new());
    for doc in classes {
        let lang = doc["lang"].as_str().expect("lang").to_string();
        for (corpus, c) in doc["per_corpus"].as_object().expect("per_corpus") {
            let n = |k: &str| c[k].as_f64().unwrap_or(0.0);
            let w = if n("primary") > 0.0 {
                n("pool") / n("primary")
            } else {
                0.0
            };
            weight.insert((lang.clone(), corpus.clone()), w);
        }
        for r in doc["rows"].as_array().expect("rows") {
            let s = |k: &str| r[k].as_str().unwrap_or("").to_string();
            class_of.insert(id(r), (lang.clone(), s("k1"), s("k1_strict")));
        }
    }
    (class_of, weight)
}

/// The evaluator's rows: every review row with a final answer, its two
/// classes and its weight; Err with every row that blocks.
pub fn bars_rows(reviews: &[Value], classes: &[Value]) -> Result<Vec<Row>, Vec<String>> {
    let (class_of, weight) = index(classes);
    let (mut out, mut blocked) = (Vec::new(), Vec::new());
    for r in reviews
        .iter()
        .flat_map(|d| d["rows"].as_array().cloned().unwrap_or_default())
    {
        let (corpus, path) = id(&r);
        if r["answer"].is_null() {
            blocked.push(format!(
                "{corpus} {path}: {}",
                r["status"].as_str().unwrap_or("")
            ));
            continue;
        }
        let (lang, k1, k1_strict) = class_of[&(corpus.clone(), path.clone())].clone();
        out.push(Row {
            weight: weight[&(lang.clone(), corpus.clone())],
            answer: r["answer"].as_str().unwrap_or("").into(),
            named: r["tag"] == "named",
            corpus,
            path,
            lang,
            k1,
            k1_strict,
        });
    }
    if blocked.is_empty() {
        Ok(out)
    } else {
        Err(blocked)
    }
}

/// A JSON file named by an environment variable, or None when unset.
pub fn input(var: &str) -> Option<Value> {
    let path = std::env::var(var).ok()?;
    Some(serde_json::from_str(&std::fs::read_to_string(&path).expect(var)).expect(var))
}

/// The audited rows carrying a §3 convention that passes.
pub fn passing_rows(rows: &[Value], table: &[Value], docs: &Docs) -> BTreeSet<Id> {
    let names: Vec<&str> = table
        .iter()
        .filter(|c| c["passes"] == true)
        .filter_map(|c| c["name"].as_str())
        .collect();
    let passing: Vec<_> = candidates()
        .into_iter()
        .filter(|c| names.contains(&c.name))
        .collect();
    rows.iter()
        .map(id)
        .filter(|(c, p)| passing.iter().any(|k| carries(k, docs, c, p)))
        .collect()
}

/// The main session's settlement rows, by (corpus, path).
pub fn settlements() -> ByRow {
    let rows = input("CE_DEADFILE_SETTLED").and_then(|v| v.as_array().cloned());
    rows.unwrap_or_default()
        .into_iter()
        .map(|s| (id(&s), s))
        .collect()
}
