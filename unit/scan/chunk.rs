//! The scan/1 chunk split battery, mounted beside its module: which
//! rows travel together, and why a file may never be cut in half.

use super::*;
use crate::scan::wire::ScanRequest;
use serde_json::Value;

/// Only the four streams the split walks; the rest of a request
/// never reaches it.
fn plan_req<'a>(
    rows: &'a [[u64; 2]],
    naming: &'a [[i64; 5]],
    blocks: &'a [usize],
    calls: &'a [[u64; 2]],
) -> ScanRequest<'a> {
    ScanRequest {
        rows,
        grades: &[],
        naming,
        row_classes: None,
        overrides: &[],
        fence: Value::Null,
        blocks,
        calls,
    }
}

/// The chunk budget pays 1 per row + 1 per riding naming fact + 1
/// per arc, so chunk + grades always fits the core's cap: with
/// budget 3 and one file per pair, [plain, code-6][plain, code-6]
/// splits between the files (1+2 each), each facts slice follows its
/// own rows, and the row span the class column is sliced by follows
/// the same cut.
#[test]
fn chunk_plan_counts_every_request_dimension() {
    let rows = [[0u64, 1], [6, 0], [0, 2], [6, 0]];
    let naming = [[4i64, 2, 0, 1, 1], [1, 2, 0, 1, 1]];
    let blocks = [2usize, 2];
    let cuts = plan(&plan_req(&rows, &naming, &blocks, &[]), 3).expect("each file fits");
    let shape: Vec<_> = cuts
        .iter()
        .map(|c| (c.rows, c.naming, c.span.clone()))
        .collect();
    assert_eq!(
        shape,
        vec![
            (&rows[..2], &naming[..1], 0..2),
            (&rows[2..], &naming[1..], 2..4)
        ]
    );
    // an empty scan still sends ONE (empty, legal) request
    let empty = plan(&plan_req(&[], &[], &[], &[]), 3).expect("empty fits");
    assert_eq!(empty.len(), 1);
    assert!(empty[0].rows.is_empty() && empty[0].span == (0..0));
}

/// The class column is the sixth dimension `CE.Scan.overCap` sums,
/// and it is one entry per row — so a classed run buys half as many
/// rows per chunk. Before this was priced, a classed tree sent the
/// core a request the core answered `degraded`, which wire.rs turns
/// into a hard "cap mirror drift" error the user cannot act on.
#[test]
fn a_class_column_costs_a_row_its_own_seat() {
    let rows = [[0u64, 1]; 4];
    let blocks = [2usize, 2];
    let classes = [0u64; 4];
    let bare = plan_req(&rows, &[], &blocks, &[]);
    assert_eq!(
        plan(&bare, 4).expect("unclassed fits").len(),
        1,
        "4 rows, budget 4: one chunk"
    );
    let mut classed = plan_req(&rows, &[], &blocks, &[]);
    classed.row_classes = Some(&classes);
    let cuts = plan(&classed, 4).expect("each file still fits");
    assert_eq!(
        cuts.iter().map(|c| c.span.clone()).collect::<Vec<_>>(),
        vec![0..2, 2..4],
        "the same 4 rows weigh 8 with a class column"
    );
    // and a single file whose classed weight passes the budget is
    // refused by name rather than sent to be degraded
    let one = {
        let mut r = plan_req(&rows, &[], &[4], &[]);
        r.row_classes = Some(&classes);
        r
    };
    let err = plan(&one, 5).err().expect("4 classed rows weigh 8");
    assert!(
        err.to_string().contains("must not straddle a chunk"),
        "{err}"
    );
}

/// The invariant the call table needs (6.5.0): a boundary falls
/// BETWEEN files, never inside one. Without it an arc stated in row
/// indices would be cut in half by the split and silently lost.
#[test]
fn every_chunk_boundary_is_a_file_boundary() {
    let rows = [[0u64, 1]; 6];
    let blocks = [2usize, 2, 2];
    for budget in [2, 3, 4, 5, 6] {
        let cuts = plan(&plan_req(&rows, &[], &blocks, &[]), budget).expect("fits");
        let seams: Vec<usize> = cuts.iter().map(|c| c.span.start).collect();
        assert!(
            seams.iter().all(|s| [0, 2, 4].contains(s)),
            "budget {budget} cut inside a file: {seams:?}"
        );
    }
}

/// And so every arc survives the split whole, rebased onto the rows
/// of the chunk that carries it.
#[test]
fn no_call_edge_crosses_a_chunk() {
    let rows = [[0u64, 1], [4, 9], [1, 3], [0, 1], [4, 9], [1, 3]];
    let blocks = [3usize, 3];
    let calls = [[1u64, 1], [4, 4]];
    let cuts = plan(&plan_req(&rows, &[], &blocks, &calls), 4).expect("each file fits");
    assert_eq!(cuts.len(), 2, "one file per chunk at budget 4");
    for chunk in &cuts {
        assert_eq!(chunk.calls, vec![[1, 1]], "rebased onto its own rows");
        assert!(
            chunk
                .calls
                .iter()
                .all(|a| a.iter().all(|&e| (e as usize) < chunk.rows.len())),
            "an endpoint left its chunk"
        );
    }
}

/// A file that cannot fit a chunk on its own is refused by name.
/// Splitting it would drop the arcs crossing the cut, and a lost arc
/// is a lost cycle — an undercount nothing would ever report.
#[test]
fn a_file_past_the_budget_is_refused_by_name() {
    let rows = [[0u64, 1]; 5];
    let err = plan(&plan_req(&rows, &[], &[5], &[]), 3)
        .err()
        .expect("one file, no room");
    let said = err.to_string();
    assert!(said.contains("must not straddle a chunk"), "{said}");
    // a deliberate refusal must not READ like a broken build: this
    // literal once carried the 14 spaces a lost `\` continuation
    // leaves behind, and no gate could see it
    assert!(
        !said.contains("  "),
        "run of spaces in the refusal: {said:?}"
    );
}
