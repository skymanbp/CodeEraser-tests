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
const SCRAPED: usize = 22;

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

/// (file, template, Chinese?) — `{id}` placeholders render through
/// the registry in the site's language; the file must contain the
/// result. Beside the comments and SVG text nodes (the two hand-drawn
/// diagrams in both languages): the how pages' `<title>` / `<meta>`
/// and the `alt` text of the stack and architecture pictures, where a
/// comment would be literal text.
const LITERALS: &[(&str, &str, bool)] = &[
    (
        "cli/src/score/model.rs",
        "CI arms {gate:floor.main#digits},",
        false,
    ),
    (
        "gui/ui/score.js",
        "CI arms {gate:floor.main#digits} while",
        false,
    ),
    (
        "site/assets/methodology.svg",
        "ce check . --fail-under {gate:floor.main#digits}</text>",
        false,
    ),
    (
        "site/assets/stack.svg",
        ">proto {ver:proto#v} · SemVer<",
        false,
    ),
    ("site/assets/stack.svg", "GHC {tool:ghc#v}</text>", false),
    (
        "site/assets/stack.svg",
        "floor {gate:floor.main#digits}</text>",
        false,
    ),
    (
        "core/app/CE/Protocol/Version.hs",
        "proto = \"{ver:proto#v}\"",
        false,
    ),
    (
        "site/how/index.html",
        "<title>How CodeEraser works — deterministic judgment, {count:booklets#word} families</title>",
        false,
    ),
    (
        "site/how/index.html",
        "language models: {count:booklets#word} judgment families,",
        false,
    ),
    (
        "site/zh/how/index.html",
        "<title>CodeEraser 工作原理——确定性判决，{count:booklets#word}个家族</title>",
        true,
    ),
    (
        "site/zh/how/index.html",
        "非确定性产出：{count:booklets#word}个判决家族，",
        true,
    ),
    (
        "site/stack/index.html",
        "NDJSON wire and {count:families#word} judgment families,",
        false,
    ),
    (
        "site/zh/stack/index.html",
        "NDJSON wire 与{count:families#word}个判决族、",
        true,
    ),
    (
        "site/assets/methodology.svg",
        "— {count:fail_conditions#word} named conditions",
        false,
    ),
    (
        "site/assets/methodology.zh.svg",
        "ce check . --fail-under {gate:floor.main#digits}</text>",
        true,
    ),
    (
        "site/assets/methodology.zh.svg",
        "——{count:fail_conditions#word}个具名条件",
        true,
    ),
    (
        "site/assets/stack.zh.svg",
        ">proto {ver:proto#v} · SemVer<",
        true,
    ),
    ("site/assets/stack.zh.svg", "GHC {tool:ghc#v}</text>", true),
    (
        "site/assets/stack.zh.svg",
        "地板 {gate:floor.main#digits}</text>",
        true,
    ),
    (
        "site/index.html",
        "NDJSON wire of {count:families#word} families to the Haskell",
        false,
    ),
    (
        "site/zh/index.html",
        "经一条{count:families#word}个家族的 NDJSON wire",
        true,
    ),
    (
        "README.md",
        "NDJSON wire of {count:families#word} families to the Haskell",
        false,
    ),
    (
        "README.zh.md",
        "经一条{count:families#word}个家族的 NDJSON wire",
        true,
    ),
];

fn render_template(template: &str, zh: bool) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let j = rest[i..].find('}').expect("closing brace") + i;
        out.push_str(&facts::render(&rest[i + 1..j], zh));
        rest = &rest[j + 1..];
    }
    out.push_str(rest);
    out
}

#[test]
fn source_literal_sites_spell_the_registry_value() {
    let root = repo_root();
    for (rel, template, zh) in LITERALS {
        let want = render_template(template, *zh);
        let text = std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
        assert!(
            text.contains(&want),
            "{rel} does not contain {want:?} (template {template:?})"
        );
    }
}
