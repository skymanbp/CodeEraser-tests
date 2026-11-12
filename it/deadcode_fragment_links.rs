//! A link to a part of a file is a link to the file (lane v235_licence,
//! part 4C; the core's CE.Resolve.Answer `holderOf`): a site whose answer
//! is a confirmed section of ANOTHER file also references that file, at
//! the answer's rung — the section stays the answer. One fixture holds
//! each case of WORD-4C section 2: a live page's fragment link keeps the
//! page alive, a dead page's makes it a dependent, a page pointing into
//! itself stays a root, a broken fragment and an unused definition read
//! as before, a used reference link and an HTML `id` count like a link;
//! and the readers of stored edges (`ce query`'s `ref` facts, the graph
//! wire `ce merge` counts its indegree from) see the file-level edge.

use crate::common;
use crate::query_face::{json_face, values};
use codeeraser::graph::deadcode::{Advisory, GraphWire};

/// The fixture: a path on a `--- ` line, its text under it. README.md
/// and index.html are entries by name.
const FIXTURE: &str = "\
--- README.md
# Top

See [c](c.md#sec), [d](d.md) and [x][r].

[r]: h.md#sec
[unused]: i.md#sec
--- c.md
# C

## sec

text
--- d.md
# D

See [e](e.md#part).
--- e.md
# E

## part
--- f.md
# F

See [g](g.md#part).
--- g.md
# G

## part
--- toc.md
# Toc

[part](#part)

## part
--- me.md
# Me

[x](me.md#part)

## part
--- broken.md
# Broken

See [t](target.md#nosuch).
--- target.md
# Target
--- h.md
# H

## sec
--- i.md
# I

## sec
--- index.html
<html><body><a href=\"page2.html#s1\">two</a></body></html>
--- page2.html
<html><body><h1 id=\"s1\">S1</h1></body></html>
";

/// What `ce deadcode` names, `path family` (a root `unref`, a dependent
/// `unreach`), in path order: c.md, e.md, h.md and page2.html are kept
/// alive by a live page's fragment link; g.md falls with f.md.
const DEAD: &str = "\
broken.md unref
f.md unref
g.md unreach
i.md unref
me.md unref
target.md unreach
toc.md unref";

#[test]
fn a_fragment_link_to_another_file_references_that_file() {
    let dir = common::seeded_tree("deadcode-fragment-links", FIXTURE);
    let (code, doc) = json_face(&dir, &["deadcode"]);
    assert_eq!((code, doc["degraded"].is_null()), (Some(0), true), "{doc}");
    let rows = doc["dead"].as_array().expect("dead");
    let names = rows.iter().filter_map(|r| r["name"].as_str());
    let verdicts = rows.iter().filter_map(|r| r["verdict"].as_str());
    let mut got = String::new();
    for (name, verdict) in names.zip(verdicts) {
        let family = verdict.split('_').next().unwrap_or(verdict);
        got += &format!("{name} {family}\n");
    }
    assert_eq!(got.trim_end(), DEAD);
}

/// `(edges into the file node, edges into its section nodes)` of `path`
/// on the graph wire — the file-node count is what `ce merge` reads as a
/// file's indegree.
fn landing(wire: &GraphWire, path: &str) -> (usize, usize) {
    let mut seen = (0, 0);
    for [_, to, _, _] in &wire.edges {
        let node = &wire.nodes[*to as usize];
        match (node.path == path, node.unit.is_empty()) {
            (true, true) => seen.0 += 1,
            (true, false) => seen.1 += 1,
            _ => {}
        }
    }
    seen
}

#[test]
fn the_readers_of_stored_edges_see_the_file_level_reference() {
    let dir = common::seeded_tree("deadcode-fragment-refs", FIXTURE);
    let (code, doc) = json_face(&dir, &["query", "ref(F, G, K, R)"]);
    assert_eq!(code, Some(0), "{doc}");
    let mut refs = Vec::new();
    for row in values(&doc) {
        refs.push(row.join(" "));
    }
    // the file beside the section, at the answer's rung; the section stays
    let wanted = "README.md c.md doc_link 2|README.md c.md#sec doc_link 2|README.md h.md doc_link 3|\
                  index.html page2.html doc_link 2|index.html page2.html#s1 doc_link 2";
    for want in wanted.split('|') {
        assert!(refs.iter().any(|r| r == want), "{want} not in {refs:#?}");
    }
    // the graph `ce merge` counts a file's indegree from: one edge lands on
    // each page's file node beside the one on its section
    let (_idx, wire) = common::graph_wire(&dir, Advisory::No);
    // a page pointing into itself draws no edge to its own file
    let got: Vec<(usize, usize)> = ["c.md", "page2.html", "me.md", "toc.md"]
        .map(|p| landing(&wire, p))
        .to_vec();
    assert_eq!(
        got,
        [(1, 1), (1, 1), (0, 1), (0, 1)],
        "(file node, section node)"
    );
}
