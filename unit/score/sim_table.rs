use super::*;

/// Every table the check road must send, one case per line: each
/// family's file pairs as `measure` hands them over (`a-b` path pairs,
/// `,`-separated; families `|`-separated in kind order 0 t1t2, 1 t3,
/// 2 docdup) `=>` the sim rows (`u v kind`, `;`-separated) and, after
/// `/`, the self pairs counted out; `#` lines say why. Since 9.1.0
/// (v2.35 V2) the wire's identity is (u, v, kind): a pair rides once
/// per family that judged it — until then a merge kept the clone row
/// alone and a pair sharing code AND prose never charged the docdup
/// axis — and the table is strictly ascending on the three columns,
/// the order the core reads a pair's strongest row from
/// (CE.Verdict.Candidates).
const CASES: &str = "
# the clone family's (a,b) (a,d); the docdup family's (a,b) again and two prose-only pairs
a-b,a-d | | a-b,a-c,b-c => 0 1 0; 0 1 2; 0 2 2; 0 3 0; 1 2 2 / 0
# both clone families found (a,b); T3 and docdup found (c,d); a family's repeat and order do not matter
a-b | b-a,d-c,c-d | d-c => 0 1 0; 0 1 1; 2 3 1; 2 3 2 / 0
# a self pair is counted out, a path outside the universe dropped
a-a | | a-z,b-c => 1 2 2 / 1
";

fn rows(s: &str) -> Vec<[i64; 5]> {
    s.split(';')
        .map(|r| {
            let v: Vec<i64> = r.split_whitespace().map(|n| n.parse().unwrap()).collect();
            [v[0], v[1], v[2], 100, 100]
        })
        .collect()
}

#[test]
fn every_family_rides_its_own_row_ascending_by_pair_and_kind() {
    let idx: HashMap<&str, i64> = [("a", 0), ("b", 1), ("c", 2), ("d", 3)].into();
    let cases = CASES
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'));
    for case in cases {
        let (given, want) = case.split_once("=>").unwrap();
        let (want, skipped) = want.split_once('/').unwrap();
        let mut sim = Vec::new();
        let mut selfs = 0;
        for (kind, family) in given.split('|').enumerate() {
            let pairs = family.split(',').filter_map(|p| p.trim().split_once('-'));
            selfs += pair_rows(pairs, &idx, kind as i64, &mut sim);
        }
        sim.sort_unstable();
        assert_eq!(sim, rows(want), "{case}");
        assert_eq!(selfs, skipped.trim().parse::<usize>().unwrap(), "{case}");
        assert!(sim.windows(2).all(|w| w[0][..3] < w[1][..3]), "{case}");
    }
}
