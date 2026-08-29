use super::*;

#[test]
fn shingles_are_order_sensitive_and_deduplicated() {
    let ab: Vec<u64> = vec![1, 2, 3, 4, 5];
    let ba: Vec<u64> = vec![5, 4, 3, 2, 1];
    assert_ne!(shingles(&ab), shingles(&ba), "order must matter");
    assert_eq!(shingles(&[1, 2, 3]).len(), 0, "below k yields empty");
    let rep: Vec<u64> = vec![7; 10];
    assert_eq!(shingles(&rep).len(), 1, "identical windows collapse");
}

#[test]
fn histogram_counts_the_multiset() {
    let h = histogram(&[9, 9, 4]);
    assert_eq!(h.get(&9), Some(&2));
    assert_eq!(h.get(&4), Some(&1));
}
