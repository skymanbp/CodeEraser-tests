//! The built-in rule pack (plan v2.33, `[rules.pack]`) through the
//! query family's three faces: the core renders the declared items to
//! CE Datalog and judges them beside the rules file, so `ce rules
//! --format json`, the library face the GUI's Query screen calls
//! (`faces::rules`) and the MCP `rules` tool must agree on one document
//! whose assertions are the pack's `pack_…` heads. The fixture is built
//! to break three items and pass two: `app/` and `lib/` read each other,
//! `lib/util.py` has two readers, `lib/` (the lower layer) reads `app/`,
//! nothing reads more than one file, and `tools/` never reads `app/`.

use crate::common;
use codeeraser::query::PRELUDE;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const FIXTURE: &str = "\
--- app/main.py
import lib.util
--- app/helpers.py
x = 1
--- lib/util.py
import app.helpers
--- tools/one.py
import lib.util
";

const PACK: &str = r#"[rules.pack]
no_dir_cycles = true
fan_in_max = 1
fan_out_max = 1
layers = ["app/**", "lib/**"]
forbid = [["tools/**", "app/**"]]
"#;

fn seeded(name: &str, pack: &str) -> PathBuf {
    let dir = common::seeded_tree(name, FIXTURE);
    common::declare(&dir, pack);
    dir
}

/// `ce <args…> . --core <core>`: (exit, stdout, stderr), the query
/// family's own runner.
fn run(dir: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    crate::query_face::run(dir, args, false)
}

fn json_rules(dir: &Path) -> (Option<i32>, Value) {
    let (code, out, err) = run(dir, &["rules", "--format", "json"]);
    let doc = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}\n{out}\n{err}"));
    (code, doc)
}

fn library(dir: &Path) -> Value {
    codeeraser::faces::doc(codeeraser::faces::rules(
        dir,
        &common::core_bin(),
        None,
        false,
        Default::default(),
    ))
    .expect("the rules face")
}

/// (goal name, the answers' values) per goal, in goal order.
fn verdicts(doc: &Value) -> Vec<(String, Vec<Vec<String>>)> {
    let goals = doc["goals"].as_array().expect("goals");
    goals
        .iter()
        .map(|g| {
            let rows = doc["answers"]
                .as_array()
                .expect("answers")
                .iter()
                .filter(|a| a["goal"] == g["goal"])
                .map(|a| serde_json::from_value(a["values"].clone()).expect("values"))
                .collect();
            (g["name"].as_str().unwrap_or("?-").to_string(), rows)
        })
        .collect()
}

#[test]
fn the_three_faces_judge_the_pack_as_one_document() {
    let dir = seeded("rules-pack-faces", PACK);
    let (code, doc) = json_rules(&dir);
    assert_eq!(code, Some(1), "{doc}");
    assert_eq!(doc, library(&dir), "the CLI prints the GUI's document");
    let mut s = common::McpSession::over(&dir);
    let mcp = s.call(1, "rules", json!({}));
    assert_eq!(mcp["content"][0]["text"], library(&dir).to_string());
    s.finish();
    let pair = |a: &str, b: &str| vec![a.to_string(), b.to_string()];
    assert_eq!(
        verdicts(&doc),
        [
            ("pack_no_dir_cycles".into(), vec![pair("app", "lib")]),
            ("pack_fan_in_max".into(), vec![pair("lib/util.py", "2")]),
            ("pack_fan_out_max".into(), vec![]),
            (
                "pack_layers_1_0".into(),
                vec![pair("lib/util.py", "app/helpers.py")]
            ),
            ("pack_forbid_0".into(), vec![]),
        ]
    );
    assert_eq!(doc["counts"]["violations"], 3);
    assert_eq!(
        (&doc["program"]["prelude"], &doc["program"]["rules_file"]),
        (&json!(8), &Value::Null)
    );
    // the console names the pack's assertions; no file, yet not "zero assertions"
    let (code, text, _) = run(&dir, &["rules"]);
    assert_eq!(code, Some(1));
    assert!(
        text.starts_with("assert pack_no_dir_cycles(D, E): 1 violation(s)\n  app  lib\n"),
        "{text}"
    );
    assert!(text.contains("assert pack_forbid_0(F, G): ok\n"), "{text}");
    // the prelude page shows what was judged
    let (code, page, _) = run(&dir, &["query", "--prelude"]);
    let rendered = page.strip_prefix(PRELUDE).expect("the prelude first");
    assert_eq!(code, Some(0));
    assert!(
        rendered.starts_with("# [rules.pack] no_dir_cycles = true\n"),
        "{rendered}"
    );
    assert!(rendered.contains("assert pack_forbid_0(F, G) :- in(F, \"tools/**\")"));
}

/// A question builds on the pack's rules as on the prelude's, and a
/// rules file may read them but not define one.
#[test]
fn the_pack_predicates_are_readable_and_reserved() {
    let dir = seeded("rules-pack-reserved", PACK);
    let (code, out, _) = run(
        &dir,
        &[
            "query",
            "pack_dir_reach(D, E), pack_dir_reach(E, D), D < E",
            "--format",
            "json",
        ],
    );
    let doc: Value = serde_json::from_str(&out).expect("a document");
    assert_eq!((code, &doc["errors"]), (Some(0), &json!([])), "{doc}");
    // the question is the last goal, after the pack's five assertions
    assert_eq!(
        verdicts(&doc).len(),
        6,
        "the pack's goals ride before the question"
    );
    let asked: Vec<&Value> = doc["answers"]
        .as_array()
        .expect("answers")
        .iter()
        .filter(|a| a["goal"] == 5)
        .collect();
    assert_eq!(asked.len(), 1, "{doc}");
    assert_eq!(asked[0]["values"], json!(["app", "lib"]));
    std::fs::write(
        dir.join("mine.rules"),
        "pack_dir_reach(D, E) :- dir(D), dir(E).\n",
    )
    .expect("mine.rules");
    let (code, out, _) = run(&dir, &["rules", "--file", "mine.rules", "--format", "json"]);
    let doc: Value = serde_json::from_str(&out).expect("a document");
    assert_eq!(code, Some(2));
    assert_eq!(doc["errors"][0]["at"], "rules 1:1", "{doc}");
    assert_eq!(doc["errors"][0]["what"], "prelude predicate redefined");
}

/// A glob the dialect cannot read is an error at its place in the pack;
/// a declaration the core cannot render is refused by name, exit 2.
#[test]
fn a_bad_pack_is_named_where_it_breaks() {
    let dir = seeded(
        "rules-pack-bad",
        "[rules.pack]\nforbid = [[\"[\", \"app/**\"]]\n",
    );
    let (code, doc) = json_rules(&dir);
    assert_eq!(code, Some(2));
    let at = doc["errors"][0]["at"].as_str().unwrap_or_default();
    assert!(at.starts_with("pack 2:"), "{doc}");
    let (_, text, _) = run(&dir, &["rules"]);
    assert!(text.starts_with("error at pack 2:"), "{text}");
    common::declare(&dir, "[rules.pack]\nlayers = [\"app/**\"]\n");
    let (code, doc) = json_rules(&dir);
    assert_eq!(code, Some(2));
    let why = doc["degraded"].as_str().unwrap_or_default();
    assert!(why.contains("a layering needs at least two"), "{doc}");
}
