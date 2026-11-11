//! The review table builder (lane v235_licence, phase 2A (b), part 3A):
//! one language's `deadfile-review-<lang>` body from its class doc, the
//! committed first and second readings and the main session's
//! settlement; the evaluator's rows from the review tables and the class
//! docs; the facts the filed review docs recorded. The builder's
//! synthetic tests are eval_deadfile_bars_four.rs; the generator and the
//! CI gate over the committed record are derived.rs and
//! eval_deadfile_review.rs.

use super::bars::Row;
use super::conventions::{Docs, candidates, carries, drawn_for_second_reading};
use super::record::{readings, settlement};
use super::review::{Facts, Readings, agreement, review_row};
use crate::common::ledger::out_file;
use crate::eval_deadfile_parts::{classes::FAMILY, doc_path, generate::pools};
use crate::eval_support::load;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Answers or settlements by (corpus, path).
pub type ByRow = BTreeMap<(String, String), Value>;
type Id = (String, String);

/// The review docs' family name.
pub const REVIEW: &str = "deadfile-review";

fn id(r: &Value) -> Id {
    let s = |k: &str| r[k].as_str().unwrap_or("").to_string();
    (s("corpus"), s("path"))
}

/// One language's review table body: the primary rows of its class doc
/// in stored order, a reserve row only where its first reading exists
/// (it replaced a primary row the auditor could not find).
pub fn review_doc(
    facts: Facts,
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
            let mut row = review_row(facts, r, given);
            row["substitute"] = json!(r["sample"] == "reserve");
            row
        })
        .collect();
    json!({"lang": class_doc["lang"], "agreement": agreement(&rows), "rows": rows})
}

/// A doc of the dead-file family by language, from `CE_DEADFILE_OUT`
/// when it is set (a generator's output), else from the tree.
pub fn per_language(family: &str) -> Vec<Value> {
    pools()
        .0
        .keys()
        .map(|lang| load(&out_file("CE_DEADFILE_OUT", &doc_path(family, lang))))
        .collect()
}

/// Every language's class doc and its review table built from the
/// committed record, languages in order.
pub fn reviewed(facts: Facts) -> (Vec<Value>, Vec<Value>) {
    let classes = per_language(FAMILY);
    let ([first, second], settled) = (readings(), settlement());
    let reviews = classes
        .iter()
        .map(|c| review_doc(facts, c, &first, &second, &settled))
        .collect();
    (classes, reviews)
}

/// The check and tag a filed review recorded for each reading, keyed by
/// (corpus, path) and the answer's four fields: the facts the CI gate
/// rebuilds the review docs from. An answer the filed docs never read is
/// refused by name.
pub fn recorded(filed: &[Value]) -> impl Fn(&Value, &Value) -> (String, Value) {
    let four = |r: &Value| {
        ["answer", "mechanism", "evidence", "searched"]
            .map(|k| r[k].as_str().unwrap_or("").to_string())
    };
    let mut seen = BTreeMap::new();
    for r in filed
        .iter()
        .flat_map(|d| d["rows"].as_array().cloned().unwrap_or_default())
    {
        for reading in [&r["first"], &r["second"]]
            .into_iter()
            .filter(|x| x.is_object())
        {
            let fact = (
                reading["check"].as_str().unwrap_or("").to_string(),
                reading["tag"].clone(),
            );
            seen.insert((id(&r), four(reading)), fact);
        }
    }
    move |row: &Value, a: &Value| {
        let found = seen.get(&(id(row), four(a))).cloned();
        found.unwrap_or_else(|| panic!("{:?}: an answer the filed review docs never read", id(row)))
    }
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
