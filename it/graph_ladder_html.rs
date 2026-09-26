//! HTML ladder fixtures (plan v2.30 step 5): a page names what the site
//! serves — another page, a document, an asset the index never holds —
//! so the habitat is a small site under `site/` (its pages carry the
//! `og:url` / `canonical` / `hreflang` facts the deployed pages carry,
//! from which the deployment root is derived), two pages outside it
//! with and without a `<base>`, and two trees that hold one
//! root-relative path each. Ambiguity rows MUST refuse: a root-relative
//! path two ancestors hold, or two declared roots serve, resolving is
//! the red condition; a fragment onto a duplicated id must degrade to
//! the file.

use codeeraser::graph::deadcode::Advisory;
use codeeraser::graph::wire::{EDGE_ASSET, EDGE_DOC_LINK};
use codeeraser::scan::lang::Lang;
use std::path::{Path, PathBuf};

use crate::common::{self, text_ladder};

/// The habitat, `==== path` over each file, then the rows
/// (common::text_ladder). Rung order is evaluation order: R1 joins on
/// the page's directory (or its `<base href>`, docs/index.html — a base
/// on another host sends the relative references off the site,
/// docs/away.html); a root-relative path asks the declared `html` roots
/// (the `@rooted` runs), else the root the page's own URL derives
/// (`site/` pages, rung 2), else the one ancestor directory holding it
/// (rung 3); an absolute URL on the page's own host is its
/// root-relative path, any other host or scheme External; a bare
/// fragment is the page's section as written (rung 4), a cross-page
/// fragment validated against the target's ids (rung 2). The query is
/// dropped, escapes decode, a directory serves its index page, and an
/// empty value is the page for a page or form reference and nothing
/// for an asset fetch.
const LADDER: &str = r##"
==== site/index.html
<!doctype html>
<html><head>
<meta property="og:url" content="https://codeeraser.dev/">
<link rel="alternate" hreflang="zh" href="https://codeeraser.dev/zh/">
<link rel="stylesheet" href="style.css">
</head><body>
<h2 id="install">Install</h2><h2 id="dup">A</h2><h2 id="dup">B</h2>
</body></html>
==== site/how/index.html
<html><head><meta property="og:url" content="https://codeeraser.dev/how/"></head>
<body><h2 id="verdict">Verdict</h2></body></html>
==== site/zh/index.html
<html><head><link rel="canonical" href="https://codeeraser.dev/zh/index.html"></head><body></body></html>
==== site/style.css
body {}
==== site/assets/logo.png
png
==== site/assets/a&b.png
png
==== site/docs/guide.md
# Guide
## Setup
==== docs/index.html
<html><head><base href="/site/"></head><body></body></html>
==== docs/away.html
<html><head><base href="https://cdn.example/lib/"></head><body></body></html>
==== docs/other.html
<html><body></body></html>
==== app/index.html
<html><body></body></html>
==== app/img/t.png
png
==== img/t.png
png
==== package.json
{}
==== @cases
href @@ site/index.html @@ how/ @@ ok site/how/index.html 1
href @@ site/index.html @@ ./how/index.html @@ ok site/how/index.html 1
href @@ site/index.html @@ how @@ ok site/how/index.html 1
src @@ site/index.html @@ assets/logo.png @@ ok site/assets/logo.png 1
srcset @@ site/index.html @@ assets/logo.png @@ ok site/assets/logo.png 1
src @@ site/index.html @@ assets/logo.png#x @@ ok site/assets/logo.png 1
link_asset @@ site/index.html @@ style.css @@ ok site/style.css 1
link_asset @@ site/index.html @@ style.css?v=3 @@ ok site/style.css 1
src @@ site/index.html @@ assets/a&amp;b.png @@ ok site/assets/a&b.png 1
src @@ site/index.html @@ assets/a%26b.png @@ ok site/assets/a&b.png 1
href @@ site/index.html @@ docs/guide.md @@ ok site/docs/guide.md 1
href @@ site/index.html @@ docs/guide.md#setup @@ sec site/docs/guide.md setup 2
href @@ site/index.html @@ docs/guide.md#nope @@ secf site/docs/guide.md 2
href @@ site/index.html @@ how/#verdict @@ sec site/how/index.html verdict 2
href @@ site/index.html @@ how/#nope @@ secf site/how/index.html 2
href @@ site/how/index.html @@ ../index.html#install @@ sec site/index.html install 2
href @@ site/how/index.html @@ ../index.html#dup @@ secf site/index.html 2
href @@ site/index.html @@ #install @@ sec site/index.html install 4
href @@ site/index.html @@ #whatever @@ sec site/index.html whatever 4
href @@ site/index.html @@ # @@ ok site/index.html 4
href @@ site/index.html @@ ?q=1 @@ ok site/index.html 4
href @@ site/index.html @@ /zh/ @@ ok site/zh/index.html 2
action @@ site/index.html @@ /how/ @@ ok site/how/index.html 2
href @@ site/how/index.html @@ /assets/logo.png @@ ok site/assets/logo.png 2
href @@ site/zh/index.html @@ /how/ @@ ok site/how/index.html 2
href @@ site/index.html @@ /nope/ @@ no out_of_scope
href @@ site/index.html @@ https://codeeraser.dev/how/#verdict @@ sec site/how/index.html verdict 2
href @@ site/index.html @@ https://codeeraser.dev @@ ok site/index.html 2
href @@ site/index.html @@ HTTPS://CodeEraser.dev/zh/ @@ ok site/zh/index.html 2
href @@ site/index.html @@ https://github.com/x/y @@ ext 5
src @@ site/index.html @@ //cdn.example/x.js @@ ext 5
href @@ site/index.html @@ mailto:a@b.example @@ ext 5
href @@ site/index.html @@ javascript:void(0) @@ ext 5
href @@ site/index.html @@ tel:+1 @@ ext 5
src @@ site/index.html @@ data:text/plain,hi @@ ext 5
src @@ site/index.html @@ missing.png @@ no out_of_scope
href @@ site/index.html @@ ../../escape.html @@ no out_of_scope
href @@ site/index.html @@  @@ ok site/index.html 4
action @@ site/index.html @@  @@ ok site/index.html 4
src @@ site/index.html @@  @@ no empty
srcset @@ site/index.html @@  @@ no empty
link_asset @@ site/index.html @@  @@ no empty
href @@ docs/other.html @@ /site/assets/logo.png @@ ok site/assets/logo.png 3
href @@ docs/other.html @@ /img/t.png @@ ok img/t.png 3
href @@ docs/other.html @@ /site/ @@ ok site/index.html 3
href @@ app/index.html @@ /img/t.png @@ no ambiguous_root
href @@ app/index.html @@ /assets/logo.png @@ no out_of_scope
src @@ docs/index.html @@ assets/logo.png @@ ok site/assets/logo.png 1
href @@ docs/index.html @@ /site/ @@ ok site/index.html 3
src @@ docs/away.html @@ x.js @@ ext 5
href @@ docs/away.html @@ /site/ @@ ok site/index.html 3
==== @rooted site
href @@ docs/other.html @@ /assets/logo.png @@ ok site/assets/logo.png 2
href @@ docs/other.html @@ /img/t.png @@ no out_of_scope
href @@ site/how/index.html @@ /zh/ @@ ok site/zh/index.html 2
==== @rooted site app
href @@ docs/other.html @@ /index.html @@ no ambiguous_root
href @@ docs/other.html @@ /how/ @@ ok site/how/index.html 2
"##;

