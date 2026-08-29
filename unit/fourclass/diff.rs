use super::*;

fn run(a: &[u64], b: &[u64]) -> (Vec<usize>, Vec<usize>) {
    let d = diff(a, b);
    assert!(!d.degraded);
    (d.removed, d.added)
}

#[test]
fn identical_inputs_have_no_changes() {
    assert_eq!(run(&[1, 2, 3], &[1, 2, 3]), (vec![], vec![]));
    assert_eq!(run(&[], &[]), (vec![], vec![]));
}

#[test]
fn pure_insertion_and_deletion() {
    assert_eq!(run(&[1, 3], &[1, 2, 3]), (vec![], vec![1]));
    assert_eq!(run(&[1, 2, 3], &[1, 3]), (vec![1], vec![]));
    assert_eq!(run(&[], &[7, 8]), (vec![], vec![0, 1]));
    assert_eq!(run(&[7, 8], &[]), (vec![0, 1], vec![]));
}

#[test]
fn one_side_empty_is_exact_beyond_the_cap() {
    // a pure creation/deletion larger than MAX_D is still exact —
    // only the bounded SEARCH degrades, never the trivial script
    let big: Vec<u64> = (0..(MAX_D as u64 + 10)).collect();
    let d = diff(&[], &big);
    assert!(!d.degraded);
    assert_eq!((d.removed.len(), d.added.len()), (0, big.len()));
    let d = diff(&big, &[]);
    assert!(!d.degraded);
    assert_eq!((d.removed.len(), d.added.len()), (big.len(), 0));
}

#[test]
fn replacement_counts_both_sides() {
    let (r, a) = run(&[1, 2, 3], &[1, 9, 3]);
    assert_eq!((r, a), (vec![1], vec![1]));
}

#[test]
fn relocation_reports_one_removal_one_addition() {
    // 5 moved from front to back — minimal script is one del + one add.
    let (r, a) = run(&[5, 1, 2, 3], &[1, 2, 3, 5]);
    assert_eq!(r.len(), 1);
    assert_eq!(a.len(), 1);
}

#[test]
fn counts_are_minimal_against_lcs_on_small_cases() {
    // |removed| = n - LCS, |added| = m - LCS for adversarial small
    // inputs (rotation, reversal, duplicate swap, alternation),
    // derived programmatically and LCS'd by brute force.
    let base: Vec<u64> = (1..=5).collect();
    let rotated = [&base[2..], &base[..2]].concat();
    let reversed: Vec<u64> = base.iter().rev().copied().collect();
    let dupes: Vec<u64> = vec![1, 1, 2, 2];
    let dupes_swapped = [&dupes[2..], &dupes[..2]].concat();
    let alt: Vec<u64> = (0..5).map(|i| 1 + i % 2).collect();
    let alt_flipped: Vec<u64> = alt.iter().map(|v| 3 - v).collect();
    for (a, b) in [
        (&base, &rotated),
        (&base, &reversed),
        (&dupes, &dupes_swapped),
        (&alt, &alt_flipped),
    ] {
        let lcs = lcs_len(a, b);
        let (r, ad) = run(a, b);
        assert_eq!(r.len(), a.len() - lcs, "removed for {a:?} vs {b:?}");
        assert_eq!(ad.len(), b.len() - lcs, "added for {a:?} vs {b:?}");
    }
}

fn lcs_len(a: &[u64], b: &[u64]) -> usize {
    let mut t = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            t[i][j] = if a[i - 1] == b[j - 1] {
                t[i - 1][j - 1] + 1
            } else {
                t[i - 1][j].max(t[i][j - 1])
            };
        }
    }
    t[a.len()][b.len()]
}
