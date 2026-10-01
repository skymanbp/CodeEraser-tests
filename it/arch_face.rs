//! The architecture family's three faces (plan v2.31 step 9, design
//! booklet §7.3) agree on ONE document: `ce arch --format json` prints
//! the library face byte for byte, the MCP tool `architecture` relays
//! it, and the console reads the same rows. The fixture is built so
//! both cut roads are live: three directories in one cycle (the core
//! searches it exactly) and fifteen in one ring (one past the exact
//! bound, so the cut comes from the greedy order), plus a Go command
//! importing a sibling package — the one package reference.

use crate::common;
use codeeraser::arch::face::SCHEMA_ID;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The ring past CE.Arch.Cost.exactVertexCap (14).
const RING: usize = 15;

/// The hand-written half of the tree; the ring is generated.
const DOC: &str = "\
--- a/x.py
import b.y
--- b/y.py
import c.z
--- c/z.py
import a.x
--- go.mod
module ex

go 1.21
--- g/main.go
package main

import \"ex/h\"

func main() { h.H() }
--- h/h.go
package h

func H() {}
";

fn seeded(name: &str) -> PathBuf {
    let ring: String = (0..RING)
        .map(|i| format!("--- r{i:02}/m.py\nimport r{:02}.m\n", (i + 1) % RING))
        .collect();
    let dir = common::doc_tree(name, &format!("{DOC}{ring}"));
    common::init_and_commit(&dir, "seed");
    dir
}

/// `ce arch . --core <core> [--format json] <extra…>`: (exit, stdout, stderr).
fn arch(dir: &Path, core: &str, json: bool, extra: &[&str]) -> (Option<i32>, String, String) {
    let mut args = vec!["arch", ".", "--core", core];
    if json {
        args.extend(["--format", "json"]);
    }
    args.extend(extra);
    common::ce_triple(dir, &args, &[])
}

/// The JSON document off one run, and the exit code beside it.
fn doc(dir: &Path, core: &str, extra: &[&str]) -> (Option<i32>, Value) {
    let (code, out, err) = arch(dir, core, true, extra);
    match serde_json::from_str(&out) {
        Ok(d) => (code, d),
        Err(e) => panic!("{e}\n{out}\n{err}"),
    }
}

/// A seeded fixture judged by the real core with `--impact` per path:
/// exit 0, and the CLI's document is the library face's.
fn judged(name: &str, impact: &[&str]) -> (PathBuf, Value) {
    let dir = seeded(name);
    let extra: Vec<&str> = impact.iter().flat_map(|p| ["--impact", p]).collect();
    let (code, d) = doc(&dir, &common::core_bin(), &extra);
    assert_eq!((code, &d), (Some(0), &face(&dir, impact)));
    (dir, d)
}

fn face(dir: &Path, impact: &[&str]) -> Value {
    let impact: Vec<String> = impact.iter().map(|s| s.to_string()).collect();
    codeeraser::faces::arch(dir, &common::core_bin(), &impact).expect("the arch face")
}

/// Every directory arc the fixture spells, by construction.
fn arcs() -> Vec<(String, String)> {
    let mut arcs: Vec<(String, String)> = [("a", "b"), ("b", "c"), ("c", "a"), ("g", "h")]
        .iter()
        .map(|(x, y)| (x.to_string(), y.to_string()))
        .collect();
    arcs.extend((0..RING).map(|i| (format!("r{i:02}"), format!("r{:02}", (i + 1) % RING))));
    arcs
}

fn rows<'a>(d: &'a Value, key: &str) -> &'a Vec<Value> {
    d[key].as_array().unwrap_or_else(|| panic!("{key}: {d}"))
}

fn text(v: &Value, key: &str) -> String {
    v[key]
        .as_str()
        .unwrap_or_else(|| panic!("{key}: {v}"))
        .to_string()
}

/// (from, to, exact) per cut arc.
fn cut_arcs(d: &Value) -> Vec<(String, String, bool)> {
    rows(d, "cuts")
        .iter()
        .map(|c| (text(c, "from"), text(c, "to"), c["exact"] == true))
        .collect()
}

#[test]
fn the_cli_prints_the_library_face_and_both_cut_roads_are_live() {
    let (_, d) = judged("arch-cli", &[]);
    assert_eq!(
        (d["schema"].as_str(), d["degraded"].is_null()),
        (Some(SCHEMA_ID), true)
    );
    let counts = (&d["counts"]["pkgEdges"], &d["counts"]["cuts"]);
    assert_eq!(counts, (&json!(1), &json!(2)), "{}", d["counts"]);
    let cuts = cut_arcs(&d);
    let exact = cuts.iter().filter(|c| c.2).count();
    assert_eq!((exact, cuts.len() - exact), (1, 1), "{cuts:?}");
    for (from, to, exact) in &cuts {
        // the ring is greedy, the triangle exact
        assert_eq!(from.starts_with('r'), !exact, "{cuts:?}");
        assert!(
            arcs().contains(&(from.clone(), to.clone())),
            "{from} → {to}"
        );
    }
    assert!(
        rows(&d, "cuts")
            .iter()
            .all(|c| !rows(c, "files").is_empty())
    );
    // with the cut arcs gone, every arc runs from a higher level down
    let level: BTreeMap<String, i64> = rows(&d, "layers")
        .iter()
        .map(|l| (text(l, "dir"), l["level"].as_i64().expect("a level")))
        .collect();
    let kept = arcs()
        .into_iter()
        .filter(|(f, t)| !cuts.iter().any(|c| (&c.0, &c.1) == (f, t)));
    for (from, to) in kept {
        assert!(level[&from] > level[&to], "{from} → {to}: {level:?}");
    }
    // the package reference is the one arc between g and h
    let metric = |dir: &str| {
        let m = rows(&d, "metrics")
            .iter()
            .find(|m| m["dir"] == dir)
            .expect(dir);
        (m["fanIn"].as_i64(), m["fanOut"].as_i64())
    };
    assert_eq!(
        (metric("g"), metric("h")),
        ((Some(0), Some(1)), (Some(1), Some(0)))
    );
}

