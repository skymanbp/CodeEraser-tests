//! The similar/1 leg against the REAL core (plan v2.29 step 5): every
//! unit of the go fixture corpus is measured the way the ROI
//! instrument measures it, its bare top-k rides the wire, and the
//! core's answer must agree with the measuring side's own arithmetic
//! — the order it sent (score descending, then identity, which is the
//! wire's tie order) and the role bit bm25.rs carries as the declared
//! mirror of CE.Similar.Cost. A disagreement is a drift between the
//! two sides of one definition, and this is the leg that sees it.
//! Then the capability gate: a link whose core does not offer the
//! family answers a named refusal, never an empty order.

use crate::common::{core_bin, repo_root};
use crate::similar_replay::measure;
use codeeraser::corelink::Link;
use codeeraser::similar::wire;

#[test]
fn the_core_orders_and_roles_every_go_unit_as_the_measurement_did() {
    let m = measure(&repo_root().join("contracts/fixtures/crosscheck/go"), "go");
    let (mut link, _) = Link::open(&core_bin()).expect("open");
    let mut judged_rows = 0;
    for (i, (bare, _)) in m.ranked.iter().enumerate() {
        let query = m.corpus.query_of(i);
        let j = wire::judge(&mut link, &query, bare).expect("judgment");
        assert_eq!(
            j.order,
            (0..bare.len()).collect::<Vec<_>>(),
            "unit {i}: the core re-ordered what the measurement sent"
        );
        let roles: Vec<bool> = bare.iter().map(|h| h.role).collect();
        assert_eq!(j.roles, roles, "unit {i}: role bits differ");
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
