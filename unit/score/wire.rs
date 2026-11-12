use super::*;

/// C1 at the assembly site: an unclassed request is the legacy
/// three-column row with no classKnobs key at all; a classed one
/// sends every row four wide and the knob table beside it.
#[test]
fn class_column_and_knob_table_ride_only_when_classed() {
    let mut r = Request::dedup_only(0, 0, Vec::new(), None);
    r.continuous = vec![[7, 0, 310, 1], [9, 1, 20, 0]];
    let legacy = body(&r);
    assert_eq!(legacy["continuous"], json!([[7, 0, 310], [9, 1, 20]]));
    assert!(
        legacy.get("classKnobs").is_none(),
        "no key on the legacy road"
    );
    r.classed = true;
    r.class_knobs = vec![[1, 0, 400]];
    let classed = body(&r);
    assert_eq!(
        classed["continuous"],
        json!([[7, 0, 310, 1], [9, 1, 20, 0]])
    );
    assert_eq!(classed["classKnobs"], json!([[1, 0, 400]]));
}

/// The reply's `extent` (10.0.0, the extent law): `{tenths, lines}` is
/// read; tenths above ten times lines, a reply without tenths and the
/// draft's exact `{num, den, lines}` are each refused by name.
#[test]
fn the_extent_is_tenths_and_lines_or_a_named_refusal() {
    let read = |e: Value| extent_of(&json!({ "extent": e }));
    assert_eq!(
        read(json!({"tenths": 26, "lines": 3})).unwrap(),
        Some([26, 3])
    );
    assert_eq!(read(Value::Null).unwrap(), None);
    let refused = [
        (
            json!({"tenths": 31, "lines": 3}),
            "tenths 31 above ten times lines 3",
        ),
        (json!({"lines": 3}), "extent.tenths: missing or not a count"),
        (
            json!({"num": 51, "den": 20, "lines": 3}),
            "the 10.0.0 draft",
        ),
    ];
    for (e, named) in refused {
        let err = read(e).expect_err(named).to_string();
        assert!(err.contains(named), "{err}");
    }
}