#[test]
fn the_console_reads_the_same_rows() {
    let dir = seeded("arch-console");
    let (_, d) = doc(&dir, &common::core_bin(), &[]);
    let (code, printed, _) = arch(&dir, &common::core_bin(), false, &[]);
    assert_eq!(code, Some(0));
    let first = printed.lines().next().unwrap_or_default();
    assert!(
        first.starts_with(&format!("{} files, ", d["counts"]["files"])),
        "{printed}"
    );
    // every cut prints the file references under its arc
    let under: Vec<String> = rows(&d, "cuts")
        .iter()
        .flat_map(|c| rows(c, "files").clone())
        .map(|f| {
            format!(
                "  {} → {}  ({})",
                text(&f, "from"),
                text(&f, "to"),
                f["refs"]
            )
        })
        .collect();
    let words = [
        "exact)",
        "greedy)",
        "no misplaced file",
        "dir  fanIn  fanOut  instability",
    ];
    for word in under.iter().map(String::as_str).chain(words) {
        assert!(printed.contains(word), "{word}: {printed}");
    }
    assert!(
        !printed.contains("impact (depth):"),
        "no focus, no impact section: {printed}"
    );
}

#[test]
fn impact_walks_the_referrers_and_a_stranger_is_refused() {
    let (dir, d) = judged("arch-impact", &["a/x.py"]);
    let depth: Vec<(String, Value)> = rows(&d, "impact")
        .iter()
        .map(|i| (text(i, "path"), i["depth"].clone()))
        .collect();
    let want = [("a/x.py", 0), ("b/y.py", 2), ("c/z.py", 1)];
    assert_eq!(depth, want.map(|(p, n)| (p.to_string(), json!(n))).to_vec());
    assert_eq!(d["counts"]["focus"], 1);
    let (_, text, _) = arch(&dir, &common::core_bin(), false, &["--impact", "a/x.py"]);
    assert!(
        text.contains("impact (depth):\n  a/x.py  0\n  b/y.py  2\n  c/z.py  1\n"),
        "{text}"
    );
    let (code, out, err) = arch(&dir, &common::core_bin(), true, &["--impact", "nope.py"]);
    let named = err.contains("--impact: nope.py is not a measured file");
    assert_eq!((code, out.as_str(), named), (Some(2), "", true), "{err}");
}

#[test]
fn the_mcp_tool_relays_the_face_and_refuses_a_path_list_that_is_not_one() {
    // `judged` holds the CLI's document equal to the library face's;
    // the relay equal to it makes the three faces one document.
    let (dir, d) = judged("arch-mcp", &["c/z.py"]);
    let mut s = common::McpSession::over(&dir);
    let focused = s.relayed(1, "architecture", json!({"impact": ["c/z.py"]}));
    assert_eq!(focused, d.to_string());
    let plain = s.relayed(2, "architecture", json!({}));
    assert_eq!(plain, face(&dir, &[]).to_string());
    let bad = s.call(3, "architecture", json!({"impact": "c/z.py"}));
    assert_eq!(bad["isError"], true, "{bad}");
    s.finish();
}

/// Two cores that cannot judge, one document shape: a stub that
/// shakes hands without the family (named absent, never read as an
/// empty architecture) and a path where no core exists (named, not an
/// error; the library face's document, since the CLI naming that path
/// as its `--core` is refused before any judgment). Exit 2, the reason
/// carried, every table empty, every count 0.
#[test]
fn a_core_without_the_family_is_a_named_degraded_document() {
    let dir = seeded("arch-degraded");
    let stub = common::stub_core::hello_only();
    let missing = common::tmp("arch-no-core").join("no-such-core.exe");
    let missing = missing.to_string_lossy().into_owned();
    common::refused_for_its_core(&dir, &["arch", ".", "--core", &missing], &missing);
    let (code, by_cli) = doc(&dir, &stub, &[]);
    assert_eq!(code, Some(2), "{by_cli}");
    let by_face = codeeraser::faces::arch(&dir, &missing, &[]).expect("face");
    for (d, reason) in [(&by_cli, "core offers no arch/1"), (&by_face, "")] {
        let why = d["degraded"].as_str().unwrap_or_default();
        assert!(!why.is_empty() && why.contains(reason), "{d}");
        for table in "layers cuts clusters misplaced impact metrics".split(' ') {
            assert_eq!(d[table], json!([]), "{table}");
        }
        assert!(
            d["counts"].as_object().unwrap().values().all(|n| n == 0),
            "{d}"
        );
    }
    let (code, text, _) = arch(&dir, &stub, false, &[]);
    assert_eq!(code, Some(2));
    let why = by_cli["degraded"].as_str().unwrap_or_default();
    assert_eq!(text, format!("arch: degraded — {why}\n"));
}