#[test]
fn html_rungs_resolve_and_refuse() {
    text_ladder(Lang::Html, LADDER);
}

/// The site both end-to-end legs read, under a fresh scratch root: a
/// page naming a stylesheet, an image and a document, and an orphan
/// asset and an orphan page beside it.
fn site(name: &str) -> PathBuf {
    let dir = common::tmp(name);
    common::ladder::materialize(
        &dir,
        &[
            (
                "index.html",
                "<html><head><link rel=\"stylesheet\" href=\"style.css\"></head>\
                 <body><img src=\"logo.png\"><a href=\"docs/guide.md\">g</a></body></html>\n",
            ),
            ("style.css", "body {}\n"),
            ("logo.png", "png\n"),
            ("docs/guide.md", "# Guide\n"),
            ("orphan.png", "png\n"),
            ("orphan.html", "<html><body></body></html>\n"),
        ],
    );
    dir
}

/// The wire end to end: a page's asset and page references travel as
/// edges of their kinds (graph/wire.rs edge_kind), the assets nodes the
/// index never held; and the asset set is a resolve_key input — the
/// image deleted, the next build sweeps its edge away with no page
/// edited.
#[test]
fn a_page_draws_asset_and_page_edges_that_follow_the_asset_set() {
    let dir = site("ladder-html-wire");
    common::build_index(&dir);
    let edge = |(p, k): (&str, i64)| (p.to_string(), k);
    assert_eq!(
        edges_of(&dir),
        [
            ("docs/guide.md", EDGE_DOC_LINK),
            ("logo.png", EDGE_ASSET),
            ("style.css", EDGE_ASSET),
        ]
        .map(edge)
    );
    std::fs::remove_file(dir.join("logo.png")).expect("remove the asset");
    common::build_index(&dir);
    assert_eq!(
        edges_of(&dir),
        [("docs/guide.md", EDGE_DOC_LINK), ("style.css", EDGE_ASSET)].map(edge)
    );
}

