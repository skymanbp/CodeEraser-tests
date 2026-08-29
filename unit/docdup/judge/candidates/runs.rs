use super::*;

fn s(v: &[u64]) -> Seq {
    indexed(v.to_vec())
}

/// Hand-derived cases in words: disjoint, overlap, interior run.
#[test]
fn run_words_measures_maximal_runs_in_words() {
    assert_eq!(run_words(&s(&[1, 2, 3]), &s(&[4, 5, 6])), 0);
    let k = spec::DOC_SHINGLE as u64;
    assert_eq!(run_words(&s(&[1, 2, 3]), &s(&[1, 2, 3])), 3 + k - 1);
    assert_eq!(run_words(&s(&[9, 1, 2, 8]), &s(&[7, 1, 2, 6])), 2 + k - 1);
    // repeated values: extension must not double-count seeds
    assert_eq!(run_words(&s(&[5, 5, 5]), &s(&[5, 5])), 2 + k - 1);
}
