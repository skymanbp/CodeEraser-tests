use super::*;
use crate::testutil::four_node_wire;

/// The arcs off one wire: the two measured files in node order; an arc
/// between them once per kind and rung (two rungs of a → b are two
/// pairs); a file's arc to a section folded onto the section's file, kept
/// even when it is the arc's own file (the core drops self pairs, not
/// this reading); a file's arc to the package as the package's path; and
/// the package's containment arc, which no file makes, nowhere.
#[test]
fn arcs_read_the_files_sections_and_packages_off_the_wire() {
    let w = four_node_wire(&[
        [0, 1, 0, 1],
        [0, 1, 0, 2],
        [1, 2, 1, 1],
        [0, 2, 1, 1],
        [0, 3, 4, 1],
        [3, 1, 5, 1],
    ]);
    let a = arcs(&w);
    assert_eq!(a.paths, ["a.rs", "b.rs"]);
    assert_eq!(a.files, [[0, 1], [0, 1]]);
    assert_eq!(a.sections, [[0, 0], [1, 0]]);
    assert_eq!(a.packages, [(0, "pkg".to_string())]);
}

/// A foreign reader and a walked asset are no measured file: no slot, no
/// arc.
#[test]
fn foreign_and_asset_files_have_no_slot() {
    let mut w = four_node_wire(&[[0, 1, 0, 1], [1, 0, 0, 1]]);
    w.nodes[0].foreign = true;
    assert!(arcs(&w).files.is_empty());
    assert_eq!(arcs(&w).paths, ["b.rs"]);
    w.nodes[0].foreign = false;
    w.nodes[1].asset = true;
    assert_eq!(
        (arcs(&w).paths, arcs(&w).files),
        (vec!["a.rs".to_string()], vec![])
    );
}
