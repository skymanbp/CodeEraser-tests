//! Lock the stack page's authored numeric facts across EN/ZH and
//! back to the registry (facts/), which owns their sources. The
//! `data-const` span is the page's own chip form — read, never
//! rewritten: three values, each a registry id. Evaluation values
//! are deliberately absent: bench_render_dashboard owns those
//! generated blocks.

use crate::common::repo_root;
use crate::facts;
use std::collections::BTreeMap;
use std::path::Path;

fn constants(path: &Path) -> BTreeMap<String, String> {
    let page = std::fs::read_to_string(path).expect("read stack page");
    let mut found = BTreeMap::new();
    let (marker, end) = (r#"<span data-const=""#, "</span>");
    let mut rest = page.as_str();
    while let Some(start) = rest.find(marker) {
        let named = &rest[start + marker.len()..];
        let quote = named.find('"').expect("data-const closing quote");
        let name = &named[..quote];
        let body = &named[quote + 1..];
        let open = body.find('>').expect("data-const tag close") + 1;
        let close = body[open..].find(end).expect("data-const closing tag") + open;
        assert!(
            found
                .insert(name.to_string(), body[open..close].to_string())
                .is_none(),
            "duplicate stack constant {name}"
        );
        rest = &body[close + end.len()..];
    }
    found
}

/// data-const name → registry id.
const IDS: &[(&str, &str)] = &[
    ("ci-floor", "gate:floor.main#digits"),
    ("ghc", "tool:ghc#v"),
    ("proto", "ver:proto#v"),
];

#[test]
fn stack_page_constants_are_locked_and_resolvable() {
    let root = repo_root();
    let en = constants(&root.join("site/stack/index.html"));
    let zh = constants(&root.join("site/zh/stack/index.html"));
    assert_eq!(en, zh, "stack EN/ZH numeric facts drifted");
    let want: BTreeMap<String, String> = IDS
        .iter()
        .map(|(name, id)| (name.to_string(), facts::resolve(id)))
        .collect();
    assert_eq!(en, want, "stack page data-const values vs the registry");
    let site_svg = std::fs::read(root.join("site/assets/stack.svg")).expect("site stack svg");
    let docs_svg = std::fs::read(root.join("docs/assets/stack.svg")).expect("docs stack svg");
    assert_eq!(site_svg, docs_svg, "site/docs stack diagrams differ");
    let svg = String::from_utf8(site_svg).expect("stack svg utf8");
    for value in en.values() {
        assert!(svg.contains(value), "stack diagram omits constant {value}");
    }
}
