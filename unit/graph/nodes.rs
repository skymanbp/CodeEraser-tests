use super::*;

fn edge(src: &str, dst: &str, unit: &str, granularity: i64) -> GraphEdge {
    GraphEdge {
        src: src.into(),
        dst_path: dst.into(),
        dst_unit: unit.into(),
        kind: 0,
        rung: 1,
        granularity,
    }
}

/// Identity triples of the dense assignment for `files`/`edges`.
fn triples(files: &[String], edges: &[GraphEdge]) -> Vec<(String, String, i64)> {
    nodes_of(files, edges, &BTreeSet::new())
        .into_iter()
        .map(|n| (n.path, n.unit, n.kind))
        .collect()
}

/// The foreign mark lands on the foreign file, on a package UNDER
/// it (the submodule's own package rows are not this tree's to
/// report), on its sections — and never on the root package or an
/// own file that merely shares a prefix string.
#[test]
fn foreign_marks_the_file_its_packages_and_sections_only() {
    let g = crate::graph::wire::GRAN_PACKAGE;
    let files = vec![
        "src/a.rs".to_string(),
        "suite/it/t.rs".to_string(),
        "suite/README.md".to_string(),
        "suitemate.rs".to_string(),
    ];
    let edges = vec![
        edge("suite/it/t.rs", "suite/it", "", g),
        edge("suite/it/t.rs", "", "", g),
        edge("src/a.rs", "suite/README.md", "Intro", 2),
        edge("src/a.rs", "src", "", g),
    ];
    let foreign: BTreeSet<String> = ["suite/it/t.rs", "suite/README.md"]
        .map(String::from)
        .into();
    let by: std::collections::BTreeMap<(String, String), bool> = nodes_of(&files, &edges, &foreign)
        .into_iter()
        .map(|n| ((n.path, n.unit), n.foreign))
        .collect();
    let want = [
        ("src/a.rs", "", false),
        ("suite/it/t.rs", "", true),
        ("suite/README.md", "", true),
        ("suite/README.md", "Intro", true),
        ("suite/it", "", true),
        ("suitemate.rs", "", false),
        ("src", "", false),
        ("", "", false),
    ];
    for (path, unit, f) in want {
        assert_eq!(
            by[&(path.to_string(), unit.to_string())],
            f,
            "{path}#{unit}"
        );
    }
}

/// The id assignment is a function of the graph, not of input
/// order — reversing the inputs yields the identical node list.
#[test]
fn assignment_is_shuffle_proof() {
    let g = crate::graph::wire::GRAN_PACKAGE;
    let files = vec!["a.rs".to_string(), "lib/b.rs".to_string()];
    let edges = vec![
        edge("a.rs", "lib", "", g),
        edge("a.rs", "doc.md", "intro", 2),
    ];
    let mut rev_files = files.clone();
    rev_files.reverse();
    let rev_edges = vec![
        edge("a.rs", "doc.md", "intro", 2),
        edge("a.rs", "lib", "", g),
    ];
    assert_eq!(triples(&files, &edges), triples(&rev_files, &rev_edges));
    // containment arcs (named AND root package) live in
    // tests/graph_containment.rs — behaviour, not identity
}

/// An absent-file target is a FILE node unless an edge's stored
/// granularity SAYS package: the asset / dangling-ref shapes the
/// old absence-inference minted as packages (review LOW).
#[test]
fn absent_targets_are_files_unless_an_edge_says_package() {
    let files = vec!["doc.md".to_string()];
    let edges = vec![
        edge("doc.md", "art/logo.png", "", 0),
        edge("doc.md", "gone.md", "", 0),
        edge("doc.md", "pkg", "", crate::graph::wire::GRAN_PACKAGE),
    ];
    let by: std::collections::BTreeMap<String, i64> = nodes_of(&files, &edges, &Default::default())
        .into_iter()
        .map(|n| (n.path, n.kind))
        .collect();
    assert_eq!(by["art/logo.png"], crate::graph::wire::GRAN_FILE);
    assert_eq!(by["gone.md"], crate::graph::wire::GRAN_FILE);
    assert_eq!(by["pkg"], crate::graph::wire::GRAN_PACKAGE);
}
