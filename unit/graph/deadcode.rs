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
    // the confidence road: a 3-column dead row carries the trust
    // column, a 2-column (legacy) row rides as it came
    let reply = json!({
        "dead": [[0, 1, 2]], "reported": [[1, 3], [2, 1]],
        "fail": true, "counts": {"kept": 7}
    });
    let j = super::consume(&reply, &nodes, None).expect("consume");
    assert!(j.advisory.is_none(), "a road not asked has no rows");
    assert_eq!(j.dead, vec![vec![0, 1, 2]]);
    assert_eq!(j.reported, vec![[1, 3], [2, 1]]);
    assert!(j.fail && j.kept == Some(7) && j.degraded.is_none());
    let skew = json!({"dead": [[2, 1]], "counts": {}});
    let err = super::consume(&skew, &nodes, None).expect_err("aggregate in dead");
    assert!(err.to_string().contains("wire skew"), "{err}");
    let outside = json!({"dead": [[0, 5]], "reported": [], "fail": true});
    let err = super::consume(&outside, &nodes, None).expect_err("verdict 5");
    assert!(err.to_string().contains("verdict 5 out of range"), "{err}");
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
        let err = super::consume(&reply, &nodes, None).expect_err("absent key");
        let text = format!("{err:#}");
        assert!(
            text.contains("wire skew") && text.contains(&format!("`{key}`")),
            "the absent key is refused by name: {text}"
        );
    }
}

/// O23 (plan v2.25): the liveness reason is a CODE; the English word
/// is the document's (CE.Graph.Document.whyCodes, plan v2.32 step 4)
/// and every code has the Chinese the console prints under --lang zh
/// here, until step 5. A code without its Chinese would ship the zh
/// console one reason in English, the leak this table exists to end.
#[test]
fn every_liveness_reason_has_its_chinese() {
    for (i, zh) in super::WHY_ZH.iter().enumerate() {
        assert!(
            !zh.is_ascii() && !zh.is_empty(),
            "code {i} is not Chinese: {zh:?}"
        );
    }
    let row: super::DeadRow = serde_json::from_value(json!({
        "name": "a.rs", "verdict": "unreach_private",
        "why": "referenced only from dead code; no entry flag",
        "whyCode": 1, "confidence": null
    }))
    .expect("a dead row");
    // the default language is English: the console word IS the document's
    assert_eq!(row.why_line(), row.why);
}
