use super::*;
use crate::tombstone::{Findings, Kind, Row};
use serde_json::json;

fn row(kind: Kind, marks: usize, names: usize) -> Row {
    Row {
        file: "f.md".into(),
        line: 1,
        kind,
        marks,
        names,
        name: String::new(),
        excerpt: String::new(),
        ledger: 0,
    }
}

#[test]
fn rows_are_the_three_integers_in_measurement_order_and_the_budget_is_knob_zero() {
    let f = Findings {
        rows: vec![
            row(Kind::Bracketed, 0, 1),
            row(Kind::Prose, 2, 1),
            row(Kind::Bare, 0, 3),
        ],
        ..Default::default()
    };
    let sent = rows(&f);
    assert_eq!(sent, [[0, 0, 1], [2, 2, 1], [1, 0, 3]]);
    assert_eq!(
        body(&sent, Some(3)),
        json!({"rows": [[0, 0, 1], [2, 2, 1], [1, 0, 3]], "knobs": [[0, 3]]})
    );
    assert_eq!(body(&[], None), json!({"rows": [], "knobs": []}));
}

#[test]
fn consume_relays_the_verdict_and_names_every_non_judgment() {
    let reply = json!({
        "sites": [0, 2], "counts": {"label": 1, "prose": 1, "rows": 3}, "over": true
    });
    assert_eq!(
        consume(&reply, 3),
        Ok(Judged {
            sites: vec![0, 2],
            label: 1,
            prose: 1,
            over: true,
        })
    );
    assert_eq!(
        consume(
            &json!({"degraded": true, "reason": "tombstone_too_large"}),
            3
        ),
        Err("tombstone_too_large".to_string())
    );
    let skew = consume(
        &json!({"sites": [5], "counts": {"label": 1, "prose": 0}, "over": false}),
        3,
    );
    assert!(
        skew.as_ref().is_err_and(|e| e.contains("wire skew")),
        "{skew:?}"
    );
    let short = consume(&json!({"sites": [], "over": false}), 0);
    assert!(
        short.as_ref().is_err_and(|e| e.contains("counts.label")),
        "{short:?}"
    );
}

/// A malformed reply is never read as a healthy one (codex review
/// 2026-09-04): the site table must ascend, the counts must add up to
/// it, and `over` must be a boolean.
#[test]
fn consume_refuses_a_reply_that_is_not_this_core_s_shape() {
    let counts = json!({"label": 1, "prose": 1});
    let bad = [
        (
            json!({"sites": [2, 0], "counts": counts, "over": false}),
            "ascend",
        ),
        (
            json!({"sites": [0, 0], "counts": counts, "over": false}),
            "ascend",
        ),
        (
            json!({"sites": [0, 2], "counts": {"label": 1, "prose": 0}, "over": false}),
            "counts disagree",
        ),
        (json!({"sites": [0, 2], "counts": counts}), "over"),
        (
            json!({"sites": [0, 2], "counts": counts, "over": "yes"}),
            "over",
        ),
    ];
    for (reply, why) in bad {
        let got = consume(&reply, 3);
        assert!(
            got.as_ref().is_err_and(|e| e.contains(why)),
            "{reply}: {got:?}"
        );
    }
}
