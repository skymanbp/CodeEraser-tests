use super::*;

#[test]
fn signatures_are_deterministic_and_set_order_free() {
    let a = signature(&[3, 1, 2], 16);
    let b = signature(&[1, 2, 3], 16);
    assert_eq!(a, b, "a set signature must not depend on order");
    assert_eq!(a, signature(&[1, 2, 3], 16), "two runs, same bytes");
    assert_ne!(a, signature(&[1, 2, 4], 16), "different set, different sig");
}

#[test]
fn empty_sets_saturate_and_band_keys_partition() {
    assert!(signature(&[], 8).iter().all(|v| *v == u64::MAX));
    let sig = signature(&[7, 9], 8);
    let keys = band_keys(&sig, 4, 2);
    assert_eq!(keys.len(), 4);
    assert_eq!(keys, band_keys(&sig, 4, 2), "banding is deterministic");
}
