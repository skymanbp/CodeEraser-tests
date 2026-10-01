use super::*;
use crate::arch::wire::{ARCH_FILE_CAP, ARCH_REF_CAP, over_cap};
use crate::structure::tree;
use crate::testutil::node;

/// The graph as the index hands it over, one node per line in the
/// wire's node order (`path#unit kind`, `*` = an asset): four measured
/// files in three directories, a package in the tree and one outside
/// it, a section of the Markdown file, an image the page names.
const NODES: &str = "\
a/x.py# file
a/y.py# file
b# package
b/z.py# file
c/d.md# file
c/d.md#Intro section
img.png# file*
out# package";

/// `src dst kind rung` per arc: a/x → a/y; a/x → b/z twice (two rungs,
/// one pair of weight 2); a/x → itself; b/z → c/d.md's section (folds
/// onto the file); c/d.md → its own section (a self pair once folded);
/// a/y → package b and → package out (outside the tree); c/d.md → the
/// image; package b's containment arc to b/z (not a file's reference).
const ARCS: &str =
    "0 1 0 1; 0 3 0 1; 0 3 0 2; 0 0 0 1; 3 5 1 1; 4 5 1 1; 1 2 0 1; 1 7 0 1; 4 6 3 1; 2 3 4 1";

fn wire() -> GraphWire {
    let kind = |k: &str| match k {
        "package" => GRAN_PACKAGE,
        "section" => GRAN_SECTION,
        _ => GRAN_FILE,
    };
    let nodes = NODES
        .lines()
        .map(|l| {
            let (id, k) = l.split_once(' ').expect("path#unit kind");
            let (path, unit) = id.split_once('#').expect("path#unit");
            let mut n = node(path, unit, kind(k.trim_end_matches('*')));
            n.asset = k.ends_with('*');
            n
        })
        .collect();
    let edges = ARCS
        .split("; ")
        .map(|a| {
            let v: Vec<i64> = a.split(' ').map(|x| x.parse().expect("an int")).collect();
            [v[0], v[1], v[2], v[3]]
        })
        .collect();
    GraphWire {
        edges,
        nodes,
        symbols: Default::default(),
        unres: Vec::new(),
        rows: Vec::new(),
        unresolved_sites: 0,
        unmentioned: None,
        mounts: None,
        scc_floor: None,
    }
}

fn assembled(focus: &[&str]) -> Result<Tables> {
    let w = wire();
    let t = tree::build(&measured_paths(&w));
    let focus: Vec<String> = focus.iter().map(|s| s.to_string()).collect();
    assemble(&w, &t, &focus, &[10, 20, 30, 40])
}

#[test]
fn the_measured_files_are_dense_in_path_order_under_their_directories() {
    let t = assembled(&[]).expect("tables");
    assert_eq!(t.paths, ["a/x.py", "a/y.py", "b/z.py", "c/d.md"]);
    assert_eq!(t.dir_paths, ["", "a", "b", "c"]);
    assert_eq!(t.files, [[0, 1, 10], [1, 1, 20], [2, 2, 30], [3, 3, 40]]);
    assert_eq!(t.dirs, [[0, -1], [1, 0], [2, 0], [3, 0]]);
}

#[test]
fn arcs_fold_to_file_pairs_and_package_directories() {
    let t = assembled(&[]).expect("tables");
    // two rungs are one pair of weight 2, a self arc and a self pair
    // after the section fold are gone, the section counts as its file
    assert_eq!(t.edges, [[0, 1, 1], [0, 2, 2], [2, 3, 1]]);
    // the package in the tree is its directory; the one outside it,
    // the asset and the containment arc have no row
    assert_eq!(t.pkg_edges, [[1, 2, 1]]);
}

#[test]
fn the_focus_is_ascending_distinct_and_a_stranger_is_named() {
    let t = assembled(&["b/z.py", "a/x.py", "b/z.py"]).expect("tables");
    assert_eq!(t.focus, [0, 2]);
    for stranger in ["nope.py", "img.png", "b"] {
        let err = assembled(&[stranger]).err().expect("refused");
        assert_eq!(
            err.to_string(),
            format!("--impact: {stranger} is not a measured file")
        );
    }
}

#[test]
fn every_directory_parent_is_an_earlier_row() {
    let t = tree::build(&[
        "z/y/x/w.rs".into(),
        "a/b.rs".into(),
        "z/a.rs".into(),
        "m/n/o.rs".into(),
    ]);
    let rows = dir_rows(&t).expect("a tree");
    let names = dir_paths(&t);
    assert_eq!(rows[0], [0, -1]);
    for &[d, parent] in &rows[1..] {
        assert!(parent < d, "{rows:?}");
        let (child, up) = (&names[d as usize], &names[parent as usize]);
        let expected = child.rsplit_once('/').map_or("", |(head, _)| head);
        assert_eq!(up, expected, "{child}'s parent");
    }
}

#[test]
fn the_caps_refuse_before_the_core_by_name() {
    let mut t = assembled(&[]).expect("tables");
    assert_eq!(over_cap(&t), None);
    t.files = vec![[0, 0, 0]; ARCH_FILE_CAP + 1];
    let files = over_cap(&t).expect("files over the cap");
    assert!(files.starts_with("arch_too_large: 131073 files"), "{files}");
    t.files.truncate(ARCH_FILE_CAP);
    assert_eq!(over_cap(&t), None);
    // the two reference tables count together: at the cap is inside
    t.edges = vec![[0, 1, 1]; ARCH_REF_CAP - t.pkg_edges.len()];
    assert_eq!(over_cap(&t), None);
    t.pkg_edges.push([0, 1, 1]);
    let refs = over_cap(&t).expect("references over the cap");
    assert!(refs.contains("524289 references"), "{refs}");
}
