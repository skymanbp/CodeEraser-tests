//! The clone-merge family's three faces (plan v2.31 step 7, design
//! booklet §6.4) agree on ONE document: `ce merge --format json` prints
//! the library face byte for byte and the MCP tool `merge_suggestions`
//! relays it. The fixture holds the three group shapes the family
//! reads: three whole functions alike but for three renamed names (a
//! T1/T2 family of whole units), a pair one statement apart (a T3 pair,
//! the statement a gap hole) and two functions sharing a run of
//! statements inside them (a T1/T2 fragment family).

use crate::common;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The fixture tree: a file's path on a `--- ` line, its text under it.
const FIXTURE: &str = "\
--- a.py
def total_price(items, rate):
    subtotal = 0
    for item in items:
        if item.count > 0 and item.active:
            subtotal = subtotal + item.price * item.count - item.discount
    taxed = subtotal * rate + len(items)
    return round(taxed, 2)


def total_weight(parcels, factor):
    subtotal = 0
    for item in parcels:
        if item.count > 0 and item.active:
            subtotal = subtotal + item.price * item.count - item.discount
    taxed = subtotal * factor + len(parcels)
    return round(taxed, 2)


def total_volume(boxes, scale):
    subtotal = 0
    for item in boxes:
        if item.count > 0 and item.active:
            subtotal = subtotal + item.price * item.count - item.discount
    taxed = subtotal * scale + len(boxes)
    return round(taxed, 2)
--- b.py
def summarize(records, limit):
    seen = set()
    out = []
    for record in records:
        key = record.name.lower()
        if key in seen:
            continue
        seen.add(key)
        out.append(record.value * 2 + limit)
    return sorted(out)


def summarize_all(records, limit):
    seen = set()
    out = []
    for record in records:
        key = record.name.lower()
        if key in seen:
            continue
        seen.add(key)
        print(key)
        out.append(record.value * 2 + limit)
    return sorted(out)
--- c.py
def load(path, mode):
    header = read_header(path)
    rows = []
    for line in open(path, mode):
        parts = line.strip().split(\",\")
        if len(parts) > 3 and parts[0] != \"#\":
            rows.append((parts[0], int(parts[1]), float(parts[2])))
    return header, rows


def reload(source, flags):
    count = 0
    rows = []
    for line in open(source, flags):
        parts = line.strip().split(\",\")
        if len(parts) > 3 and parts[0] != \"#\":
            rows.append((parts[0], int(parts[1]), float(parts[2])))
    count = len(rows)
    return count
";

fn seeded(name: &str) -> PathBuf {
    common::seeded_tree(name, FIXTURE)
}

/// `ce merge . --core <core> <args…>`: (exit, stdout, stderr).
fn run(dir: &Path, core: &str, args: &[&str]) -> (Option<i32>, String, String) {
    let mut full = vec!["merge", ".", "--core", core];
    full.extend_from_slice(args);
    common::ce_triple(dir, &full, &[])
}

fn group<'d>(doc: &'d Value, family: &str, fragment: bool) -> &'d Value {
    doc["groups"]
        .as_array()
        .expect("groups")
        .iter()
        .find(|g| g["family"] == family && g["fragment"] == fragment)
        .unwrap_or_else(|| panic!("no {family} group (fragment {fragment}): {doc}"))
}

fn texts(param: &Value, key: &str) -> Vec<String> {
    param["values"]
        .as_array()
        .expect("values")
        .iter()
        .map(|v| v[key].as_str().expect(key).to_string())
        .collect()
}