/// The verdict end to end (plan v2.30 step 5): a page's stylesheet and
/// image are nodes the index never parsed, so `ce deadcode` must never
/// call them dead — the role the wire sends lands on the core's
/// dyn-referenced bit — while an orphan PAGE beside them is dead as
/// ever (the counterfactual: the silence is the role's, not the
/// family's), and an orphan asset is as silent as a referenced one.
/// The structure family, whose universe is the measured tier, must
/// still answer on such a tree: it refused the first asset node as
/// "outside the walked tree" before the tier excluded them.
#[test]
fn assets_are_never_dead_rows_and_never_measured() {
    let dir = site("ladder-html-dead");
    let dead = common::run_ce(&dir, &["deadcode", "--format", "json", "."]);
    let report: serde_json::Value = serde_json::from_slice(&dead.stdout).expect("deadcode json");
    let names: Vec<&str> = report["dead"]
        .as_array()
        .expect("dead rows")
        .iter()
        .map(|r| r["name"].as_str().expect("a dead row's name"))
        .collect();
    assert_eq!(
        names,
        ["orphan.html"],
        "{}",
        String::from_utf8_lossy(&dead.stderr)
    );
    let structure = common::run_ce(&dir, &["structure", "--format", "json", "."]);
    assert!(
        structure.status.success(),
        "{}",
        String::from_utf8_lossy(&structure.stderr)
    );
}

/// The wire's edges out of index.html as (target path, kind), sorted.
fn edges_of(dir: &Path) -> Vec<(String, i64)> {
    let (_idx, w) = common::graph_wire(dir, Advisory::No);
    let node = |i: i64| &w.nodes[usize::try_from(i).expect("a node index")];
    let page = w
        .nodes
        .iter()
        .position(|n| n.path == "index.html" && n.unit.is_empty())
        .expect("the page is a node");
    let mut edges: Vec<(String, i64)> = w
        .edges
        .iter()
        .filter(|e| e[0] == page as i64)
        .map(|e| (node(e[1]).path.clone(), e[2]))
        .collect();
    edges.sort();
    edges
}
