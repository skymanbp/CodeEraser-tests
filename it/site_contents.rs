//! The how page opens with a contents panel: one row per section, that
//! section's families as numbered pills beside it. The panel is only
//! honest while it mirrors the headings under it — every link lands on
//! an id the page carries, every `<h2 id>` and every family heading
//! `<h3 id="fNN">` has a link, and a pill's number is the number of the
//! family it links. The one-line jump list this panel replaced had no
//! reader, and the family count grew from twelve to fifteen under it by
//! hand. The two languages are held to one outline: the same ids, in
//! the same order.

use crate::common::repo_root;
use crate::facts::read;
use std::collections::BTreeSet;

const PAGES: [&str; 2] = ["site/how/index.html", "site/zh/how/index.html"];

/// Every value of the attribute `needle` opens (`href="#`, ` id="`), in
/// page order.
fn values(text: &str, needle: &str) -> Vec<String> {
    text.match_indices(needle)
        .map(|(i, _)| {
            let rest = &text[i + needle.len()..];
            rest[..rest.find('"').expect("a closed attribute")].to_string()
        })
        .collect()
}

/// What the panel gets wrong on one page, and the outline it names.
fn audit(rel: &str, text: &str) -> (Vec<String>, Vec<String>) {
    let toc = text
        .split_once("<nav class=\"toc\"")
        .unwrap_or_else(|| panic!("{rel}: no contents panel"))
        .1
        .split_once("</nav>")
        .expect("a closed nav")
        .0;
    let linked = values(toc, "href=\"#");
    let ids: BTreeSet<String> = values(text, " id=\"").into_iter().collect();
    let set: BTreeSet<&String> = linked.iter().collect();
    let mut headings = values(text, "<h2 id=\"");
    headings.extend(values(text, "<h3 id=\""));
    let mut errors: Vec<String> = linked
        .iter()
        .filter(|id| !ids.contains(*id))
        .map(|id| format!("{rel}: the panel links #{id}; no element carries that id"))
        .collect();
    errors.extend(
        headings
            .iter()
            .filter(|id| !set.contains(id))
            .map(|id| format!("{rel}: heading #{id} has no link in the panel")),
    );
    for (i, _) in toc.match_indices("<b>") {
        let number = &toc[i + "<b>".len()..i + "<b>".len() + 2];
        if !toc[..i].ends_with(&format!("href=\"#f{number}\">")) {
            errors.push(format!(
                "{rel}: pill {number} does not link family f{number}"
            ));
        }
    }
    (errors, linked)
}

#[test]
fn the_contents_panel_mirrors_the_headings() {
    let root = repo_root();
    let outlines: Vec<Vec<String>> = PAGES
        .iter()
        .map(|rel| {
            let (errors, linked) = audit(rel, &read(&root, rel));
            assert!(errors.is_empty(), "{}", errors.join("\n"));
            linked
        })
        .collect();
    assert_eq!(
        outlines[0], outlines[1],
        "the two how pages' contents panels name different outlines"
    );
}

#[test]
fn a_dangling_link_an_unlisted_heading_and_a_misnumbered_pill_are_named() {
    let text = read(&repo_root(), PAGES[0]);
    let pill = "<li><a href=\"#f15\"><b>15</b>";
    assert!(text.contains(pill) && !text.contains("id=\"f16\""));
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
    ] {
        let (errors, _) = audit("probe", &mutant);
        assert!(
            errors.iter().any(|e| e.contains(said)),
            "the probe expected {said:?}, got {errors:?}"
        );
    }
}
