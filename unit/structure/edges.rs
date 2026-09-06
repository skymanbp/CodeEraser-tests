use super::*;

/// One synthetic geometry, every count hand-checked: files 0,1
/// live in dir 0, file 2 in dir 1; edges 0→1 (intra), 0→2 and
/// 1→2 (inter), 2→0 (inter, reverse direction stays distinct).
/// The same geometry proves the two tables are ONE graph, which is
/// what lets the core read the intra mass off `inside` instead of
/// asking for it a second time: dir 0's inside sum is 2 = twice its
/// one intra edge, and its outside sum is 3 = the dir-edge mass
/// incident to it (2 out + 1 in).
#[test]
fn aggregate_counts_a_small_geometry_by_hand() {
    let file_dirs = [0, 0, 1];
    let edges = [(0, 1), (0, 2), (1, 2), (2, 0)];
    // file 0: intra 0→1 (+1 inside) + inter 0→2, 2→0 (+2 outside)
    // file 1: intra (+1 inside) + inter 1→2 (+1 outside)
    // file 2: three inter touches, zero inside
    let files = aggregate(&edges, &file_dirs);
    assert_eq!(files, vec![[1, 2], [1, 1], [0, 3]]);
    // direction preserved, ascending keys, intra edges absent
    assert_eq!(
        directed(&edges, &file_dirs),
        vec![[0, 1, 2], [1, 0, 1]],
        "the crossing table carries direction and nothing intra"
    );
    let (inside0, outside0) = (files[0][0] + files[1][0], files[0][1] + files[1][1]);
    assert_eq!((inside0, outside0), (2, 3), "the cross-table law, by hand");
}
