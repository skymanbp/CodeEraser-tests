//! The rules half of the code-query family's faces (plan v2.31 step 2,
//! split out of query_face.rs, plan v2.33 step 11b): `ce rules` reads
//! its exit code off the query document's `counts.violations`, the
//! rules file is named by flag, `[rules] file` or the default, and the
//! MCP tools `query` and `rules` relay the library faces. The fixture
//! and the CLI / library roads are query_face.rs's, read from there.

use crate::common;
use crate::query_face::{json_face, query_face, run, seeded, values};
use codeeraser::faces;
use serde_json::{Value, json};
use std::path::Path;

/// Two assertions: one every tree with a dead file violates, one none does.
const RULES: &str = "assert no_dead(F) :- dead(F).\nassert huge(F, N) :- lines(F, N), N > 1000.\n";

/// The second assertion alone: a rules file this tree passes.
fn clean() -> &'static str {
    RULES
        .split_once('\n')
        .map(|(_, rest)| rest)
        .expect("two rules")
}

/// The rules document the library face answers (the road every face
/// takes), with the rules file and the derivation switch.
fn rules_face(dir: &Path, file: Option<&Path>, why: bool) -> Value {
    let core = common::core_bin();
    faces::doc(faces::rules(dir, &core, file, why, Default::default())).expect("rules face")
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
    assert_eq!(doc, rules_face(&dir, None, false));
    assert_eq!(doc["schema"], common::stub_core::catalogue_schema("rules"));
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
        rules_face(&dir, None, false).to_string()
    );
    let filed = s.call(3, "rules", json!({"file": "ce.rules", "why": true}));
    assert_eq!(
        filed["content"][0]["text"],
        rules_face(&dir, Some(Path::new("ce.rules")), true).to_string()
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
