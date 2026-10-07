//! HTML: seeded trees of pages (with and without `<html lang>`, one or
//! two `<base href>`s, canonical links under odd `rel` spellings, an
//! `og:url`, hreflang alternates, ids — duplicated, non-ASCII, escaped,
//! empty — and URLs that name the page under each of its ancestors, as a
//! directory URL, an extensionless one or the file itself, on the own
//! host and others) beside Markdown documents with headings, code files,
//! walked assets and directories with and without an index page, a
//! declared `html` search root in some trees (declared empty in a few):
//! one leg per aim — `href` references (relative, root-relative,
//! absolute, odd), the fetch kinds (`src`, `srcset`, an asset `<link>`;
//! empty ones among them), forms (`action`, empty = the page), cross-page
//! fragments (against a page's ids and a Markdown document's slugs,
//! percent- and character-escaped, behind a query) and absolute URLs
//! (userinfo, port, host case, empty path). The pages are written under
//! the tree root: the frozen rungs read them, the request's facts too.

use super::common::{Tree, World, legs, put};
use super::rng::Rng;
use super::tables::t;
use crate::scan::lang::Lang;
use std::collections::BTreeSet;
use std::path::Path;

fn is_html(p: &str) -> bool {
    Lang::from_path(Path::new(p)) == Some(Lang::Html)
}

fn is_md(p: &str) -> bool {
    Lang::from_path(Path::new(p)) == Some(Lang::Markdown)
}

/// A URL path naming `path` under one of its ancestor directories: the
/// directory part cut at a random depth, then (now and then) its index
/// page dropped to a directory URL or its `.html` to an extensionless one.
fn served(rng: &mut Rng, path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    let rest = parts[rng.below(parts.len())..].join("/");
    let rest = match rng.below(4) {
        0 => ["index.html", "index.htm"]
            .iter()
            .find_map(|i| rest.strip_suffix(i))
            .unwrap_or(&rest)
            .to_string(),
        1 => rest.strip_suffix(".html").unwrap_or(&rest).to_string(),
        _ => rest,
    };
    format!("/{rest}")
}

/// A URL: an origin of the table, then a path serving `page` (mostly) or
/// another file of the tree, now and then a query or a fragment.
fn url(rng: &mut Rng, page: &str, own: &[String]) -> String {
    let path = if rng.chance(65) || own.is_empty() {
        served(rng, page)
    } else {
        let other = own[rng.below(own.len())].clone();
        served(rng, &other)
    };
    let tail = ["", "", "", "?v=2", "#top", "?a=1#x", "/"][rng.below(7)];
    format!("{}{path}{tail}", rng.pick(t("html ORIGINS")))
}

/// A page: its `<html lang>`, a head of bases, canonical links, an
/// `og:url` and alternates naming it, and a body of ids.
fn page(rng: &mut Rng, path: &str, own: &[String]) -> String {
    let mut text = match rng.pick(t("html LANGS")) {
        "-" => "<!doctype html><html><head>".to_string(),
        l => format!("<!doctype html><html lang=\"{l}\"><head>"),
    };
    for _ in 0..rng.below(3) {
        let href = rng.pick(t("html BASE_HREFS"));
        text.push_str(&format!("<base href=\"{href}\">"));
    }
    if rng.chance(35) {
        let prop = ["og:url", "og:url", "OG:url", "og:title"][rng.below(4)];
        let content = url(rng, path, own);
        text.push_str(&format!("<meta property=\"{prop}\" content=\"{content}\">"));
    }
    // links: a canonical-ish `rel` from the table, or an hreflang alternate
    for _ in 0..rng.below(7) {
        let href = url(rng, path, own);
        let link = if rng.chance(45) {
            format!("rel=\"{}\"", rng.pick(t("html RELS")))
        } else {
            let rel = ["alternate", "Alternate", "alternate stylesheet"][rng.below(3)];
            format!(
                "rel=\"{rel}\" hreflang=\"{}\"",
                rng.pick(t("html HREFLANGS"))
            )
        };
        text.push_str(&format!("<link {link} href=\"{href}\">"));
    }
    text.push_str("</head><body>");
    for _ in 0..rng.below(6) {
        let id = rng.pick(t("html IDS"));
        let tag = ["h2", "p", "a", "section"][rng.below(4)];
        text.push_str(&format!("<{tag} id=\"{id}\">x</{tag}>"));
    }
    text.push_str("</body></html>\n");
    text
}

