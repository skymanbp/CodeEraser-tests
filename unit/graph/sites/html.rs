//! HTML's attribute-site pass pinned (plan v2.30 step 5): which
//! (element, attribute) pairs open a site and under which label, the
//! `<link>` relationships that make one an asset, and where a `srcset`
//! candidate or a wrapped value anchors its site.

use super::{asset_rel, label_of, sites};
use crate::scan::ast;
use crate::scan::lang::Lang;

/// `rel ⇒ asset|page` per row: the relationships under which a
/// `<link href>` fetches a resource, and those under which it names
/// another page — a token list, any case, the empty one a page.
const RELS: &str = "\
stylesheet ⇒ asset
preload ⇒ asset
modulepreload ⇒ asset
prefetch ⇒ asset
manifest ⇒ asset
icon ⇒ asset
shortcut icon ⇒ asset
apple-touch-icon ⇒ asset
mask-icon ⇒ asset
STYLESHEET ⇒ asset
alternate stylesheet ⇒ asset
canonical ⇒ page
alternate ⇒ page
next ⇒ page
noopener ⇒ page
 ⇒ page";

#[test]
fn a_link_is_an_asset_under_the_fetching_relationships() {
    for row in RELS.lines() {
        let (rel, want) = row.split_once(" ⇒ ").expect("rel ⇒ verdict");
        let asset = want == "asset";
        assert_eq!(asset_rel(rel), asset, "{row:?}");
        let label = if asset { "link_asset" } else { "href" };
        assert_eq!(label_of("link", "href", Some(rel)), Some(label), "{row:?}");
    }
    assert_eq!(
        label_of("link", "href", None),
        Some("href"),
        "no rel: a page"
    );
}

/// `element attribute ⇒ label` per row, `-` for no site: the pairs the
/// table names, and the neighbours it does not — a non-URL attribute,
/// a URL attribute on an element that has none, `srcdoc` (markup).
const PAIRS: &str = "\
a href ⇒ href
area href ⇒ href
base href ⇒ href
use href ⇒ href
form action ⇒ action
script src ⇒ src
img src ⇒ src
img srcset ⇒ srcset
source srcset ⇒ srcset
iframe src ⇒ src
video poster ⇒ src
object data ⇒ src
a rel ⇒ -
img alt ⇒ -
div href ⇒ -
iframe srcdoc ⇒ -
meta content ⇒ -";

#[test]
fn only_the_table_pairs_open_a_site() {
    for row in PAIRS.lines() {
        let [element, attr, _, want] = row.split(' ').collect::<Vec<_>>()[..] else {
            panic!("{row:?}: element attribute ⇒ label")
        };
        let want = (want != "-").then_some(want);
        assert_eq!(label_of(element, attr, None), want, "{row:?}");
    }
}

/// The sites of one fragment as `kind@line=spec` rows.
fn found(html: &str) -> Vec<String> {
    ast::with_tree(html, Lang::Html, |tree| {
        sites(tree.root_node(), html.as_bytes())
            .iter()
            .map(|s| format!("{}@{}={}", s.kind, s.line, s.spec))
            .collect()
    })
}

/// A `srcset` opens one site per candidate, each on the line its URL
/// is written on, a trailing comma none; a value that opens on the
/// line after its quote anchors its site where the spec is written;
/// a valueless attribute is an empty site on the tag's line.
#[test]
fn srcset_and_wrapped_values_anchor_where_their_spec_is_written() {
    let html =
        "<img srcset=\"a.png 1x,\n  b.png 2x,\n\">\n<a href=\"\n  x.html\">x</a>\n<a href>v</a>\n";
    assert_eq!(
        found(html),
        [
            "srcset@1=a.png",
            "srcset@2=b.png",
            "href@5=x.html",
            "href@6="
        ]
    );
}
