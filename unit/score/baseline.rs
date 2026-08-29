use super::*;

/// The §7.2 counterfactual pair: side order must not matter, and
/// every field participates (a moved LINE changes nothing here
/// because lines are not fields).
#[test]
fn member_identity_is_order_free_and_field_sensitive() {
    let a: Side = ("a.rs".into(), "work/1".into(), 0);
    let b: Side = ("b.rs".into(), "work/1".into(), 1);
    assert_eq!(member_id("clone", &a, &b), member_id("clone", &b, &a));
    let c: Side = ("b.rs".into(), "work/1".into(), 2);
    assert_ne!(member_id("clone", &a, &b), member_id("clone", &a, &c));
    assert_ne!(member_id("clone", &a, &b), member_id("t3", &a, &b));
}
