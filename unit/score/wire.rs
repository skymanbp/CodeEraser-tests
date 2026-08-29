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
