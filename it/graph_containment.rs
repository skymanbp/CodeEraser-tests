//! Package→member containment arcs. Containment is a FACT the graph
//! asserts (rung 1), so a package that contains nothing makes its own
//! files look unreferenced — and `ce deadcode` then reports them dead.
//!
//! The root case is the one that shipped broken: a package at the
//! REPO ROOT has path "", the prefix was built as `format!("{}/",
//! path)` = "/", and no repo-relative member starts with that. Live
//! repro before the fix — a Go module whose `go.mod` sits at the root:
//!
//!     dead: lib.go  unref_private  (no kept in-edge and no entry flag)
//!
//! with `lib.go` imported by `cmd/main.go` (review 2026-08-19, codex
//! lane). Verified against a real tree, pinned here as a unit fact.

use codeeraser::graph::load::GraphEdge;
use codeeraser::graph::nodes::{contain, ids, nodes_of};
use codeeraser::graph::wire::{EDGE_CONTAIN, GRAN_PACKAGE};
use std::collections::{BTreeMap, BTreeSet};

fn pkg_edge(src: &str, dst: &str) -> GraphEdge {
    GraphEdge {
        src: src.into(),
        dst_path: dst.into(),
        dst_unit: String::new(),
        kind: 0,
        rung: 1,
        granularity: GRAN_PACKAGE,
    }
}

/// Does `contain` derive a package→member arc for this pair, under
/// the declared package code `code` (root → files)?
fn arc(
    files: &[String],
    edges: &[GraphEdge],
    code: &BTreeMap<String, BTreeSet<String>>,
    pkg: &str,
    member: &str,
) -> bool {
    let nodes = nodes_of(files, edges, &Default::default());
    let ids = ids(&nodes);
    let mut wire = BTreeSet::new();
    contain(&nodes, &ids, code, &mut wire);
    let (p, m) = (ids[&(pkg, "")] as i64, ids[&(member, "")] as i64);
    wire.contains(&[p, m, EDGE_CONTAIN, 1])
}

/// A table, because the two rows differ only in the package's PATH —
/// and the empty one is the whole finding.
#[test]
fn every_package_contains_the_files_under_it_including_the_root() {
    for (src, pkg, member, why) in [
        ("a.rs", "lib", "lib/b.rs", "a named package"),
        ("cmd/main.go", "", "lib.go", "the ROOT package"),
    ] {
        let files = vec![member.to_string(), src.to_string()];
        let edges = vec![pkg_edge(src, pkg)];
        assert!(
            arc(&files, &edges, &BTreeMap::new(), pkg, member),
            "{why} contains {member}"
        );
    }
}

/// A package whose code is declared (an R package's DESCRIPTION, plan
/// v2.30 step 4) reaches that code and nothing else under its
/// directory — at the tree root too, where "every file under it" is
/// the whole tree: `library(pkg)` runs `R/`, not NEWS.md.
#[test]
fn a_declared_package_contains_its_code_only() {
    let files: Vec<String> = ["R/a.R", "NEWS.md", "tests/testthat/test-a.R", "sub/R/x.R"]
        .map(String::from)
        .into();
    let edges = vec![
        pkg_edge("tests/testthat/test-a.R", ""),
        pkg_edge("tests/testthat/test-a.R", "sub"),
    ];
    let code: BTreeMap<String, BTreeSet<String>> = [
        ("".to_string(), ["R/a.R".to_string()].into()),
        ("sub".to_string(), BTreeSet::new()),
    ]
    .into();
    for (pkg, member, held, why) in [
        ("", "R/a.R", true, "the declared code"),
        (
            "",
            "NEWS.md",
            false,
            "a file under the root that is not the code",
        ),
        (
            "",
            "tests/testthat/test-a.R",
            false,
            "the test that attaches the package",
        ),
        (
            "",
            "sub/R/x.R",
            false,
            "another package's code under the root",
        ),
        (
            "sub",
            "sub/R/x.R",
            false,
            "a package whose declared code is empty",
        ),
    ] {
        assert_eq!(arc(&files, &edges, &code, pkg, member), held, "{why}");
    }
}
