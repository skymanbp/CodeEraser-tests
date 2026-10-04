use super::*;

/// A truncated cache blob is refused, never silently shortened
/// (the whole-row decode is what every docdup run already rides).
#[test]
fn truncated_shingle_blob_is_refused_not_shortened() {
    let err = shingle_set(&[0; 17]).expect_err("truncated").to_string();
    assert!(err.contains("17 bytes"), "{err}");
}
