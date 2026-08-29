use super::*;

/// One synthetic geometry, every count hand-checked: files 0,1
/// live in dir 0, file 2 in dir 1; edges 0→1 (intra), 0→2 and
/// 1→2 (inter), 2→0 (inter, reverse direction stays distinct).
#[test]
fn aggregate_counts_a_small_geometry_by_hand() {
    let file_dirs = [0, 0, 1];
    let edges = [(0, 1), (0, 2), (1, 2), (2, 0)];
    // file 0: intra 0→1 (+1 inside) + inter 0→2, 2→0 (+2 outside)
    // file 1: intra (+1 inside) + inter 1→2 (+1 outside)
    // file 2: three inter touches, zero inside
    assert_eq!(aggregate(&edges, &file_dirs), vec![[1, 2], [1, 1], [0, 3]]);
}
