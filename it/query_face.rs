//! The code-query family's three faces (plan v2.31 step 2, design
//! booklet §4.6) agree on ONE document: `ce query --format json`
//! prints the library face byte for byte, the MCP tools `query` and
//! `rules` relay it, and `ce rules` reads its exit code off the same
//! document's `counts.violations`. The fixture is built to be judged:
//! a named entry importing one file and one file nothing reaches, so
//! `dead(F)` must name exactly what `ce deadcode` names.

use crate::common;
use codeeraser::query::PRELUDE;
use codeeraser::query::face::{RULES_SCHEMA_ID, SCHEMA_ID};
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

/// Two assertions: one every tree with a dead file violates, one none does.
const RULES: &str = "assert no_dead(F) :- dead(F).\nassert huge(F, N) :- lines(F, N), N > 1000.\n";

/// `body|where|what` per line: a lexical fault, an unreadable glob, a
/// syntax error and an unsafe variable — exit 2, each named at its place.
const FAULTS: &str = "\
mention(\"x, F)|query 1:12|unterminated string
in(F, \"[\")|query 1:10|
file(F), F = |query 1:16|syntax error
file(F), G != F|query 1:13|unsafe variable
";

/// The second assertion alone: a rules file this tree passes.
fn clean() -> &'static str {
    RULES
        .split_once('\n')
        .map(|(_, rest)| rest)
        .expect("two rules")
}

fn seeded(name: &str) -> PathBuf {
    common::seeded_tree(name, FIXTURE)
}

/// `ce <args…> . --core <core>` (+ `--format json`): (exit, stdout, stderr).
fn run(dir: &Path, args: &[&str], json: bool) -> (Option<i32>, String, String) {
    let core = common::core_bin();
    let mut full: Vec<&str> = args.to_vec();
    full.extend_from_slice(&[".", "--core", &core]);
    if json {
        full.extend_from_slice(&["--format", "json"]);
    }
    common::ce_triple(dir, &full, &[])
}

fn json_face(dir: &Path, args: &[&str]) -> (Option<i32>, Value) {
    let (code, out, err) = run(dir, args, true);
    let doc = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{args:?}: {e}\n{out}\n{err}"));
    (code, doc)
}

fn values(doc: &Value) -> Vec<Vec<String>> {
    doc["answers"]
        .as_array()
        .expect("answers")
        .iter()
        .map(|a| serde_json::from_value(a["values"].clone()).expect("values"))
        .collect()
}

fn query_face(dir: &Path, body: &str, why: bool) -> Value {
    codeeraser::faces::query(dir, &common::core_bin(), body, why, None).expect("face")
}

