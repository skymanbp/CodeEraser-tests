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

/// The 2.18.0 split consumed whole (batch-7 slice 4, the fixture
/// the inventory found missing): reported rows label sections as
/// path#unit, the core's fail bit is relayed, and an aggregate
/// smuggled into the FAILING table refuses as wire skew — that
/// table licenses erase's class-0 rows and must never carry a
/// directory.
#[test]
fn reported_rows_and_fail_bit_consume_and_skew_refuses() {
    let nodes = vec![
        node("a.rs", "", super::super::wire::GRAN_FILE),
        node("docs/x.md", "Intro", super::super::wire::GRAN_SECTION),
        node("pkg", "", super::super::wire::GRAN_PACKAGE),
    ];
    // the confidence road: a 3-column dead row carries the
    // trust column, a 2-column (legacy) row answers None below
    let reply = json!({
        "dead": [[0, 1, 2]], "reported": [[1, 3], [2, 1]],
        "fail": true, "counts": {"kept": 7}
    });
    let r = super::consume(&reply, &nodes, 0, None).expect("consume");
    assert!(r.unmentioned.is_none(), "a road not asked has no face");
    assert_eq!(r.dead.len(), 1);
    let d = &r.dead[0];
    assert_eq!(
        (d.path.as_str(), d.verdict, d.why.as_str(), d.conf),
        (
            "a.rs",
            "unref_private",
            "no kept in-edge and no entry flag",
            Some(2)
        )
    );
    assert_eq!(
        r.reported,
        vec![
            ("docs/x.md#Intro".into(), "unreach_private"),
            ("pkg".into(), "unref_private"),
        ]
    );
    assert!(r.fail && r.kept == 7);
    let skew = json!({"dead": [[2, 1]], "counts": {}});
    let err = super::consume(&skew, &nodes, 0, None).expect_err("aggregate in dead");
    assert!(err.to_string().contains("wire skew"), "{err}");
    // a reply without the 2.18.0 keys is wire skew, not an older
    // core to accommodate: no pre-2.18 core passes the handshake,
    // and the conjunction that used to stand in is retired (O62)
    for (reply, key) in [
        (
            json!({"dead": [[0, 1]], "reported": [], "counts": {}}),
            "fail",
        ),
        (
            json!({"dead": [[0, 1]], "fail": true, "counts": {}}),
            "reported",
        ),
    ] {
        let err = super::consume(&reply, &nodes, 0, None).expect_err("absent key");
        let text = format!("{err:#}");
        assert!(
            text.contains("wire skew") && text.contains(&format!("`{key}`")),
            "the absent key is refused by name: {text}"
        );
    }
}
