//! The how pages open with a contents panel: one row per section, that
//! section's families as numbered pills beside it. The panel is only
//! honest while it mirrors the headings it names — every link lands on
//! an id the page carries (or, for a link into a sibling page by its
//! served path, an id that page carries), every `<h2 id>` and every
//! family heading `<h3 id="fNN">` has a link, and a pill's number is
//! the number of the family it links. The one-line jump list this
//! panel replaced had no reader, and the family count grew from twelve
//! to fifteen under it by hand. Since the analysis track (plan v2.31)
//! the families continue on a second page pair, so a pill may point
//! across; the two languages of each pair are held to one outline: the
//! same targets, in the same order, the `/zh` prefix set aside.

use crate::common::repo_root;
use crate::facts::read;
use std::collections::{BTreeMap, BTreeSet};

/// `served-path file` per line: the English page, then its Chinese
/// twin, pair after pair.
const PAGES: &str = "\
/how/ site/how/index.html
/zh/how/ site/zh/how/index.html
/how/analysis/ site/how/analysis/index.html
/zh/how/analysis/ site/zh/how/analysis/index.html
";

/// The pages read: (served path, file, text).
fn loaded() -> Vec<(&'static str, &'static str, String)> {
    let root = repo_root();
    PAGES
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let (path, rel) = l.split_once(' ').expect("served-path file");
            (path, rel, read(&root, rel))
        })
        .collect()
}

/// Every value of the attribute `needle` opens (`href="`, ` id="`), in
/// page order.
fn values(text: &str, needle: &str) -> Vec<String> {
    text.match_indices(needle)
        .map(|(i, _)| {
            let rest = &text[i + needle.len()..];
            rest[..rest.find('"').expect("a closed attribute")].to_string()
        })
        .collect()
}

/// The ids every page carries, by served path.
fn sites(pages: &[(&str, &str, String)]) -> BTreeMap<String, BTreeSet<String>> {
    pages
        .iter()
        .map(|(path, _, text)| {
            (
                path.to_string(),
                values(text, " id=\"").into_iter().collect(),
            )
        })
        .collect()
}

/// A link's fault, if any: a fragment this page (or the sibling page
/// the path names) does not carry, or a path no page is served at.
fn bad_link(
    rel: &str,
    href: &str,
    own: &BTreeSet<String>,
    sites: &BTreeMap<String, BTreeSet<String>>,
) -> Option<String> {
    let (path, id) = href.split_once('#').unwrap_or((href, ""));
    let ids = if path.is_empty() {
        Some(own)
    } else {
        sites.get(path)
    };
    match ids {
        None => Some(format!(
            "{rel}: the panel links {href}; no page is served there"
        )),
        Some(ids) if !id.is_empty() && !ids.contains(id) => Some(format!(
            "{rel}: the panel links {href}; no element carries that id"
        )),
        _ => None,
    }
}

/// What the panel gets wrong on one page, and the targets it names.
fn audit(
    rel: &str,
    text: &str,
    sites: &BTreeMap<String, BTreeSet<String>>,
) -> (Vec<String>, Vec<String>) {
    let toc = text
        .split_once("<nav class=\"toc\"")
        .unwrap_or_else(|| panic!("{rel}: no contents panel"))
        .1
        .split_once("</nav>")
        .expect("a closed nav")
        .0;
    let own: BTreeSet<String> = values(text, " id=\"").into_iter().collect();
    let linked = values(toc, "href=\"");
    let mut errors: Vec<String> = linked
        .iter()
        .filter_map(|href| bad_link(rel, href, &own, sites))
        .collect();
    let fragments: BTreeSet<&str> = linked.iter().filter_map(|h| h.strip_prefix('#')).collect();
    let mut headings = values(text, "<h2 id=\"");
    headings.extend(values(text, "<h3 id=\""));
    errors.extend(
        headings
            .iter()
            .filter(|id| !fragments.contains(id.as_str()))
            .map(|id| format!("{rel}: heading #{id} has no link in the panel")),
    );
    for (i, _) in toc.match_indices("<b>") {
        let number = &toc[i + "<b>".len()..i + "<b>".len() + 2];
        if !toc[..i].ends_with(&format!("#f{number}\">")) {
            errors.push(format!(
                "{rel}: pill {number} does not link family f{number}"
            ));
        }
    }
    (errors, linked)
}

/// A panel's outline with the language prefix set aside, so a pair
/// compares target for target.
fn outline(linked: Vec<String>) -> Vec<String> {
    linked
        .into_iter()
        .map(|h| {
            h.strip_prefix("/zh")
                .map_or_else(|| h.clone(), str::to_string)
        })
        .collect()
}

#[test]
fn the_contents_panel_mirrors_the_headings() {
    let pages = loaded();
    let sites = sites(&pages);
    let outlines: Vec<Vec<String>> = pages
        .iter()
        .map(|(_, rel, text)| {
            let (errors, linked) = audit(rel, text, &sites);
            assert!(errors.is_empty(), "{}", errors.join("\n"));
            outline(linked)
        })
        .collect();
    for pair in outlines.chunks(2) {
        assert_eq!(
            pair[0], pair[1],
            "a page pair's contents panels name different outlines"
        );
    }
}

#[test]
fn a_dangling_link_an_unlisted_heading_and_a_misnumbered_pill_are_named() {
    let pages = loaded();
    let sites = sites(&pages);
    let text = &pages[0].2;
    let pill = "<li><a href=\"#f15\"><b>15</b>";
    let across = "<a href=\"/how/analysis/#f16\">";
    assert!(text.contains(pill) && text.contains(across) && !text.contains("id=\"f16\""));
    for (mutant, said) in [
        (
            text.replace(pill, "<li><a href=\"#f16\"><b>16</b>"),
            "links #f16",
        ),
        (
            text.replacen("<h3 id=\"f15\"", "<h3 id=\"f16\"", 1),
            "heading #f16 has no link",
        ),
        (
            text.replace(pill, "<li><a href=\"#f15\"><b>16</b>"),
            "pill 16 does not link family f16",
        ),
        (
            text.replace(across, "<a href=\"/how/analysis/#f99\">"),
            "links /how/analysis/#f99",
        ),
        (
            text.replace(across, "<a href=\"/how/nowhere/#f16\">"),
            "no page is served there",
        ),
    ] {
        let (errors, _) = audit("probe", &mutant, &sites);
        assert!(
            errors.iter().any(|e| e.contains(said)),
            "the probe expected {said:?}, got {errors:?}"
        );
    }
}
