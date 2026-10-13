//! The code-query family's three faces (plan v2.31 step 2, design
//! booklet §4.6) agree on ONE document: `ce query --format json`
//! prints the library face byte for byte, the MCP tools `query` and
//! `rules` relay it, and `ce rules` reads its exit code off the same
//! document's `counts.violations`. The fixture is built to be judged:
//! a named entry importing one file and one file nothing reaches, so
//! `dead(F)` must name exactly what `ce deadcode` names. This file holds
//! the query half and the fixture; the rules half and the MCP relay sit
//! in rules_face.rs (split, plan v2.33 step 11b) and read both from here.

use crate::common;
use codeeraser::faces;
use codeeraser::query::PRELUDE;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// The fixture tree: a file's path on a `--- ` line, its text under it.
const FIXTURE: &str = "\
--- Cargo.toml
[package]
name = \"fixture\"
version = \"0.1.0\"
edition = \"2021\"
--- src/main.rs
mod b;
fn main() {
    b::go();
}
--- src/b.rs
pub fn go() {}
--- src/c.rs
pub fn orphan() {}
";

/// `body|where|what` per line: a lexical fault, an unreadable glob, a
/// syntax error and an unsafe variable — exit 2, each named at its place.
const FAULTS: &str = "\
mention(\"x, F)|query 1:12|unterminated string
in(F, \"[\")|query 1:10|
file(F), F = |query 1:16|syntax error
file(F), G != F|query 1:13|unsafe variable
";

pub(crate) fn seeded(name: &str) -> PathBuf {
    common::seeded_tree(name, FIXTURE)
}

/// `ce <args…> . --core <core>` (+ `--format json`): (exit, stdout, stderr).
/// Also the rule pack's runner (`rules_pack`).
pub(crate) fn run(dir: &Path, args: &[&str], json: bool) -> (Option<i32>, String, String) {
    let core = common::core_bin();
    let mut full: Vec<&str> = args.to_vec();
    full.extend_from_slice(&[".", "--core", &core]);
    if json {
        full.extend_from_slice(&["--format", "json"]);
    }
    common::ce_triple(dir, &full, &[])
}

pub(crate) fn json_face(dir: &Path, args: &[&str]) -> (Option<i32>, Value) {
    let (code, out, err) = run(dir, args, true);
    let doc = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{args:?}: {e}\n{out}\n{err}"));
    (code, doc)
}

pub(crate) fn values(doc: &Value) -> Vec<Vec<String>> {
    doc["answers"]
        .as_array()
        .expect("answers")
        .iter()
        .map(|a| serde_json::from_value(a["values"].clone()).expect("values"))
        .collect()
}

pub(crate) fn query_face(dir: &Path, body: &str, why: bool) -> Value {
    let answer = faces::query(
        dir,
        &common::core_bin(),
        (body, why),
        None,
        Default::default(),
    );
    faces::doc(answer).expect("face")
}

#[test]
fn the_cli_prints_the_library_face_and_dead_f_is_the_deadcode_verdict() {
    let dir = seeded("query-cli");
    let (code, doc) = json_face(&dir, &["query", "dead(F)"]);
    assert_eq!(code, Some(0));
    assert_eq!(doc, query_face(&dir, "dead(F)", false));
    assert_eq!(doc["schema"], common::stub_core::catalogue_schema("query"));
    assert!(
        doc["degraded"].is_null() && doc["errors"] == json!([]),
        "{doc}"
    );
    assert_eq!(
        doc["goals"],
        json!([{"goal": 0, "answers": [0], "kind": "query", "name": null, "columns": ["F"], "sorts": ["node"]}])
    );
    assert_eq!(values(&doc), [["src/c.rs"]]);
    assert_eq!(doc["program"]["prelude"], 8);
    assert!(doc["program"]["rules_file"].is_null());
    // the graph judgment's own road names the same file
    let (_, dead) = json_face(&dir, &["deadcode"]);
    let named: Vec<&str> = dead["dead"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| r["name"].as_str())
        .collect();
    assert_eq!(
        (named, dead["degraded"].is_null()),
        (vec!["src/c.rs"], true)
    );
    // --why carries the derivation, and the console prints it under the answer
    let (_, why) = json_face(&dir, &["query", "dead(F)", "--why"]);
    assert_eq!(why, query_face(&dir, "dead(F)", true));
    let chain: Vec<String> = why["proof"]
        .as_array()
        .expect("proof")
        .iter()
        .map(|p| {
            format!(
                "{}({})",
                p["pred"].as_str().unwrap(),
                p["args"][0].as_str().unwrap()
            )
        })
        .collect();
    assert_eq!(chain, ["?-(src/c.rs)", "dead(src/c.rs)", "file(src/c.rs)"]);
    let (code, text, _) = run(&dir, &["query", "dead(F)", "--why"], false);
    assert_eq!(code, Some(0));
    let head = "?- F: 1 answer(s)\n  src/c.rs\n    dead(src/c.rs) (clause 3)\n      file(src/c.rs) (fact)\n";
    assert!(text.starts_with(head), "{text}");
    assert!(text.trim_end().ends_with("proof node(s)"), "{text}");
}

#[test]
fn sugar_aggregates_arithmetic_and_every_program_error_go_the_same_road() {
    let dir = seeded("query-shapes");
    let (_, count) = json_face(&dir, &["query", "N = count(F : in(F, \"*.rs\"))"]);
    assert_eq!(values(&count), [["3"]]);
    let (_, arith) = json_face(&dir, &["query", "lines(F, N), M = N * 2, M > 4"]);
    assert_eq!(values(&arith), [["src/main.rs", "4", "8"]]);
    let (_, live) = json_face(
        &dir,
        &["query", "node(F, file), lang(F, rust), not dead(F)"],
    );
    assert_eq!(values(&live).len(), 2, "{live}");
    for row in FAULTS.lines() {
        let [body, at, what] = row.split('|').collect::<Vec<_>>()[..] else {
            panic!("{row}")
        };
        let (code, doc) = json_face(&dir, &["query", body]);
        assert_eq!(code, Some(2), "{body}");
        assert_eq!(doc["errors"][0]["at"], at, "{body}: {doc}");
        let said = doc["errors"][0]["what"].as_str().expect("what");
        assert!(said.contains(what), "{body}: {said}");
        assert_eq!(doc["answers"], json!([]));
    }
    let (code, text, _) = run(&dir, &["query", "file(F), F = "], false);
    assert_eq!(
        (code, text.as_str()),
        (Some(2), "error at query 1:16: syntax error\n")
    );
    let (code, prelude, _) = common::ce_triple(&dir, &["query", "--prelude", "."], &[]);
    assert_eq!((code, prelude.as_str()), (Some(0), PRELUDE));
}
