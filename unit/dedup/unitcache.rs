use super::*;

/// A blob that is not whole rows is refused by name, never
/// silently shortened; whole rows decode to exactly their count.
#[test]
fn a_truncated_blob_is_refused_not_shortened() {
    let whole = sig_blob(&[7u64, 9].into_iter().collect());
    assert_eq!(whole_rows(&whole, 8).expect("whole").count(), 2);
    assert!(whole_rows(&whole[..whole.len() - 1], 8).is_err());
    assert!(
        whole_rows(&whole, 12).is_err(),
        "16 bytes are not 12-byte rows"
    );
}
