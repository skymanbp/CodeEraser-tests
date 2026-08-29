use super::*;

/// A truncated cache blob is refused, never silently shortened
/// (the whole-row decode is what every docdup run already rides).
#[test]
fn truncated_shingle_blob_is_refused_not_shortened() {
    let err = shingle_set(&[0; 17]).expect_err("truncated").to_string();
    assert!(err.contains("17 bytes"), "{err}");
}

/// Hot groups chain instead of skipping (D4) and the union dedups
/// across sources.
#[test]
fn hot_groups_chain_and_pairs_dedup() {
    let mut cand = BTreeSet::new();
    let mut hot = 0;
    let big: Vec<usize> = (0..HOT_GROUP_CAP + 2).collect();
    let n = group_pairs([big.clone()].iter(), &mut hot, &mut cand);
    assert_eq!(hot, 1);
    assert_eq!(n as usize, HOT_GROUP_CAP + 1, "adjacent chain");
    let again = group_pairs([big].iter(), &mut hot, &mut cand);
    assert_eq!(again, 0, "second source adds nothing new");
}
