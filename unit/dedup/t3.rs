use super::*;

/// The verdict boundary in BOTH directions at max = 100: ted 15
/// leaves similarity exactly 85/100 (clone), ted 16 falls below.
#[test]
fn verdict_sits_exactly_on_the_threshold() {
    assert!(is_clone(15, 100, 90));
    assert!(!is_clone(16, 100, 90));
    assert!(is_clone(0, 3, 3));
}
