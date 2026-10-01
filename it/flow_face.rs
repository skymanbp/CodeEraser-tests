//! The flow family's three faces (plan v2.31 step 5, design booklet
//! §5.4) agree on ONE document: `ce flow --format json` prints the
//! library face byte for byte, the MCP tool `flow` relays it, `--kind`
//! narrows the listing and never the counts, and `--check` is its own
//! gate — exit 1 only at `[flow] tier = "deny"` with a judged finding,
//! which `flow::judged_mask()` decides. The fixture holds one finding
//! of three kinds: an unreachable statement, a dead store, an unused
//! parameter (advisory in every language).

use crate::common;
use codeeraser::flow_report::face::SCHEMA_ID;
use codeeraser::scan::lang::Lang;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const PY: &str = "\
def gone():
    return 1
    print(\"never\")


def overwritten():
    x = 1
    x = 2
    return x


def ignores(a):
    return 3
";

fn seeded(name: &str) -> PathBuf {
    let dir = common::tmp(name);
    std::fs::write(dir.join("k.py"), PY).expect("k.py");
    dir
}

/// `ce flow . --core <core> --format json <more…>`: the exit and the
/// document (a document that does not parse is the failure).
fn cli(dir: &Path, more: &[&str], core: &str) -> (Option<i32>, Value) {
    let mut args = vec!["flow", ".", "--core", core, "--format", "json"];
    args.extend_from_slice(more);
    let (code, out, err) = common::ce_triple(dir, &args, &[]);
    let doc = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{more:?}: {e}\n{out}\n{err}"));
    (code, doc)
}

/// A run refused before any judgment: exit 2, its reason naming `needle`.
fn refused(dir: &Path, args: &[&str], needle: &str) {
    let (code, _, err) = common::ce_triple(dir, args, &[]);
    assert_eq!((code, err.contains(needle)), (Some(2), true), "{err}");
}

fn face(dir: &Path, kinds: &[String]) -> Value {
    codeeraser::faces::flow(dir, &common::core_bin(), kinds).expect("face")
}

/// The finding rows as `(kind, line, var)`.
fn rows(doc: &Value) -> Vec<(String, u64, Value)> {
    doc["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .map(|f| {
            (
                f["kind"].as_str().unwrap().into(),
                f["line"].as_u64().unwrap(),
                f["var"].clone(),
            )
        })
        .collect()
}

#[test]
fn the_cli_prints_the_library_face_and_kind_narrows_only_the_listing() {
    let dir = seeded("flow-face-cli");
    let core = common::core_bin();
    let (code, doc) = cli(&dir, &[], &core);
    assert_eq!(code, Some(0));
    assert_eq!(doc, face(&dir, &[]));
    assert_eq!(doc["schema"], SCHEMA_ID);
    assert!(doc["degraded"].is_null(), "{doc}");
    assert_eq!(
        rows(&doc),
        [
            ("unreachable".into(), 3, Value::Null),
            ("dead_store".into(), 7, json!("x")),
            ("unused_param".into(), 12, json!("a")),
        ]
    );
    assert_eq!(
        doc["findings"][2]["judged"], false,
        "a parameter is advisory"
    );
    let (code, one) = cli(&dir, &["--kind", "unreachable"], &core);
    assert_eq!(code, Some(0));
    assert_eq!(one, face(&dir, &["unreachable".into()]));
    assert_eq!(rows(&one).len(), 1);
    assert_eq!(
        (&one["counts"]["findings"], &one["counts"]["shown"]),
        (&json!(3), &json!(1))
    );
    refused(&dir, &["flow", ".", "--kind", "dead"], "unknown kind");
}

/// `--check` reads `[flow] tier` and the judged count: deny with a
/// judged finding exits 1; with the language outside the mask every
/// finding is advisory and the gate passes. Both roads are written,
/// the mask picks the live one.
#[test]
fn check_is_the_classs_own_gate_at_deny() {
    let dir = seeded("flow-face-check");
    let core = common::core_bin();
    let (code, _) = cli(&dir, &["--check"], &core);
    assert_eq!(code, Some(0), "observe never fails the gate");
    common::declare(&dir, "[flow]\ntier = \"deny\"\n");
    let (code, doc) = cli(&dir, &["--check"], &core);
    let judged = codeeraser::flow_report::lang_judged(Lang::Python);
    let expected = if judged { (Some(1), 2) } else { (Some(0), 0) };
    assert_eq!(
        (code, doc["counts"]["judged"].as_u64().unwrap()),
        expected,
        "{doc}"
    );
    let (code, _) = cli(&dir, &["--check", "--kind", "unused_param"], &core);
    assert_eq!(code, expected.0, "the filter never moves the gate");
    // a tier outside the four is refused at the load throat, by name
    common::declare(&dir, "[flow]\ntier = \"Deny\"\n");
    refused(&dir, &["flow", "."], "[flow] tier");
}

#[test]
fn the_mcp_tool_relays_the_same_document() {
    let dir = seeded("flow-face-mcp");
    let mut s = common::McpSession::over(&dir);
    let got = s.call(1, "flow", json!({}));
    assert_eq!(got["isError"], false, "{got}");
    assert_eq!(got["content"][0]["text"], face(&dir, &[]).to_string());
    let one = s.call(2, "flow", json!({"kind": "dead_store"}));
    let text = one["content"][0]["text"].as_str().expect("text");
    assert_eq!(text, face(&dir, &["dead_store".into()]).to_string());
    s.finish();
}

/// No core to lay the document out: the CLI naming that path as its
/// `--core` is refused before any judgment, the library face by name —
/// no document either way.
#[test]
fn no_core_is_a_named_refusal_and_exit_2() {
    let dir = seeded("flow-face-degraded");
    let missing = dir.join("no-such-core").display().to_string();
    common::refused_for_its_core(&dir, &["flow", ".", "--core", &missing], &missing);
    let err = codeeraser::faces::flow(&dir, &missing, &[]).expect_err("no core");
    assert!(err.to_string().contains("flow document:"), "{err:#}");
}
