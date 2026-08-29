use super::token_eq;

/// The fold must still be an equality — timing is unobservable
/// in a unit test, so this pins the truth table.
#[test]
fn token_eq_matches_equality() {
    assert!(token_eq("abc123", "abc123"));
    assert!(!token_eq("abc123", "abc124"), "last byte differs");
    assert!(!token_eq("abc123", "abc12"), "length differs");
    assert!(!token_eq("", "x"), "empty vs non-empty");
    assert!(token_eq("", ""), "both empty");
}
