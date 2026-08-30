//! The two bilingual diagrams (architecture, judgment data flow) are
//! archify IR under docs/diagrams/, rendered by scripts/diagram.mjs
//! from one pinned archify commit into docs/assets + site/assets. Four
//! things are held here (plan v2.21, architecture-diagram clause): the
//! cache is present AND at the pin — a gate that cannot render refuses
//! by name instead of passing; the committed SVGs are byte-for-byte
//! what this pin renders (CE_BLESS=1 rewrites them locally); the two
//! languages of one diagram share every byte of geometry and differ
//! only in text; every `sources` path an IR cites exists in the tree.

use crate::common::repo_root;
use crate::facts::{blessing, read};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

const PIN: &str = "e1ac748f19cf805e44bf74fb93c796662152e273"; // tt-a1i/archify v2.15.0
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

fn cache_head(root: &Path) -> Option<String> {
    let out = Command::new("git")
        .args(["-C", "cli/target/archify", "rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

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
    let root = repo_root();
    let head = cache_head(&root);
    assert_eq!(
        head.as_deref(),
        Some(PIN),
        "archify cache absent or off-pin ({}): run `node scripts/diagram.mjs --fetch`",
        head.as_deref().unwrap_or("absent")
    );
    let mode = if blessing() { "--write" } else { "--check" };
    let out = Command::new("node")
        .args(["scripts/diagram.mjs", mode])
        .current_dir(&root)
        .output()
        .expect("node is on PATH (the diagram driver needs no packages)");
    assert!(
        out.status.success(),
        "diagrams drifted or failed to render:\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
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
