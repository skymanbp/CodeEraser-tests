//! The how-page consts parser (split out of docs_consts.rs at the
//! 300-line dogfood wall, plan v2.31 step 2, when the analysis page
//! joined the parse): the pages docs_lang lists under /how/ in one
//! language, each `<ul class="consts">` block keyed by the numbered
//! heading before it, each `<li><b>name</b> value</li>` a chip. The
//! leaf owns the shape; docs_consts binds and judges.

use crate::docs_lang::SURFACES;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) struct Chip {
    pub(crate) name: String,
    pub(crate) value: String,
}

pub(crate) type Families = BTreeMap<String, Vec<Chip>>;

fn page(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn between<'a>(text: &'a str, start: usize, end: usize, what: &str) -> &'a str {
    text.get(start..end)
        .unwrap_or_else(|| panic!("consts parser: invalid {what} range"))
}

/// Find `pat` at or after `from`, or panic naming the surface — the
/// ONE owner of the find-or-refuse idiom both parsers lean on.
fn seek(text: &str, from: usize, pat: &str, what: &str) -> usize {
    text[from..]
        .find(pat)
        .map(|i| from + i)
        .unwrap_or_else(|| panic!("{what} has no {pat}"))
}

fn parse_chips(body: &str, label: &str, family: &str) -> Vec<Chip> {
    let mut chips = Vec::new();
    let mut item = 0;
    let ctx = format!("{label} family {family}: chip");
    while let Some(rel) = body[item..].find("<li><b>") {
        let name_start = item + rel + "<li><b>".len();
        let name_end = seek(body, name_start, "</b>", &ctx);
        let value_start = name_end + "</b>".len();
        let value_end = seek(body, value_start, "</li>", &ctx);
        chips.push(Chip {
            name: between(body, name_start, name_end, "chip name").to_string(),
            value: between(body, value_start, value_end, "chip value")
                .trim()
                .to_string(),
        });
        item = value_end + "</li>".len();
    }
    chips
}

/// The chip pages of one language — docs_lang's how pages, which
/// continue one family numbering from the how page onto the analysis
/// track's page; a family on two pages is a fault, not a merge.
pub(crate) fn families(root: &Path, zh: bool) -> Families {
    let label = if zh { "ZH" } else { "EN" };
    let mut all = Families::new();
    for (rel, _) in SURFACES
        .iter()
        .filter(|(rel, z)| *z == zh && rel.contains("/how/"))
    {
        for (family, chips) in parse_page(&page(root, rel), label) {
            assert!(
                all.insert(family.clone(), chips).is_none(),
                "{rel}: family {family} is on another page too"
            );
        }
    }
    all
}

fn parse_page(text: &str, label: &str) -> Families {
    let mut families = BTreeMap::new();
    let mut cursor = 0;
    while let Some(rel) = text[cursor..].find(r#"<ul class="consts">"#) {
        let ul = cursor + rel;
        let end = seek(text, ul, "</ul>", &format!("{label}: consts block"));
        let before = &text[..ul];
        // the number span, not the <h3> around it: the heading carries
        // an id (`<h3 id="fNN">`) so the page's jump list can reach it
        let heading = before
            .rfind(r#"<span class="n">"#)
            .unwrap_or_else(|| panic!("{label}: consts block with no preceding numbered heading"));
        let n_start = heading + r#"<span class="n">"#.len();
        let n_end = seek(
            text,
            n_start,
            "</span>",
            &format!("{label}: numbered heading"),
        );
        let family = between(text, n_start, n_end, "family number").to_string();
        assert!(
            !family.is_empty() && family.bytes().all(|b| b.is_ascii_digit()),
            "{label}: consts block heading is not numbered"
        );
        let body = between(
            text,
            ul + r#"<ul class="consts">"#.len(),
            end,
            "consts body",
        );
        assert!(
            families
                .insert(family.clone(), parse_chips(body, label, &family))
                .is_none(),
            "{label}: duplicate family {family}"
        );
        cursor = end + "</ul>".len();
    }
    families
}
