//! The similar/1 leg against the REAL core (plan v2.29 step 5): every
//! unit of the go fixture corpus is measured the way the ROI
//! instrument measures it — ranked by rank/1, judged by similar/1,
//! which must keep the rank order (score descending, then seat, which
//! is identity) — and every role bit must be the spec's conjunction over
//! the evidence row the core answered: N ≥ 1 ∧ C ≥ 1, or N ≥ 2 with
//! the shape equal (spelled here, the test's own reading of booklet 15
//! §5). Then the capability gate: a link whose core does not offer the
//! family answers a named refusal, never an empty order.

use crate::common::{core_bin, repo_root};
use crate::similar_replay::measure;
use codeeraser::corelink::Link;
use codeeraser::similar::wire;

#[test]
fn the_core_ranks_and_roles_every_go_unit_by_the_spec_conjunction() {
    let m = measure(&repo_root().join("contracts/fixtures/crosscheck/go"), "go");
    let mut judged_rows = 0;
    for (i, (bare, _)) in m.ranked.iter().enumerate() {
        for c in bare {
            let [n, _, callee, ..] = c.hits;
            let want = (n >= 1 && callee >= 1) || (n >= 2 && c.shape_equal);
            assert_eq!(c.role, want, "unit {i}: seat {} role bit", c.doc);
        }
        judged_rows += bare.len();
    }
    assert!(judged_rows > 100, "the go corpus judged {judged_rows} rows");
}

/// The handshake offers the family; a well-formed body is judged; a
/// malformed row comes back as the core's NAMED contract refusal
/// through the same link (never an empty order), and the link is
/// still in step afterwards.
#[test]
fn the_family_is_offered_judged_and_refuses_by_name_over_one_link() {
    let (mut link, hello) = Link::open(&core_bin()).expect("open");
    assert!(hello.capabilities.iter().any(|c| c == wire::CAP));
    let rows = [[1, 0, 1, 0, 0, 0, 0, 65536, 65536]];
    let query = [[7u64, 768u64]];
    let reply = wire::ask(&mut link, &query, &rows).expect("answered");
    assert_eq!(
        wire::consume(&reply, 1).expect("judged"),
        wire::Judged {
            order: vec![0],
            roles: vec![true]
        }
    );
    let bad = [[1, 0, 1, 0, 0, 0, 2, 65536, 65536]];
    let err = wire::ask(&mut link, &query, &bad).expect_err("refused");
    assert!(
        err.contains("contract") && err.contains("row 0: shapeEqual not a boolean"),
        "{err}"
    );
    let again = wire::ask(&mut link, &query, &rows).expect("still in step");
    assert_eq!(wire::consume(&again, 1).expect("judged").order, vec![0]);
}
