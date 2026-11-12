//! The graph screen's document on two fixture trees, one line per tree
//! in `it/golden/graph_screen.ndjson`. At plan v2.32 step 4B the file
//! was the b3443723 code's answer, kept to prove that moving the canvas
//! assembly into the core (CE.Graph.Screen) changed nothing; that proof
//! is done and stays in history, and the `rust` line is still that
//! answer byte for byte (this leg asserts nothing about it). It pins the
//! document as the code gives it today. Lane v235_licence part 4C moved
//! the `docs` line by its rule (a link to a confirmed section of another
//! file references that file): `docs/guide.md` is alive, README <->
//! guide is a cycle, the kept edges go 5 -> 6. The road: `CE_BLESS=1`
//! rewrites the file from `documents()` through the suite's one golden
//! writer, `common::assert_matches_golden`.

use crate::common;

const GOLDEN: &str = "cli/tests/it/golden/graph_screen.ndjson";

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
fn the_graph_screen_is_the_golden_document() {
    let path = common::repo_root().join(GOLDEN);
    let now = documents();
    if crate::facts::blessing() {
        return common::assert_matches_golden(now.trim_end(), &path);
    }
    let golden = std::fs::read_to_string(&path).expect("the graph screen golden");
    let (golden, now): (Vec<&str>, Vec<&str>) = (golden.lines().collect(), now.lines().collect());
    assert_eq!(golden.len(), now.len(), "one line per tree");
    for (tag, (a, b)) in ["rust", "docs"].iter().zip(golden.iter().zip(&now)) {
        assert_eq!(
            a, b,
            "{tag}: the graph screen moved off its golden \
             (CE_BLESS=1 cargo test --test it -- graph_screen_frozen rewrites it)"
        );
    }
}
