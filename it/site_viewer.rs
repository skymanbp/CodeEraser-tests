//! The site's picture viewer (site/viewer.js) is the first script the
//! eight pages carry, and it is a contract between three things that
//! no other gate reads together: the pages (which figures are stages,
//! which pages load the script, which cache-busting version they name),
//! the script (the camera's arithmetic and the words it writes into the
//! page in the page's language), and the READMEs (which deep-link into
//! the pages' figure ids as "zoom and pan this diagram at …").
//!
//! Five legs. The camera runs under Node (cli/tests/site/camera.js):
//! fitting, anchored zoom, the clamp and the scale bounds are numbers,
//! and a browser run would not state them. The pages: every page with
//! a `.stage` loads the script and its sheet (viewer.css) and no page
//! without one does, and the three cached files are asked for under
//! ONE `?v=` each across the site (a bump missed on one page pairs its
//! fresh markup with a stale, edge-cached file), and every stage holds
//! exactly one picture that states its box — the camera reads the
//! aspect off those attributes.
//! The words: the script's text table has the same keys in both
//! languages and each half speaks its own (docs_lang.rs cannot see
//! text a script writes). The READMEs: every `codeeraser.dev/…#id`
//! they name resolves to an `id="…"` on the committed page.

use crate::common::{expect_ok, node, repo_root};
use crate::docs_lang::SURFACES;
use crate::facts::read;
use std::collections::BTreeSet;

/// The picture pages, as (page, Chinese?): docs_lang's surfaces minus
/// the two READMEs, which are Markdown and hang no stage.
fn pages() -> Vec<(&'static str, bool)> {
    SURFACES
        .iter()
        .copied()
        .filter(|(rel, _)| rel.ends_with(".html"))
        .collect()
}

/// The `?v=` values one asset is asked for under, across the pages.
fn versions(pages: &[(&str, bool)], asset: &str) -> BTreeSet<String> {
    let root = repo_root();
    let needle = format!("\"/{asset}?v=");
    let mut seen = BTreeSet::new();
    for (rel, _) in pages {
        let text = read(&root, rel);
        for (i, _) in text.match_indices(&needle) {
            let rest = &text[i + needle.len()..];
            seen.insert(rest[..rest.find('"').expect("a closed src")].to_string());
        }
    }
    seen
}

#[test]
fn the_camera_holds_its_four_properties() {
    expect_ok(
        &node(&["cli/tests/site/camera.js"], &[]),
        "site/viewer.js: a camera property broke",
    );
}

#[test]
fn every_page_with_a_stage_loads_the_viewer_under_one_version() {
    let root = repo_root();
    let pages = pages();
    for (rel, _) in &pages {
        let text = read(&root, rel);
        let stages = text.matches("<div class=\"stage\">").count();
        let script = text.contains("<script src=\"/viewer.js?v=");
        let sheet = text.contains("<link rel=\"stylesheet\" href=\"/viewer.css?v=");
        assert!(
            (stages > 0) == script && script == sheet,
            "{rel}: {stages} stage(s), viewer.js {}, viewer.css {} — a page with a picture loads both, a page without loads neither",
            if script { "loaded" } else { "not loaded" },
            if sheet { "linked" } else { "not linked" }
        );
        // the script defers, so it never blocks the page it enhances
        assert!(
            !script || text.contains("\" defer></script>"),
            "{rel}: the viewer script must be deferred"
        );
    }
    for asset in ["style.css", "viewer.css", "viewer.js"] {
        let seen = versions(&pages, asset);
        assert_eq!(
            seen.len(),
            1,
            "{asset} is asked for under {seen:?}: one ?v= across the site, or a page pairs fresh markup with a stale cached file"
        );
    }
}

#[test]
fn every_stage_holds_one_picture_that_states_its_box() {
    let root = repo_root();
    for (rel, _) in pages() {
        let text = read(&root, rel);
        for (n, block) in text.split("<div class=\"stage\">").skip(1).enumerate() {
            let inner = block.split_once("</div>").expect("a closed stage").0;
            assert_eq!(
                inner.matches("<img ").count(),
                1,
                "{rel}: stage {n} holds {} pictures, not one",
                inner.matches("<img ").count()
            );
            let boxed = ["width=\"", "height=\""].iter().all(|attr| {
                inner
                    .split_once(attr)
                    .is_some_and(|(_, v)| v.starts_with(|c: char| c.is_ascii_digit()))
            });
            assert!(
                boxed,
                "{rel}: stage {n}'s picture states no width/height — the camera reads its aspect there"
            );
        }
    }
}

#[test]
fn the_bar_speaks_both_languages() {
    let out = node(
        &[
            "-e",
            "process.stdout.write(JSON.stringify(require('./site/viewer.js').TEXT))",
        ],
        &[],
    );
    expect_ok(
        &out,
        "site/viewer.js does not export its text table under Node",
    );
    let table: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("the text table is JSON");
    let keys = |lang: &str| -> BTreeSet<String> {
        table[lang]
            .as_object()
            .unwrap_or_else(|| panic!("TEXT.{lang} is a table"))
            .keys()
            .cloned()
            .collect()
    };
    assert_eq!(
        keys("en"),
        keys("zh"),
        "TEXT.en and TEXT.zh name different words"
    );
    assert!(
        !keys("en").is_empty(),
        "the text table is empty; nothing is held"
    );
    let cjk = |s: &str| s.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));
    for (key, en) in table["en"].as_object().expect("TEXT.en") {
        let zh = table["zh"][key].as_str().expect("a zh word");
        assert!(
            !cjk(en.as_str().expect("an en word")),
            "TEXT.en.{key} carries CJK: {en}"
        );
        assert!(cjk(zh), "TEXT.zh.{key} carries no Chinese: {zh}");
    }
}

#[test]
fn every_readme_deep_link_lands_on_a_figure_id() {
    let root = repo_root();
    let mut seen = 0;
    for (rel, _) in SURFACES.iter().filter(|(rel, _)| rel.ends_with(".md")) {
        let text = read(&root, rel);
        for (i, _) in text.match_indices("https://codeeraser.dev/") {
            let rest = &text["https://codeeraser.dev".len() + i..];
            let end = rest.find([')', ' ', '\n']).unwrap_or(rest.len());
            let Some((path, id)) = rest[..end].split_once('#') else {
                continue;
            };
            let page = format!("site{path}index.html");
            let html = read(&root, &page);
            assert!(
                html.contains(&format!(" id=\"{id}\"")),
                "{rel} links {path}#{id}, but {page} has no element with that id"
            );
            seen += 1;
        }
    }
    assert!(
        seen >= 4,
        "the READMEs name {seen} figure links; the two diagrams × two languages are expected"
    );
}
