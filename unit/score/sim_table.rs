use super::*;

/// Every merge `one_row_per_pair` must make, one case per line: the
/// families' rows as `measure` hands them over (each family ascending
/// on its own, appended whole) `=>` the merged table; a row is
/// `u v kind num den`, rows are `;`-separated, `#` lines say why. The
/// merged table is strictly ascending by pair (the wire's own
/// contract, `table "sim" (simRow n) 2`), carries each pair exactly
/// once and keeps the STRONGER kind: 0 t1t2 over 1 t3 (plan v2.30 step
/// 5b-9) over 2 docdup — the order picks which bar the verified ratio
/// is read against (both clone kinds share the clone bar), and nothing
/// downstream can see the choice on the shipped 100/100 rows, so this
/// is the only place it is nailed down. The identity is the PAIR,
/// never the whole row: two rows that differ in kind alone are one
/// pair, and a dedup comparing five columns would keep both.
const CASES: &str = "
# the clone family's (0,1) (0,3); the docdup family's (0,1) again and two prose-only pairs
0 1 0 100 100; 0 3 0 100 100; 0 1 2 100 100; 0 2 2 100 100; 1 2 2 100 100 => 0 1 0 100 100; 0 2 2 100 100; 0 3 0 100 100; 1 2 2 100 100
# both clone families found (0,1); the T3 family and the docdup family found (2,3)
0 1 0 100 100; 0 1 1 100 100; 2 3 1 100 100; 2 3 2 100 100 => 0 1 0 100 100; 2 3 1 100 100
# one pair, two kinds, the weaker first — the order a plain Vec::dedup would keep both in
4 9 2 100 100; 4 9 0 100 100 => 4 9 0 100 100
";

fn rows(s: &str) -> Vec<[i64; 5]> {
    s.split(';')
        .map(|r| {
            let v: Vec<i64> = r.split_whitespace().map(|n| n.parse().unwrap()).collect();
            v.try_into().unwrap()
        })
        .collect()
}

#[test]
fn one_row_per_pair_keeps_the_stronger_kind_ascending_by_pair() {
    let cases = CASES
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'));
    for case in cases {
        let (given, want) = case.split_once("=>").unwrap();
        let mut sim = rows(given);
        one_row_per_pair(&mut sim);
        assert_eq!(sim, rows(want), "{case}");
        assert!(
            sim.windows(2)
                .all(|w| (w[0][0], w[0][1]) < (w[1][0], w[1][1])),
            "strictly ascending by pair: {case}"
        );
    }
}
