//! ADR-009 gate 2 (plan v2.21 S3): the registry resolves, the
//! scraped tier only shrinks, and every source-literal site — a place
//! that spells a fact but cannot carry a chip: a Rust doc comment, a
//! JS comment, an SVG text node, the Haskell mirror of PROTO — spells
//! the registry's value verbatim. The report-id family's two-way
//! closure over cli/src runs inside facts::registry() itself.

use crate::common::repo_root;
use crate::facts::{self, Tier};

/// Scraped rows today. Lower it when a fact is promoted to a typed
/// path; raising it is the plan v2.21 ⑤ exception and goes in the
/// CHANGELOG by name.
const SCRAPED: usize = 15;

#[test]
fn every_fact_resolves_and_the_scraped_tier_only_shrinks() {
    let all = facts::registry();
    assert!(!all.is_empty());
    for f in all {
        assert_eq!(facts::resolve(&f.id), f.value);
        assert!(!f.source.is_empty(), "{}: a fact names its source", f.id);
    }
    let scraped: Vec<&str> = all
        .iter()
        .filter(|f| f.tier == Tier::Scraped)
        .map(|f| f.id.as_str())
        .collect();
    assert_eq!(
        scraped.len(),
        SCRAPED,
        "scraped tier ratchet (facts_registry.rs::SCRAPED): {scraped:?}"
    );
}

/// (file, template) — `{id}` placeholders render through the
/// registry; the file must contain the result.
const LITERALS: &[(&str, &str)] = &[
    (
        "cli/src/score/model.rs",
        "CI arms {gate:floor.main#digits},",
    ),
    ("gui/ui/score.js", "CI arms {gate:floor.main#digits} while"),
    (
        "site/assets/methodology.svg",
        "ce check . --fail-under {gate:floor.main#digits}</text>",
    ),
    ("site/assets/stack.svg", ">proto {ver:proto#v} · SemVer<"),
    ("site/assets/stack.svg", "GHC {tool:ghc#v}</text>"),
    (
        "site/assets/stack.svg",
        "floor {gate:floor.main#digits}</text>",
    ),
    (
        "core/app/CE/Protocol/Version.hs",
        "proto = \"{ver:proto#v}\"",
    ),
];

fn render_template(template: &str) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let j = rest[i..].find('}').expect("closing brace") + i;
        out.push_str(&facts::resolve(&rest[i + 1..j]));
        rest = &rest[j + 1..];
    }
    out.push_str(rest);
    out
}

#[test]
fn source_literal_sites_spell_the_registry_value() {
    let root = repo_root();
    for (rel, template) in LITERALS {
        let want = render_template(template);
        let text = std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
        assert!(
            text.contains(&want),
            "{rel} does not contain {want:?} (template {template:?})"
        );
    }
}
