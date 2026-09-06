//! The two bilingual diagrams (architecture, judgment data flow) are
//! archify IR under docs/diagrams/, rendered by scripts/diagram.mjs
//! from one pinned archify commit into docs/assets + site/assets. Four
//! things are held here (plan v2.21, architecture-diagram clause): the
//! cache is present AND at the pin, read by the renderer alone (its
//! `requireCache` exits 2 by name; a second `git rev-parse` here once
//! answered for the CodeEraser checkout under a hollow cache), so a
//! gate that cannot render refuses instead of passing; the committed
//! SVGs are byte-for-byte what this pin renders (CE_BLESS=1 rewrites
//! them locally); the two languages of one diagram share every byte of
//! geometry and differ only in text; every `sources` path an IR cites
//! exists in the tree.
//! And the zh twin speaks one language: archify writes some chrome of
//! its own that no IR key reaches, so the renderer carries a map for it
//! (`CHROME` in scripts/diagram_svg.mjs) and the fifth leg reads that
//! map back out to prove every term in it actually left the file —
//! anywhere in it: two of the terms are aria-labels, not text nodes.
//!
//! And the IR itself is hand-written: four of its sublabels spell
//! facts the registry owns (proto and the family count, the grammars,
//! the GUI screens, the MCP tools). A sixth leg renders those ids and
//! holds the committed IR to what they say, so a bump that moves one
//! cannot ship a stale picture quietly.

use crate::common::{expect_ok, node, repo_root};
use crate::facts::{self, blessing, read};
use serde_json::Value;
use std::path::Path;

const DIAGRAMS: &[&str] = &["architecture", "judgment"];
/// The keys a translation may change; everything else is geometry.
const TEXT: &[&str] = &[
    "label",
    "sublabel",
    "tag",
    "title",
    "subtitle",
    "classification",
    "note",
    "items",
];

fn geometry(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(k, _)| !TEXT.contains(&k.as_str()))
                .map(|(k, v)| (k.clone(), geometry(v)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(geometry).collect()),
        other => other.clone(),
    }
}

fn cited_paths(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::Array(sources)) = map.get("sources") {
                out.extend(
                    sources
                        .iter()
                        .filter_map(|s| s["path"].as_str().map(str::to_string)),
                );
            }
            map.values().for_each(|v| cited_paths(v, out));
        }
        Value::Array(items) => items.iter().for_each(|v| cited_paths(v, out)),
        _ => {}
    }
}

fn ir(root: &Path, name: &str, lang: &str) -> Value {
    let rel = format!("docs/diagrams/{name}.{lang}.json");
    serde_json::from_str(&read(root, &rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

#[test]
fn the_committed_diagrams_are_what_the_pinned_archify_renders() {
    let mode = if blessing() { "--write" } else { "--check" };
    expect_ok(
        &node(&["scripts/diagram.mjs", mode], &[]),
        "diagrams drifted or failed to render",
    );
}

#[test]
fn the_two_languages_share_one_geometry_and_cite_real_paths() {
    let root = repo_root();
    for name in DIAGRAMS {
        let (en, zh) = (ir(&root, name, "en"), ir(&root, name, "zh"));
        assert_eq!(
            geometry(&en),
            geometry(&zh),
            "{name}: the en and zh IR differ outside their text"
        );
        let mut paths = Vec::new();
        cited_paths(&en, &mut paths);
        cited_paths(&zh, &mut paths);
        // `sources` is an architecture-schema field; the data-flow schema
        // has none, so only the architecture diagram owes evidence
        assert!(
            !paths.is_empty() || *name != "architecture",
            "{name}: no component cites a source"
        );
        for path in paths {
            assert!(
                root.join(&path).exists(),
                "{name}: cited source {path} is not in the tree"
            );
        }
    }
}

/// The `zh` terms of the renderer's chrome map, read from its source:
/// the gate must not carry a second copy of a list whose whole job is
/// to be applied.
fn chrome_terms(root: &Path) -> Vec<String> {
    let js = read(root, "scripts/diagram_svg.mjs");
    let table = js
        .split_once("const CHROME = { zh: {")
        .expect("diagram_svg.mjs declares CHROME.zh")
        .1
        .split_once('}')
        .expect("the zh table closes")
        .0;
    table
        .split(',')
        .filter_map(|pair| pair.split_once(':'))
        .map(|(key, _)| key.trim().trim_matches('"').to_string())
        .filter(|key| !key.is_empty())
        .collect()
}

/// The architecture sublabels that spell a derived fact: `node|en|zh`,
/// one row per component, `{id}` rendered through the registry
/// (facts::template — the same channel the source-literal sites use).
/// One literal rather than a table of same-shaped tuples: such a table
/// is a clone of every other one by construction (facts_chips.rs).
const SPELLED: &str = "
scan|tree-sitter · {count:grammars#word} grammars|tree-sitter · {count:grammars#word}套语法
wire|proto {ver:proto#v} · {count:families#word} families|proto {ver:proto#v} · {count:families#word}个家族
gui|Tauri · {count:screens#word} screens|Tauri · {count:screens#word}屏
mcp|{count:mcp_tools#word} read-only tools|{count:mcp_tools#word}个只读工具
";

/// The `sublabel` of one architecture component, by id.
fn sublabel(ir: &Value, id: &str) -> String {
    ir["components"]
        .as_array()
        .expect("the architecture IR carries components")
        .iter()
        .find(|c| c["id"] == id)
        .unwrap_or_else(|| panic!("the architecture IR has no component {id:?}"))["sublabel"]
        .as_str()
        .expect("a component sublabel")
        .to_string()
}

#[test]
fn the_architecture_sublabels_spell_the_registry_values() {
    let root = repo_root();
    for row in SPELLED.lines().filter(|l| !l.is_empty()) {
        let (id, rest) = row.split_once('|').expect("node|en|zh");
        let (en, zh) = rest.split_once('|').expect("node|en|zh");
        for (lang, template, is_zh) in [("en", en, false), ("zh", zh, true)] {
            assert_eq!(
                sublabel(&ir(&root, "architecture", lang), id),
                facts::template(template, is_zh),
                "docs/diagrams/architecture.{lang}.json: component {id} drifted from the \
                 registry — edit the IR, then re-render with `node scripts/diagram.mjs --write`"
            );
        }
    }
}

#[test]
fn the_chinese_diagrams_carry_no_chrome_the_renderer_should_have_mapped() {
    let root = repo_root();
    let terms = chrome_terms(&root);
    assert!(
        !terms.is_empty(),
        "the chrome map is empty; nothing is held"
    );
    for name in DIAGRAMS {
        for dir in ["docs/assets", "site/assets"] {
            let svg = read(&root, &format!("{dir}/{name}.zh.svg"));
            for term in &terms {
                assert!(
                    !svg.contains(term.as_str()),
                    "{dir}/{name}.zh.svg still carries {term:?}: the renderer maps it \
                     (scripts/diagram_svg.mjs CHROME.zh) but this file was rendered \
                     without the map, or by an older one — re-render with \
                     `node scripts/diagram.mjs --write`"
                );
            }
        }
    }
}