/// A reference written on page `from`: a tree file reached from its
/// directory or the root, a served URL, or a table target; then now and
/// then a query, a fragment, an escape.
fn target(rng: &mut Rng, from: &str, own: &[String], aim: usize) -> String {
    let file = |rng: &mut Rng| own[rng.below(own.len())].clone();
    let mut s = match (rng.below(5), own.is_empty()) {
        (0, false) => format!("{}{}", "../".repeat(from.matches('/').count()), file(rng)),
        (1, false) => {
            let f = file(rng);
            let dir = from.rsplit_once('/').map_or("", |(d, _)| d);
            f.strip_prefix(&format!("{dir}/")).unwrap_or(&f).to_string()
        }
        (2, false) => {
            let f = file(rng);
            served(rng, &f)
        }
        (3, false) => {
            let f = file(rng);
            url(rng, &f, own)
        }
        _ => rng.pick(t("html TARGETS")).to_string(),
    };
    if aim == 4 && rng.chance(70) && !own.is_empty() {
        let f = file(rng);
        s = url(rng, &f, own);
    }
    if rng.chance(15) {
        s.push_str(["?q", "?", "?a=1&amp;b=2"][rng.below(3)]);
    }
    if rng.chance(if aim == 3 { 80 } else { 25 }) {
        s = format!("{s}#{}", rng.pick(t("html FRAGS")));
    }
    if rng.chance(10) {
        s = s.replacen('/', ["%2F", "&#47;", "&#x2f;"][rng.below(3)], 1);
    }
    if rng.chance(8) {
        s = s.replace(' ', "%20");
    }
    s
}

/// The site kind of a leg's site: hrefs, the fetch kinds, forms.
fn kind(rng: &mut Rng, aim: usize) -> &'static str {
    match aim {
        1 => ["src", "srcset", "link_asset"][rng.below(3)],
        2 => "action",
        _ => "href",
    }
}

/// The tree of a leg aimed at `aim` (the LEGS column): six to
/// twenty-five walked files, up to six walked assets, the pages and the
/// Markdown documents written, a page beside some (unwalked), a declared
/// `html` root in some trees.
fn tree(rng: &mut Rng, root: &Path, aim: usize) -> Tree {
    let (n, assets) = (6 + rng.below(20), rng.below(7));
    let mut world = World {
        files: rng.paths(t("html DIRS"), t("html BASES"), n),
        assets: rng.paths("|", t("html ASSETS"), assets),
        ..World::default()
    };
    if rng.chance(4) {
        world.search_roots.insert("html".into(), BTreeSet::new());
    } else {
        world.declare(rng, "html", t("html ROOTS"));
    }
    let own: Vec<String> = world.files.iter().chain(&world.assets).cloned().collect();
    write(rng, root, &world, &own);
    let owned = world.sites(rng, is_html, (t("html DIRS"), "zz.html"), |rng, from| {
        let spec = if aim != 0 && rng.chance(12) {
            String::new()
        } else {
            target(rng, &from, &own, aim)
        };
        (Lang::Html, kind(rng, aim), from, spec)
    });
    (world, owned)
}

/// Every walked page and Markdown document written, and an unwalked page
/// in some directories (a site's own file the walk did not hold).
fn write(rng: &mut Rng, root: &Path, world: &World, own: &[String]) {
    for f in &world.files {
        if is_html(f) {
            put(root, f, &page(rng, f, own));
        } else if is_md(f) {
            let heads: Vec<String> = (0..rng.below(4))
                .map(|_| format!("# {}\n", rng.pick(t("html MD_HEADS"))))
                .collect();
            put(root, f, &heads.concat());
        }
    }
    for dir in t("html DIRS").split('|') {
        if rng.chance(15) {
            let f = format!("{dir}/zz.html");
            let f = f.trim_start_matches('/');
            put(root, f, &page(rng, f, own));
        }
    }
}

#[test]
#[ignore = "instrument: needs a core (CE_CORE_BIN); run with --ignored --nocapture"]
fn html_random_trees_agree() {
    legs("html", tree);
}
