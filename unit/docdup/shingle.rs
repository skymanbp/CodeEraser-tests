use super::*;

/// One unmasked pass — the probe both equivalence tests share
/// (their twin stanzas were a clone block).
fn words(line: &str) -> Vec<u64> {
    let mut out = Vec::new();
    line_words(line, None, &mut out);
    out
}

#[test]
fn words_are_case_folded_alnum_runs_and_masks_erase() {
    assert_eq!(words("Hello, WORLD-42!"), words("hello world 42"));
    let line = "keep `code span` keep";
    let mut mask = vec![false; line.len()];
    mask[5..16].fill(true);
    let mut m = Vec::new();
    line_words(line, Some(&mask), &mut m);
    let mut plain = Vec::new();
    line_words("keep keep", None, &mut plain);
    assert_eq!(m, plain);
}

/// Canonical equivalence: the same prose typed NFC and NFD must
/// yield the SAME word hashes, and the combining mark must not
/// split a word in two. Unnormalized, the decomposed line gave
/// three words to the composed line's two.
#[test]
fn composed_and_decomposed_prose_hash_alike() {
    let nfc = words("caf\u{e9} na\u{ef}ve");
    assert_eq!(
        nfc,
        words("cafe\u{301} nai\u{308}ve"),
        "canonical equivalents hash alike"
    );
    assert_eq!(nfc.len(), 2, "a mark must not end the word it sits on");
}

#[test]
fn shingle_set_is_order_free_and_seq_is_not() {
    let w1: Vec<u64> = (0..8).collect();
    let w2: Vec<u64> = (0..8).rev().collect();
    assert_ne!(shingle_seq(&w1), shingle_seq(&w2));
    let mut s1 = shingle_set(&w1);
    s1.sort_unstable();
    assert_eq!(shingle_set(&w1), s1, "already sorted deduped");
    assert_eq!(shingle_seq(&w1).len(), 8 - DOC_SHINGLE + 1);
}
