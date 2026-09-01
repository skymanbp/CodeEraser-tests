use super::*;
use crate::scan::metrics::{FileMetrics, FnMetrics};
use crate::scan::report::blocks_of;

/// A file of `units` functions, each measured at the same cognitive
/// value, carrying `calls` as its proved arcs. Only the fields the
/// row arithmetic reads carry meaning here.
fn file(path: &str, units: usize, cognitive: u32, calls: &[(u32, u32)]) -> FileMetrics {
    FileMetrics {
        path: path.into(),
        lang: "rust",
        total_lines: 10,
        comment_lines: 0,
        functions: (0..units)
            .map(|i| FnMetrics {
                name: format!("f{i}"),
                start_line: i + 1,
                end_line: i + 2,
                lines: 2,
                params: 0,
                cyclomatic: 1,
                cognitive,
                max_nesting: 0,
                name_ok: true,
                naming: [0, 1, 0, 0, 0],
            })
            .collect(),
        calls: calls.to_vec(),
    }
}

#[test]
fn an_arc_lands_on_its_units_cognitive_row() {
    // file 0 holds 1 + 6*2 = 13 rows, so file 1 starts at row 13.
    let files = [
        file("a.rs", 2, 3, &[(0, 1), (1, 1)]),
        file("b.rs", 1, 3, &[(0, 0)]),
    ];
    let blocks = blocks_of(&files);
    assert_eq!(blocks, vec![13, 7]);
    // unit j of a file at offset o answers at o + 1 + 6j + 3
    assert_eq!(
        arcs(&files, &blocks),
        vec![[4, 10], [10, 10], [17, 17]],
        "the arcs of one file never leave its own block"
    );
}

#[test]
fn identical_arcs_arrive_once_and_in_order() {
    let files = [file("a.rs", 2, 3, &[(1, 0), (0, 1), (1, 0)])];
    let blocks = blocks_of(&files);
    let out = arcs(&files, &blocks);
    assert_eq!(out, vec![[4, 10], [10, 4]]);
    assert!(
        out.windows(2).all(|w| w[0] < w[1]),
        "the core refuses a table that is not strictly ascending"
    );
}

#[test]
fn a_raised_value_reaches_the_function_it_names() {
    let mut files = [file("a.rs", 2, 3, &[]), file("b.rs", 1, 7, &[])];
    let blocks = blocks_of(&files);
    apply(&mut files, &blocks, &[[10, 4], [17, 8]]).expect("applied");
    let seen: Vec<u32> = files
        .iter()
        .flat_map(|f| &f.functions)
        .map(|f| f.cognitive)
        .collect();
    assert_eq!(seen, vec![3, 4, 8], "only the named rows moved");
}

/// Every shape the core could answer that this side must refuse
/// rather than write somewhere plausible.
#[test]
fn a_row_that_is_not_a_cognitive_seat_is_refused_by_name() {
    for (bumped, why) in [
        (vec![[9, 4]], "a fn-lines row, not a cognitive one"),
        (vec![[0, 4]], "the file row"),
        (vec![[13, 4]], "past the only file's rows"),
        (vec![[10, 2]], "a value below what was measured"),
    ] {
        let mut files = [file("a.rs", 2, 3, &[])];
        let blocks = blocks_of(&files);
        let err = apply(&mut files, &blocks, &bumped).expect_err(why);
        assert!(format!("{err}").contains("cocBumped row"), "{why}: {err}");
    }
}
