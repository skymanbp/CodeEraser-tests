use super::*;

/// Zero width is answered, not computed: both entries used to run
/// an unsigned `- 1` on it and panic in debug.
/// Sixteen tokens through both entries at (k, w) — the shared
/// probe of the two width tests (their stanzas were a clone).
fn widths(k: usize, w: usize) -> (usize, usize) {
    let tokens: Vec<u64> = (0..16).collect();
    let fp = fingerprints(
        &tokens,
        Params {
            kgram: k,
            window: w,
        },
    );
    (kgram_hashes(&tokens, k).len(), fp.len())
}

#[test]
fn zero_width_selects_nothing_instead_of_underflowing() {
    assert_eq!(widths(0, 4).0, 0, "k == 0");
    assert_eq!(widths(5, 0).1, 0, "window == 0");
}

/// The guard is narrow — the normal shape is untouched.
#[test]
fn nonzero_width_still_fingerprints() {
    let (grams, fps) = widths(5, 4);
    assert_eq!(grams, 12);
    assert!(fps > 0);
}
