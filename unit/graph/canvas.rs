use super::super::deadcode::{GraphWire, Report};
use super::super::wire::{GRAN_FILE, GRAN_PACKAGE, GRAN_SECTION};
use super::*;
use crate::testutil::node;

/// Two files, one section of a.rs, one package; b.rs judged dead.
fn fixture() -> (GraphWire, Report) {
    let nodes = vec![
        node("a.rs", "", GRAN_FILE),
        node("b.rs", "", GRAN_FILE),
        node("a.rs", "Intro", GRAN_SECTION),
        node("pkg", "", GRAN_PACKAGE),
    ];
    let edges = [[2, 1, 0, 0], [0, 1, 5, 0], [3, 0, 0, 0], [2, 0, 0, 0]]
        .into_iter()
        .collect();
    let w = GraphWire {
        nodes,
        rows: vec![],
        edges,
        unresolved_sites: 7,
        unres: vec![],
        // the canvas draws the graph, it never reads the export
        // surface or the advisory — an empty table and the
        // road-not-asked None are this fixture's whole claim
        symbols: Default::default(),
        unmentioned: None,
        mounts: None,
        scc_floor: None,
    };
    let report = Report {
        dead: vec![crate::graph::deadcode::DeadRow {
            path: "b.rs".into(),
            verdict: "unref_private",
            why: "no kept in-edge and no entry flag".into(),
            conf: Some(2),
        }],
        reported: vec![],
        nodes: 4,
        files: 4,
        kept: 3,
        unresolved_sites: 7,
        degraded: None,
        fail: true,
        unmentioned: None,
    };
    (w, report)
}

#[test]
fn sections_collapse_packages_drop_and_cycles_count() {
    let (w, report) = fixture();
    let pos = HashMap::from([
        ("a.rs".into(), [1, 2, 0, 1, 0]),
        ("b.rs".into(), [2, 0, 4, 2, 1]),
    ]);
    // the core reports SCC 4 = {b.rs, pkg}; the canvas counts it
    // because it holds a file, and the row carries the membership
    let cycles = file_cycles(&json!({ "cycles": [[4, [1, 3]]] }), &w).expect("cycles");
    let doc = document(&w, &report, &pos, &cycles);
    assert_eq!(doc["edges"], json!([[0, 1]]));
    assert_eq!(doc["counts"]["files"], 2);
    assert_eq!(doc["counts"]["edges"], 1);
    assert_eq!(doc["counts"]["dead"], 1);
    assert_eq!(doc["counts"]["cycles"], 1);
    assert_eq!(doc["files"][0]["cycle"], false);
    assert_eq!(doc["files"][1]["cycle"], true);
    assert_eq!(doc["files"][0]["verdict"], Value::Null);
    assert_eq!(doc["files"][0]["pos"], json!([1, 2, 0, 1, 0]));
    // a live file carries no trust column either: absence is null,
    // never a fabricated 0, which on this scale means UNVOUCHED
    assert_eq!(doc["files"][0]["conf"], Value::Null);
    assert_eq!(doc["files"][1]["verdict"], "unref_private");
    assert_eq!(doc["files"][1]["why"], "no kept in-edge and no entry flag");
    assert_eq!(doc["files"][1]["conf"], 2);
    assert_eq!(doc["files"][1]["pos"], json!([2, 0, 4, 2, 1]));
    assert_eq!(doc["unresolvedSites"], 7);
    assert_eq!(doc["degraded"], Value::Null);
    assert_eq!(doc["schema"], SCHEMA_ID);
}

/// The floor is never re-derived here: a reported SCC counts iff
/// it holds a FILE (a section-only SCC is the core's to report and
/// this tier's to ignore), the degraded reply's empty list counts
/// zero, and a member outside our node list is refused.
#[test]
fn file_cycles_take_the_core_report_and_restrict_it_to_files() {
    let (w, _) = fixture();
    let c = file_cycles(&json!({ "cycles": [[4, [1, 3]], [5, [2]]] }), &w).expect("cycles");
    assert_eq!(c.count, 1, "the section-only SCC is not a file-tier cycle");
    assert_eq!(c.files, BTreeSet::from(["b.rs".to_string()]));
    let none = file_cycles(&json!({ "cycles": [] }), &w).expect("degraded reply");
    assert_eq!((none.count, none.files.len()), (0, 0));
    assert!(file_cycles(&json!({ "cycles": [[0, [9]]] }), &w).is_err());
    assert!(
        file_cycles(&json!({}), &w).is_err(),
        "a reply without the key is malformed"
    );
}
