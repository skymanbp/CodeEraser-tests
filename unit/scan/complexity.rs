use super::*;
use crate::scan::metrics::vocab::{CHAIN, DIRECT, Event, LOGIC_ROOT, NESTING};
use crate::scan::metrics::{FileMetrics, FnMetrics};
use crate::scan::report::blocks_of;

/// A file of `units` functions carrying `calls` as its proved arcs and
/// `events` in each unit — the two tables this module projects. The
/// three complexity numbers start at 0, as `measure` leaves them; only
/// the fields the row arithmetic reads carry meaning here.
fn file(path: &str, units: usize, calls: &[(u32, u32)], events: &[Event]) -> FileMetrics {
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
                cyclomatic: 0,
                cognitive: 0,
                max_nesting: 0,
                name_ok: true,
                naming: [0, 1, 0, 0, 0],
                events: events.to_vec(),
            })
            .collect(),
        calls: calls.to_vec(),
    }
}

/// An event with no operand count and no operators — what most rows
/// here need; the rest are struct-updated where they are built.
fn event(seq: u32, parent: Option<u32>, pos: u8, flags: u16) -> Event {
    Event {
        seq,
        parent,
        pos,
        flags,
        aux: 0,
        ops: Vec::new(),
    }
}

#[test]
fn an_arc_lands_on_its_units_cognitive_row() {
    // file 0 holds 1 + 6*2 = 13 rows, so file 1 starts at row 13.
    let files = [
        file("a.rs", 2, &[(0, 1), (1, 1)], &[]),
        file("b.rs", 1, &[(0, 0)], &[]),
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
    let files = [file("a.rs", 2, &[(1, 0), (0, 1), (1, 0)], &[])];
    let blocks = blocks_of(&files);
    let out = arcs(&files, &blocks);
    assert_eq!(out, vec![[4, 10], [10, 4]]);
    assert!(
        out.windows(2).all(|w| w[0] < w[1]),
        "the core refuses a table that is not strictly ascending"
    );
}

/// A unit's events ride keyed by its cognitive row — the same seat
/// the arcs land on — in (row, seq) order, the parent spelled -1 for
/// the unit itself and a logic root's operator ids trailing; a unit
/// without structure sends no row at all.
#[test]
fn every_event_rides_keyed_by_its_units_cognitive_row() {
    let structure = [
        event(0, None, 0, NESTING),
        Event {
            aux: 2,
            ..event(1, Some(0), 1, CHAIN | DIRECT)
        },
    ];
    let files = [
        file("a.rs", 2, &[], &structure),
        file(
            "b.rs",
            1,
            &[],
            &[Event {
                ops: vec![0, 1],
                ..event(0, None, 0, LOGIC_ROOT)
            }],
        ),
    ];
    let blocks = blocks_of(&files);
    let rows = events(&files, &blocks);
    assert_eq!(
        rows,
        vec![
            vec![4, 0, -1, 0, 1, 0],
            vec![4, 1, 0, 1, 544, 2],
            vec![10, 0, -1, 0, 1, 0],
            vec![10, 1, 0, 1, 544, 2],
            vec![17, 0, -1, 0, 256, 0, 0, 1],
        ]
    );
    // (row, seq) is unique per row, so the table's own order is the
    // sorted one exactly when it ascends by that pair
    let mut sorted = rows.clone();
    sorted.sort();
    assert_eq!(rows, sorted, "ascending by (row, seq), the core's order");
    let quiet = [file("q.rs", 1, &[], &[])];
    assert!(events(&quiet, &blocks_of(&quiet)).is_empty());
}

/// The per-unit triples the core answers for two files of two and
/// one units: `[row, value]` for the cyclomatic, cognitive and
/// nesting rows in row order (seats 3, 9 and 16 open each block).
const TRIPLES: [[u64; 3]; 3] = [[2, 3, 1], [4, 5, 2], [1, 0, 0]];

fn derived_rows() -> Vec<[u64; 2]> {
    [3u64, 9, 16]
        .iter()
        .zip(TRIPLES)
        .flat_map(|(&seat, t)| (0..3).map(move |k| [seat + k, t[k as usize]]))
        .collect()
}

fn two_files() -> [FileMetrics; 2] {
    [file("a.rs", 2, &[], &[]), file("b.rs", 1, &[], &[])]
}

#[test]
fn the_derived_values_reach_the_three_rows_of_every_unit() {
    let mut files = two_files();
    let blocks = blocks_of(&files);
    apply(&mut files, &blocks, &derived_rows(), &[[10, 5]]).expect("applied");
    let seen: Vec<[u32; 3]> = files
        .iter()
        .flat_map(|f| &f.functions)
        .map(|f| [f.cyclomatic, f.cognitive, f.max_nesting])
        .collect();
    assert_eq!(
        seen,
        TRIPLES.map(|t| t.map(|v| v as u32)),
        "each unit carries the triple the core derived, and the cocBumped row agreeing with it passes"
    );
}

/// Every shape the core could answer that this side must refuse
/// rather than write somewhere plausible: a table short of a unit's
/// three rows or out of order, a row on no complexity seat (the file
/// row, a naming row, past the rows), and a cocBumped row disagreeing
/// with the derived value at its index, landing on a cyclomatic seat
/// or on no seat at all.
#[test]
fn a_table_that_is_not_every_units_three_rows_is_refused_by_name() {
    let blocks = blocks_of(&two_files());
    let full = derived_rows();
    let mut short = full.clone();
    short.pop();
    let mut disordered = full.clone();
    disordered.swap(0, 1);
    for (table, why) in [
        (short, "a unit left without its three"),
        (disordered, "out of order"),
    ] {
        let err = apply(&mut two_files(), &blocks, &table, &[]).expect_err(why);
        assert!(
            err.to_string().contains("every unit answers three"),
            "{why}: {err}"
        );
    }
    for i in [0u64, 6, 20] {
        let mut seat = full.clone();
        seat[0] = [i, 1];
        let err = apply(&mut two_files(), &blocks, &seat, &[]).expect_err("no complexity seat");
        assert!(
            err.to_string().contains(&format!("derived row {i}")),
            "{err}"
        );
    }
    for [i, v] in [[10u64, 6], [9, 4], [0, 4]] {
        let err = apply(&mut two_files(), &blocks, &full, &[[i, v]])
            .expect_err("cocBumped disagrees with the derived value or seats nowhere");
        assert!(
            err.to_string().contains(&format!("cocBumped row {i}")),
            "{err}"
        );
    }
}
