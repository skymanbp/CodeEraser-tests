use super::*;

/// The dense mapping is request-scoped and first-seen ordered:
/// two trees sharing kinds share codes, and codes never exceed
/// the distinct-kind count (the judge compares for equality only,
/// but small labels keep the wire inspectable). The generic pin
/// battery lives with corelink; this family owns only its knob
/// list, exercised through its own parse_result.
#[test]
fn dense_labels_are_request_scoped_and_knobs_pin() {
    let a = UnitTree {
        lab: vec![900, 700, 900],
        lld: vec![0, 1, 0],
    };
    let b = UnitTree {
        lab: vec![700, 800],
        lld: vec![0, 0],
    };
    let body = request_body(&[&a, &b], &[[0, 1]]);
    assert_eq!(body["trees"][0]["lab"], json!([0, 1, 0]));
    assert_eq!(body["trees"][1]["lab"], json!([1, 2]));
    assert_eq!(body["pairs"], json!([[0, 1]]));
    let ok = json!({"scores": [[0, 1, 2, 3, 3]], "verdicts": [false],
            "counts": {"judged": 1, "prefiltered": 0},
            "knobs": {"tsedNum": 85, "tsedDen": 100, "minUnitNodes": 24}, "degraded": false});
    assert_eq!(
        parse_result(&ok).expect("well-formed").0,
        vec![(0, 1, (2, 3, 3, false))]
    );
    let mut drifted = ok;
    drifted["knobs"]["tsedNum"] = json!(80);
    assert!(parse_result(&drifted).is_err(), "threshold drift refuses");
}
