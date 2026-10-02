//! The graph screen's document, frozen (plan v2.32 step 4B): before
//! the canvas assembly moved into the core (CE.Graph.Screen), the
//! b3443723 code's `faces::graph_screen` over the two trees below was
//! written to `it/golden/graph_screen.b3443723.ndjson`, one line per
//! tree; the document the core lays out today must be that file byte
//! for byte. Nothing here writes it: the file is the old code's
//! answer, and only a checkout of the code its name records (with that
//! tree's core) can answer it again.

use crate::common;

const GOLDEN: &str = "cli/tests/it/golden/graph_screen.b3443723.ndjson";

/// A crate with a cycle, a file no module declares, an unspelled
/// public function and a library the binary reaches.
const RUST: &str = "\
--- Cargo.toml
[package]
name = \"screen\"
version = \"0.1.0\"
edition = \"2021\"
--- src/main.rs
mod a;
mod b;
fn main() {
    a::run();
}
--- src/a.rs
pub fn run() {
    crate::b::step();
}
pub fn never_spelled() -> u32 {
    7
}
--- src/b.rs
pub fn step() {
    crate::a::run();
}
--- src/orphan.rs
pub fn lost() {}
";

/// Pages that link one another by section, a page nothing links, a
/// stylesheet a page names, and a directory a page links whole.
const DOCS: &str = "\
--- README.md
# Screen

See [the guide](docs/guide.md#install) and [the kit](kit/).
--- docs/guide.md
# Guide

## Install

Back to [the top](../README.md).
--- docs/orphan.md
# Nobody links here
--- kit/tool.md
# Tool
--- site/index.html
<!doctype html>
<html><head><link rel=\"stylesheet\" href=\"style.css\"></head>
<body><a href=\"../README.md\">readme</a></body></html>
--- site/style.css
body { margin: 0; }
";

/// Each tree's graph screen document, one compact line per tree.
fn documents() -> String {
    let core = common::core_bin();
    let mut out = String::new();
    for (tag, tree) in [("rust", RUST), ("docs", DOCS)] {
        let dir = common::tmp(&format!("graph-screen-frozen-{tag}"));
        common::write_doc(&dir, tree);
        let doc = codeeraser::faces::graph_screen(&dir, &core).expect(tag);
        out.push_str(&serde_json::json!({"tree": tag, "document": doc}).to_string());
        out.push('\n');
    }
    out
}

#[test]
fn the_graph_screen_is_the_frozen_document() {
    let path = common::repo_root().join(GOLDEN);
    let now = documents();
    let frozen = std::fs::read_to_string(&path).expect("the frozen graph screen documents");
    let (frozen, now): (Vec<&str>, Vec<&str>) = (frozen.lines().collect(), now.lines().collect());
    assert_eq!(frozen.len(), now.len(), "one line per tree");
    for (tag, (a, b)) in ["rust", "docs"].iter().zip(frozen.iter().zip(&now)) {
        assert_eq!(
            a, b,
            "{tag}: the graph screen moved off the frozen document"
        );
    }
}
