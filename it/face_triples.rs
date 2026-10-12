//! Three faces on real trees (plan v2.33 step 10, item 6): for every
//! report family, with every document option set at least once, the
//! CLI's `--format json`, the MCP tool over `ce mcp` and the library
//! face the Tauri command passes through to (face_parity_roads.rs) are
//! asked the same thing on the same tree, and the three documents must
//! be the same JSON. Ignored: it judges whole trees. The roots come in
//! `CE_TRIPLES_ROOTS` (`;`-separated; each a git work tree the run may
//! index); every pair is printed, and any difference fails the leg.

use crate::common::{McpSession, core_bin, run_ce};
use crate::face_triples_asks::{Ask, Picks, all};
use codeeraser::faces::{self, Plumb, doc};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The options that name a place, read off the root: a line inside a
/// unit, that unit's file spelled `./x` (the arch cleanup), a key one
/// unit alone holds, and the rules file when the root has one.
fn picks(root: &Path, core: &str) -> Picks {
    let scan = doc(faces::scan(root, core)).expect("scan");
    let (path, line) = scan["files"]
        .as_array()
        .expect("files")
        .iter()
        .find_map(|f| {
            let fun = f["functions"].as_array()?.first()?;
            Some((f["path"].as_str()?.to_string(), fun["start_line"].as_u64()?))
        })
        .expect("a file with a function");
    let units = doc(faces::clone(root, core, true, Plumb::default())).expect("units");
    let keys = crate::common::listed(&units, "units", "key");
    let unit = keys
        .iter()
        .find(|k| !k.starts_with('(') && keys.iter().filter(|o| o == k).count() == 1)
        .map(|k| k.to_string());
    let rules = root
        .join("ce.rules")
        .exists()
        .then(|| "ce.rules".to_string());
    Picks {
        at: format!("{path}:{}", line + 1),
        impact: format!("./{path}"),
        unit,
        rules,
    }
}

/// The three answers to one ask, each as compact JSON (or what failed).
fn answers(root: &Path, core: &str, mcp: &mut McpSession, id: u64, a: &Ask) -> [String; 3] {
    let mut words: Vec<&str> = a.cli.iter().map(String::as_str).collect();
    words.extend(["--format", "json", "."]);
    let out = run_ce(root, &words);
    let cli = serde_json::from_slice::<Value>(&out.stdout).map_or_else(
        |e| format!("CLI rc={:?} unparsed: {e}", out.status.code()),
        |v| v.to_string(),
    );
    let got = mcp.call(id, a.tool, a.args.clone());
    let text = got["content"][0]["text"].as_str().unwrap_or("");
    let machine = serde_json::from_str::<Value>(text)
        .map_or_else(|_| format!("MCP: {got}"), |v| v.to_string());
    let lib = (a.face)(root, core).map_or_else(|e| format!("face: {e:#}"), |v| v.to_string());
    [cli, machine, lib]
}

/// Every ask on one root: the pairs that agree, and each that does not.
/// Each ask runs once through the library first: the T3 judgment cache
/// counts which run filled it (`judged` / `cached` / `requests` in the
/// clone document), so the first face asked on a cold root would read
/// differently from the two after it by run state, not by face.
fn judged(root: &Path, core: &str, differ: &mut Vec<String>) -> usize {
    let asks = all(&picks(root, core));
    for a in &asks {
        let _warm = (a.face)(root, core);
    }
    let mut mcp = McpSession::over(root);
    let mut same = 0;
    for (i, a) in asks.iter().enumerate() {
        let [cli, machine, lib] = answers(root, core, &mut mcp, 100 + i as u64, a);
        let label = format!("{} :: ce {}", root.display(), a.cli.join(" "));
        let pairs = [
            ("CLI=MCP", cli == machine),
            ("MCP=face", machine == lib),
            ("CLI=face", cli == lib),
        ];
        let flags = pairs.map(|(n, ok)| format!("{n} {ok}"));
        println!("{label}: {} B; {}", cli.len(), flags.join(", "));
        same += pairs.iter().filter(|(_, ok)| *ok).count();
        if pairs.iter().any(|(_, ok)| !ok) {
            let at = first_difference(&cli, &machine).min(first_difference(&cli, &lib));
            let [c, m, l] = [&cli, &machine, &lib].map(|s| near(s, at));
            let bad: Vec<&str> = pairs
                .iter()
                .filter(|(_, ok)| !ok)
                .map(|(n, _)| *n)
                .collect();
            differ.push(format!(
                "{label}: {bad:?} at byte {at}\n  CLI {c}\n  MCP {m}\n  lib {l}"
            ));
        }
    }
    mcp.finish();
    same
}

#[test]
#[ignore = "judges whole trees: CE_TRIPLES_ROOTS=<root>;<root>"]
fn every_family_answers_the_same_document_on_all_three_faces() {
    let roots = std::env::var("CE_TRIPLES_ROOTS").expect("CE_TRIPLES_ROOTS");
    let core = core_bin();
    let mut differ = Vec::new();
    let mut same = 0;
    for root in roots.split(';').filter(|r| !r.is_empty()) {
        same += judged(&PathBuf::from(root), &core, &mut differ);
    }
    println!("pairs identical: {same}; differing: {}", differ.len());
    assert!(differ.is_empty(), "{}", differ.join("\n"));
}

fn first_difference(a: &str, b: &str) -> usize {
    a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count()
}

/// The text around byte `at`, to show where two documents part.
fn near(s: &str, at: usize) -> &str {
    let from = s.floor_char_boundary(at.saturating_sub(120));
    &s[from..s.floor_char_boundary(at + 200)]
}