#[test]
fn the_cli_prints_the_library_face_and_every_group_shape_is_judged() {
    let dir = seeded("merge-cli");
    let core = common::core_bin();
    let (code, out, err) = run(&dir, &core, &["--format", "json"]);
    assert_eq!(code, Some(0), "{err}");
    let doc: Value = serde_json::from_str(&out).expect("json");
    assert_eq!(doc, codeeraser::faces::merge(&dir, &core).expect("face"));
    assert_eq!(doc["schema"], codeeraser::merge::face::SCHEMA_ID);
    assert!(doc["degraded"].is_null(), "{doc}");
    let whole = group(&doc, "t1t2", false);
    assert_eq!(
        (whole["feasible"].clone(), whole["params"].clone()),
        (true.into(), 3.into())
    );
    let names: Vec<Vec<String>> = whole["holes"]
        .as_array()
        .expect("holes")
        .iter()
        .map(|p| texts(p, "text"))
        .collect();
    let want = [
        ["total_price", "total_weight", "total_volume"],
        ["items", "parcels", "boxes"],
        ["rate", "factor", "scale"],
    ];
    assert_eq!(names, want, "one parameter per renamed name");
    let pair = doc["groups"]
        .as_array()
        .expect("groups")
        .iter()
        .find(|g| g["members"][0]["path"] == "b.py")
        .expect("the b.py pair");
    assert_eq!(
        (pair["family"].as_str(), pair["reason"].as_str()),
        (Some("t3"), Some("spans_statements"))
    );
    let gap = &pair["holes"].as_array().expect("holes")[1];
    assert_eq!(
        texts(gap, "text"),
        ["", "print(key)"],
        "the extra statement"
    );
    runs_sit_in_their_spans(&doc);
}

/// A fragment member has no unit and sends a run inside its span; a
/// whole unit's run is its span.
fn runs_sit_in_their_spans(doc: &Value) {
    let fragment = group(doc, "t1t2", true);
    assert_eq!(fragment["members"][0]["unit"], Value::Null);
    for m in fragment["members"].as_array().expect("members") {
        let (at, run) = (&m["lines"], &m["run"]);
        assert!(
            at[0].as_u64() <= run[0].as_u64() && run[1].as_u64() <= at[1].as_u64(),
            "{m}"
        );
    }
    let whole = &group(doc, "t1t2", false)["members"][0];
    assert_eq!(whole["run"], whole["lines"]);
}

/// `--group` prints one group; a number past the document is an
/// argument error; `ce clone` over the same fixture answers what it
/// answered before merge/1's columns rode its walk.
#[test]
fn one_group_prints_alone_and_the_clone_report_is_unmoved() {
    let dir = seeded("merge-group");
    let core = common::core_bin();
    let (code, out, _) = run(&dir, &core, &["--group", "1"]);
    assert_eq!(code, Some(0));
    let heads: Vec<&str> = out.lines().filter(|l| l.starts_with("group ")).collect();
    assert_eq!(heads.len(), 1, "{out}");
    assert!(heads[0].starts_with("group 1:"), "{out}");
    let (code, _, err) = run(&dir, &core, &["--group", "99"]);
    assert_eq!(code, Some(2));
    assert!(err.contains("--group 99"), "{err}");
    let clone = codeeraser::faces::clone_t3(&dir, &core).expect("clone");
    let hits: Vec<String> = clone["clones"]
        .as_array()
        .expect("clones")
        .iter()
        .map(|h| format!("{} {} {} {}/{}", h["a"], h["b"], h["ted"], h["n1"], h["n2"]))
        .collect();
    let want = [
        "\"a.py:total_price/2#0\" \"a.py:total_volume/2#0\" 0 58/58",
        "\"a.py:total_price/2#0\" \"a.py:total_weight/2#0\" 0 58/58",
        "\"a.py:total_volume/2#0\" \"a.py:total_weight/2#0\" 0 58/58",
        "\"b.py:summarize/2#0\" \"b.py:summarize_all/2#0\" 5 61/66",
        "\"c.py:load/2#0\" \"c.py:reload/2#0\" 11 84/86",
    ];
    assert_eq!(hits, want);
}

/// The MCP tool relays the face; a core that cannot be reached is a
/// named degraded document and exit 2.
#[test]
fn the_mcp_tool_relays_the_face_and_an_absent_core_degrades() {
    let dir = seeded("merge-mcp");
    let core = common::core_bin();
    let mut s = common::McpSession::over(&dir);
    let got = s.call(1, "merge_suggestions", serde_json::json!({}));
    assert_eq!(got["isError"], false, "{got}");
    assert_eq!(
        got["content"][0]["text"],
        codeeraser::faces::merge(&dir, &core)
            .expect("face")
            .to_string()
    );
    s.finish();
    let absent = dir.join("no-such-core.exe");
    let absent = absent.to_str().expect("utf-8");
    let (code, out, _) = run(&dir, absent, &["--format", "json"]);
    assert_eq!(code, Some(2));
    let doc: Value = serde_json::from_str(&out).expect("json");
    assert!(doc["degraded"].is_string(), "{doc}");
    assert_eq!(doc["groups"], serde_json::json!([]));
}
