use super::*;

/// v2.35 G6 (R3-43): a predicate a producer names that the core's
/// schema lacks is a core of another build — refused by name when the
/// tables are finished, never a panic mid-assembly; nothing is read or
/// written under the missing name meanwhile.
#[test]
fn a_predicate_the_schema_lacks_is_a_named_skew() {
    let mut sink = Sink {
        codes: [("node".to_string(), 1000)].into(),
        wanted: [1000].into(),
        tables: [(1000, BTreeSet::new())].into(),
        skew: std::cell::RefCell::new(None),
    };
    assert!(sink.wants("node"));
    assert!(!sink.wants("gone"));
    sink.row("gone", vec![1]);
    sink.row("node", vec![2]);
    let err = sink.finish().expect_err("a schema skew");
    assert!(format!("{err}").contains("no predicate `gone`"), "{err}");
}
