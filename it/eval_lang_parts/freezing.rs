//! What both exam families freeze through (split from generate.rs at
//! its 300-line soft line when the flow exams became the second
//! reader): the frozen doc's envelope and the product's walk over a
//! pinned tree for any row shape.

use super::generate::{blob, product_walk, tree_paths};
use crate::eval_support::generated_from;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

/// The envelope every frozen exam doc carries — its schema, method,
/// pre-registered constants and provenance stamp — laid over the
/// family's own keys (the language exams' and the flow exams' docs
/// both freeze through it).
pub(crate) fn frozen(schema: &str, method: &str, constants: Value, mut body: Value) -> Value {
    let head = json!({
        "schema": schema, "method": method, "constants": constants,
        "generated_from": generated_from(),
    });
    for (key, value) in head.as_object().expect("object") {
        body[key] = value.clone();
    }
    body
}

/// The walk itself, for any row shape: `in_scope` is the exam's
/// extension test, `row` the frozen row of one in-scope file's path
/// and text (the language exams' site row, the flow exams' unit row).
pub(crate) fn walked(
    repo: &str,
    tip: &str,
    in_scope: &dyn Fn(&str) -> bool,
    row: &dyn Fn(&str, &str) -> Value,
) -> (Vec<Value>, BTreeMap<&'static str, u64>) {
    let mut scope = product_walk(repo);
    let (mut files, mut excluded) = (Vec::new(), BTreeMap::new());
    for path in tree_paths(repo, tip) {
        let tally = if !in_scope(&path) {
            "other_extension"
        } else if !scope.contains(Path::new(&path)) {
            "walk_refused"
        } else {
            files.push(row(&path, &blob(repo, tip, &path)));
            continue;
        };
        *excluded.entry(tally).or_insert(0) += 1;
    }
    files.sort_by(|a: &Value, b: &Value| a["path"].as_str().cmp(&b["path"].as_str()));
    (files, excluded)
}
