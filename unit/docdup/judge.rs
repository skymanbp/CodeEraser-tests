use super::*;

/// The verdict boundary in BOTH directions at the 80/100 ratio,
/// and the verbatim disjunct alone.
#[test]
fn verdict_sits_exactly_on_the_threshold() {
    assert!(is_dup(4, 5, 0));
    assert!(!is_dup(3, 4, 0));
    assert!(is_dup(0, 100, 50), "verbatim hard hit ignores jaccard");
    assert!(!is_dup(0, 100, 49));
}
