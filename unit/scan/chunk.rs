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
    assert!(
        err.to_string().contains("must not straddle a chunk"),
        "{err}"
    );
}