#[test]
fn the_cli_prints_the_library_face_and_dead_f_is_the_deadcode_verdict() {
    let dir = seeded("query-cli");
    let (code, doc) = json_face(&dir, &["query", "dead(F)"]);
    assert_eq!(code, Some(0));
    assert_eq!(doc, query_face(&dir, "dead(F)", false));
    assert_eq!(doc["schema"], SCHEMA_ID);
    assert!(
        doc["degraded"].is_null() && doc["errors"] == json!([]),
        "{doc}"
    );
    assert_eq!(
        doc["goals"],
        json!([{"goal": 0, "kind": "query", "name": null, "columns": ["F"], "sorts": ["node"]}])
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

#[test]
fn rules_is_a_gate_on_the_documents_violations() {
    let dir = seeded("query-rules");
    let (code, text, _) = run(&dir, &["rules"], false);
    assert_eq!(code, Some(0));
    assert!(
        text.starts_with("rules: no rules file — zero assertions\n"),
        "{text}"
    );
    std::fs::write(dir.join("ce.rules"), RULES).expect("ce.rules");
    let (code, doc) = json_face(&dir, &["rules"]);
    assert_eq!(code, Some(1));
    assert_eq!(
        doc,
        codeeraser::faces::rules(&dir, &common::core_bin(), None, false).unwrap()
    );
    assert_eq!(doc["schema"], RULES_SCHEMA_ID);
    assert!(
        doc["program"]["rules_file"]
            .as_str()
            .expect("label")
            .ends_with("ce.rules")
    );
    assert_eq!(doc["counts"]["violations"], 1);
    let goals: Vec<(&str, &str)> = doc["goals"]
        .as_array()
        .expect("goals")
        .iter()
        .map(|g| (g["kind"].as_str().unwrap(), g["name"].as_str().unwrap()))
        .collect();
    assert_eq!(goals, [("assert", "no_dead"), ("assert", "huge")]);
    assert_eq!(values(&doc), [["src/c.rs"]]);
    let derived = doc["proof"]
        .as_array()
        .expect("proof")
        .iter()
        .any(|p| p["pred"] == "dead");
    assert!(
        derived,
        "a violation carries its derivation: {}",
        doc["proof"]
    );
    let (code, text, _) = run(&dir, &["rules"], false);
    assert_eq!(code, Some(1));
    let head = "assert no_dead(F): 1 violation(s)\n  src/c.rs\n    dead(src/c.rs) (clause 3)\n      \
                file(src/c.rs) (fact)\nassert huge(F, N): ok\n";
    assert!(text.starts_with(head), "{text}");
    // a question builds on the same rules; its exit code is not the gate's
    let (code, q) = json_face(&dir, &["query", "dead(F)"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        (
            q["goals"].as_array().unwrap().len(),
            &q["counts"]["violations"]
        ),
        (3, &json!(1))
    );
}

#[test]
fn the_rules_file_is_named_by_flag_config_or_default_and_must_exist() {
    let dir = seeded("query-rules-file");
    std::fs::write(dir.join("ce.rules"), RULES).expect("ce.rules");
    std::fs::write(dir.join("clean.rules"), clean()).expect("clean.rules");
    let (code, passed) = json_face(&dir, &["rules", "--file", "clean.rules"]);
    assert_eq!(
        (code, &passed["counts"]["violations"]),
        (Some(0), &json!(0))
    );
    assert!(
        passed["program"]["rules_file"]
            .as_str()
            .unwrap()
            .ends_with("clean.rules")
    );
    let (code, _, err) = run(&dir, &["rules", "--file", "missing.rules"], false);
    assert_eq!(code, Some(2));
    assert!(err.contains("read rules file"), "{err}");
    // `[rules] file` names the default over ce.rules; a declared file must exist
    common::declare(&dir, "[rules]\nfile = \"clean.rules\"\n");
    let (code, declared) = json_face(&dir, &["rules"]);
    assert_eq!(code, Some(0));
    assert!(
        declared["program"]["rules_file"]
            .as_str()
            .unwrap()
            .ends_with("clean.rules")
    );
    common::declare(&dir, "[rules]\nfile = \"gone.rules\"\n");
    let (code, _, err) = run(&dir, &["rules"], false);
    assert_eq!((code, err.contains("gone.rules")), (Some(2), true), "{err}");
    // a rules file that does not parse is a program error, exit 2
    std::fs::write(dir.join("bad.rules"), "assert x(F) :- dead(F)\n").expect("bad.rules");
    let (code, bad) = json_face(&dir, &["rules", "--file", "bad.rules"]);
    assert_eq!(code, Some(2));
    assert_eq!(bad["errors"][0]["what"], "syntax error");
    assert_eq!(bad["counts"]["violations"], 0);
}

#[test]
fn the_mcp_tools_relay_the_faces_and_refuse_a_question_that_is_not_one() {
    let dir = seeded("query-mcp");
    let core = common::core_bin();
    std::fs::write(dir.join("ce.rules"), RULES).expect("ce.rules");
    let mut s = common::McpSession::over(&dir);
    let got = s.call(1, "query", json!({"body": "dead(F)", "why": true}));
    assert_eq!(got["isError"], false, "{got}");
    assert_eq!(
        got["content"][0]["text"],
        query_face(&dir, "dead(F)", true).to_string()
    );
    let rules = s.call(2, "rules", json!({}));
    assert_eq!(
        rules["content"][0]["text"],
        codeeraser::faces::rules(&dir, &core, None, false)
            .unwrap()
            .to_string()
    );
    let filed = s.call(3, "rules", json!({"file": "ce.rules", "why": true}));
    assert_eq!(
        filed["content"][0]["text"],
        codeeraser::faces::rules(&dir, &core, Some(Path::new("ce.rules")), true)
            .unwrap()
            .to_string()
    );
    let none = s.call(4, "query", json!({}));
    assert_eq!(none["isError"], true);
    assert!(
        none["content"][0]["text"]
            .as_str()
            .expect("why")
            .contains("required")
    );
    let missing = s.call(5, "rules", json!({"file": "missing.rules"}));
    assert_eq!(missing["isError"], true, "{missing}");
    s.finish();
}
