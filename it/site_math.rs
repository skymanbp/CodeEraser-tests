//! The math page pair (site/math, site/zh/math) gathers one entry per
//! rule and repeats, under each, the constants in play — copies of the
//! how pages' `<ul class="consts">` chips, which docs_consts binds to
//! their source constants. A copy nothing reads drifts the day the
//! constant moves, so this gate holds every chip on the math page to a
//! chip of the same name and the same value in the family its entry
//! is numbered by (the `<span class="n">` before it), on the how page
//! of the same language.

use crate::common::repo_root;
use crate::docs_consts_parts::page::{Families, families, parse_chips};
use crate::facts::read;

const PAGES: [(&str, bool); 2] = [
    ("site/math/index.html", false),
    ("site/zh/math/index.html", true),
];

/// Every math-page chip with no twin in its family, named.
fn strays(label: &str, text: &str, how: &Families) -> Vec<String> {
    let mut out = Vec::new();
    for (at, _) in text.match_indices(r#"<ul class="consts">"#) {
        let n_at = text[..at]
            .rfind(r#"<span class="n">"#)
            .expect("an entry number")
            + r#"<span class="n">"#.len();
        let family = &text[n_at..n_at + 2];
        let body = &text[at..at + text[at..].find("</ul>").expect("a closed list")];
        let twins = how.get(family).map_or(&[][..], Vec::as_slice);
        for chip in parse_chips(body, label, family) {
            if !twins
                .iter()
                .any(|t| t.name == chip.name && t.value == chip.value)
            {
                out.push(format!(
                    "{label}: family {family} chip {} {} is not on the how page",
                    chip.name, chip.value
                ));
            }
        }
    }
    out
}

#[test]
fn math_page_constants_are_the_how_pages_chips() {
    let root = repo_root();
    for (rel, zh) in PAGES {
        let text = read(&root, rel);
        assert!(
            text.contains(r#"<ul class="consts">"#),
            "{rel}: no constants at all"
        );
        let errors = strays(rel, &text, &families(&root, zh));
        assert!(errors.is_empty(), "{}", errors.join("\n"));
    }
}

#[test]
fn a_moved_constant_is_named() {
    let root = repo_root();
    let text = read(&root, PAGES[0].0);
    let chip = "<li><b>kgram</b> 25</li>";
    assert!(text.contains(chip));
    let errors = strays(
        "probe",
        &text.replacen(chip, "<li><b>kgram</b> 26</li>", 1),
        &families(&root, false),
    );
    assert!(
        errors.iter().any(|e| e.contains("family 01 chip kgram 26")),
        "{errors:?}"
    );
}
