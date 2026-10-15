//! The moves/1 plan meets the protocol's line (plan v2.33 step 11b item
//! 2). A line row `[content, fnv1a, width]` is ~40 bytes on the wire with
//! its changed index, so the line is crossed below the row cap the
//! packing used to stop at; the core then answered the id-less
//! `too_large` and the daemon read it as a dead link. The fixture is
//! sized from the two ceilings the plan packs by (`judged::line_max`,
//! `limits.moves.line_cap`), not from literals: fixed 500,000-line
//! halves sized for the 33,554,432-byte line fit one body once the line
//! became 134,217,728 bytes (30a56389), and the leg read `1 != 2`
//! (1.9.2 T1). The changeset's rows sit midway between where the line
//! is crossed and the row cap, so only the line can cut it: two added
//! files of half those rows each, and one added file of all of them.
//! Ignored by default: ~1 GB of request values; run under the lanes'
//! heavy lock.

use crate::common::core_bin;
use codeeraser::corelink::{Link, judged};
use codeeraser::fourclass::batch::{PairInput, classify_batch};
use codeeraser::fourclass::moves;
use codeeraser::scan::lang::Lang;

fn added(lines: usize) -> String {
    (0..lines).map(|i| format!("v{i} = {i}\n")).collect()
}

fn pair(after: &str) -> PairInput<'_> {
    PairInput {
        before: "",
        after,
        lang: Lang::Python,
    }
}

/// The changeset's rows: midway between the rows at which the line is
/// crossed (bytes per row measured on a probe pair) and the row cap.
fn rows_between_the_line_and_the_row_cap() -> usize {
    const PROBE: usize = 200_000;
    let probe = added(PROBE);
    let (_, bodies) = moves::request(&[pair(&probe)]);
    let per_row = judged::line_len(moves::KIND, &bodies[0]).div_ceil(PROBE);
    let by_line = judged::line_max() / per_row;
    let cap = codeeraser::tables::get().limits.moves.line_cap;
    assert!(
        by_line < cap,
        "the line is crossed at ~{by_line} rows ({per_row} B each), past the {cap}-row cap: no changeset can be cut by the line alone"
    );
    (by_line + cap) / 2
}

#[test]
#[ignore = "heavy: ~1 GB of request values (plan v2.33 step 11b item 2)"]
fn a_changeset_over_the_line_is_cut_into_bodies_that_fit_it() {
    let rows = rows_between_the_line_and_the_row_cap();
    let half = added(rows / 2);
    let two = [pair(half.as_str()), pair(half.as_str())];
    let (_, bodies) = moves::request(&two);
    assert_eq!(bodies.len(), 2, "cut by the line, not by the row cap");
    for b in &bodies {
        judged::fits_line(moves::KIND, b).expect("each body within the line");
    }
    let (mut link, _) = Link::open(&core_bin()).expect("open core");
    let out = classify_batch(&two, Some(&mut link));
    assert!(!out.link_failed, "{:?}", out.degraded);
    let novel: Vec<usize> = out.pairs.iter().map(|c| c.counts.added_novel).collect();
    assert_eq!(novel, [rows / 2, rows / 2], "{:?}", out.degraded);

    let whole = added(rows);
    let alone = [pair(whole.as_str())];
    let out = classify_batch(&alone, Some(&mut link));
    let why = out.degraded.unwrap_or_default();
    assert!(
        why.starts_with("moves/1: request line ") && out.pairs.is_empty() && !out.link_failed,
        "a pair over the line alone is named unjudged, not a dead link: {why}"
    );
    let next = classify_batch(&[pair("x = 1\n")], Some(&mut link));
    assert_eq!(
        next.pairs.len(),
        1,
        "the link still judges: {:?}",
        next.degraded
    );
}
