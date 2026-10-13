//! The moves/1 plan meets the protocol's line (plan v2.33 step 11b item
//! 2). A line row `[content, fnv1a, width]` is ~40 bytes on the wire with
//! its changed index, so the 33,554,432-byte line is crossed near 0.8 M
//! rows — a fifth of the 4,194,304-row cap the packing used to stop at;
//! the core then answered the id-less `too_large` and the daemon read it
//! as a dead link. Sizes: two added files of 500,000 lines each (~20 MB
//! per pair on the wire, ~41 MB together, 1 M rows: one body by the row
//! cap alone) and one added file of 900,000 lines (~37 MB alone).
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

#[test]
#[ignore = "heavy: ~1 GB of request values (plan v2.33 step 11b item 2)"]
fn a_changeset_over_the_line_is_cut_into_bodies_that_fit_it() {
    let half = added(500_000);
    let pair = |after| PairInput {
        before: "",
        after,
        lang: Lang::Python,
    };
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
    assert_eq!(novel, [500_000, 500_000], "{:?}", out.degraded);

    let whole = added(900_000);
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
