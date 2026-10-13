use crate::testutil::node;
use serde_json::json;

/// The degradation loop closes: a stamped deadcode degradation
/// is COUNTED by the same health surface `ce doctor` prints —
/// asserted, not assumed (2h exit row).
#[test]
fn degraded_stamp_reaches_the_health_counter() {
    let root = crate::testutil::scratch("dc-observe");
    assert_eq!(crate::health::degraded_runs(&root), (0, 0));
    super::observe(&root, "graph_too_large");
    assert_eq!(crate::health::degraded_runs(&root), (1, 1));
    std::fs::remove_dir_all(&root).ok();
}

/// The 2.18.0 split held at the boundary (batch-7 slice 4, the
/// fixture the inventory found missing): the dead and reported rows
/// keep the judgment's codes (the document names them, plan v2.32
/// step 4 — the verdict names and the `path#unit` label are the
/// document leg's, unit/graph/deadcode/document.rs), the core's fail
/// bit and kept count are relayed, and an aggregate smuggled into the
/// FAILING table refuses as wire skew — that table licenses erase's
/// class-0 rows and must never carry a directory.
#[test]
fn reported_rows_and_fail_bit_consume_and_skew_refuses() {
    let nodes = vec![
        node("a.rs", "", super::super::wire::GRAN_FILE),
        node("docs/x.md", "Intro", super::super::wire::GRAN_SECTION),
        node("pkg", "", super::super::wire::GRAN_PACKAGE),
    ];
    // a dead row carries the trust column: request_body always sends
    // `unres`, so the core always answers three columns
    let reply = json!({
        "dead": [[0, 1, 2]], "reported": [[1, 3], [2, 1]],
        "fail": true, "counts": {"kept": 7}
    });
    let j = super::consume(&reply, &nodes, None).expect("consume");
    assert!(j.advisory.is_none(), "a road not asked has no rows");
    assert_eq!(j.dead, vec![vec![0, 1, 2]]);
    assert_eq!(j.reported, vec![[1, 3], [2, 1]]);
    assert!(j.fail && j.kept == Some(7) && j.degraded.is_none());
    // each refused by name: an aggregate in the failing table, a row
    // without its trust column, a verdict outside the code range, and
    // a reply without a 2.18.0 key — skew (corelink::skew), not an
    // older core to accommodate: none passes the handshake, and the
    // conjunction that used to stand in is retired (O62)
    let skewed = [
        json!({"dead": [[2, 1, 0]], "reported": [], "fail": true, "counts": {}}),
        json!({"dead": [[0, 1]], "reported": [], "fail": true, "counts": {}}),
        json!({"dead": [[0, 5, 0]], "reported": [], "fail": true}),
        json!({"dead": [[0, 1, 0]], "reported": [], "counts": {}}),
        json!({"dead": [[0, 1, 0]], "fail": true, "counts": {}}),
    ];
    let says = [
        "wire skew",
        "dead row of arity 2",
        "verdict 5 out of range",
        "offers no graph/1 `fail`",
        "offers no graph/1 `reported`",
    ];
    for (reply, says) in skewed.iter().zip(says) {
        let err = super::consume(reply, &nodes, None).expect_err(says);
        let text = format!("{err:#}");
        assert!(text.contains(says), "refused by name: {text}");
    }
}
