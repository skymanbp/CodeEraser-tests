use super::*;

/// Two families, each ascending on its own, concatenated the way
/// `measure` hands them over — with one pair in BOTH. The merged
/// table is strictly ascending by pair (the wire's own contract,
/// `table "sim" (simRow n) 2`), carries that pair exactly once, and
/// keeps the STRONGER kind for it: 0 t1t2 over 2 docdup. Nothing
/// downstream can see that last choice today (both rows ride the
/// verified 100/100 and clear their family's bar), so this is the
/// only place it is nailed down.
#[test]
fn one_row_per_pair_merges_two_ascending_families() {
    let mut sim = vec![
        // the clone family's set: (0,1) and (0,3)
        [0, 1, 0, 100, 100],
        [0, 3, 0, 100, 100],
        // the docdup family's, appended whole: (0,1) again, then two
        // pairs only prose relates
        [0, 1, 2, 100, 100],
        [0, 2, 2, 100, 100],
        [1, 2, 2, 100, 100],
    ];
    one_row_per_pair(&mut sim);
    assert_eq!(
        sim,
        vec![
            [0, 1, 0, 100, 100], // the clone row survived, not its docdup twin
            [0, 2, 2, 100, 100],
            [0, 3, 0, 100, 100],
            [1, 2, 2, 100, 100],
        ]
    );
    assert!(
        sim.windows(2)
            .all(|w| (w[0][0], w[0][1]) < (w[1][0], w[1][1])),
        "strictly ascending by pair: {sim:?}"
    );
}

/// The identity is the PAIR, never the whole row: two rows that
/// differ in kind alone are one pair, and the merge must not be
/// fooled by a dedup that compares five columns. Ordered the way a
/// plain `Vec::dedup` would keep both.
#[test]
fn the_row_identity_is_the_pair_not_the_kind() {
    let mut sim = vec![[4, 9, 2, 100, 100], [4, 9, 0, 100, 100]];
    one_row_per_pair(&mut sim);
    assert_eq!(sim, vec![[4, 9, 0, 100, 100]]);
}
