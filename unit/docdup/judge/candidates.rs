use super::*;

/// A truncated cache blob is refused, never silently shortened
/// (the whole-row decode is what every docdup run already rides).
#[test]
fn truncated_shingle_blob_is_refused_not_shortened() {
    let err = shingle_set(&[0; 17]).expect_err("truncated").to_string();
    assert!(err.contains("17 bytes"), "{err}");
}

// Differential gate for the docdup coarse filter and verbatim runs'
// move into the core (plan v2.33 W3, user instruction 2026-10-03):
// seeded random segment sets and shingle sequences through (a) the
// frozen Rust oracle (unit/w3_oracle/docdup.rs, the 093aede4 code) and
// (b) the core over the real wire (docpairs/1, then docdup/1's
// sequence shape) — the kept pairs, the tally, and per pair the
// intersection, union, verbatim run and verdict, all equal. Edges: no
// segment, one segment, identical sets (band groups and seed groups at
// and past the hot cap), a set at the per-set ceiling and one past it,
// u64 shingles at and above 2^63 and u64::MAX.

use crate::w3_oracle::{Lcg, core_link, docdup as oracle};

const SHINGLES: [u64; 9] = [
    0,
    1,
    5,
    1 << 62,
    1 << 63,
    (1 << 63) + 9,
    u64::MAX - 3,
    u64::MAX,
    0xD1B5_4A32_D192_ED03,
];

fn seg(set: Vec<u64>) -> SegRow {
    SegRow {
        path: "a.md".into(),
        kind: 0,
        start_line: 1,
        end_line: 1,
        words: 0,
        set,
    }
}

fn set_of(rng: &mut Lcg, len: usize) -> Vec<u64> {
    let mut s: Vec<u64> = (0..len)
        .map(|_| rng.pick(&SHINGLES) ^ (rng.next(3) as u64))
        .collect();
    s.sort_unstable();
    s.dedup();
    s
}

fn segs(rng: &mut Lcg, n: usize) -> Vec<SegRow> {
    let cap = super::super::wire::limits().doc_set_cap;
    match n % 7 {
        0 => Vec::new(),
        1 => {
            let len = 1 + rng.next(6);
            vec![seg(set_of(rng, len))]
        }
        2 => (0..64 + rng.next(4))
            .map(|_| seg(vec![SHINGLES[4], SHINGLES[7]]))
            .collect(),
        3 => {
            // one shingle shared by 64 to 67 otherwise distinct sets
            (0..64 + rng.next(4) as u64)
                .map(|i| seg(vec![i * 2 + 3, SHINGLES[7]]))
                .collect()
        }
        // the per-set ceiling and one past it, once a hundred cases
        4 if n % 100 == 4 => vec![
            seg((0..cap as u64).collect()),
            seg((0..cap as u64 + 1).collect()),
            seg(vec![0, 1]),
        ],
        _ => (0..rng.next(30))
            .map(|_| {
                let len = 1 + rng.next(8);
                seg(set_of(rng, len))
            })
            .collect(),
    }
}

#[test]
fn ten_thousand_seeded_coarse_passes_answer_as_the_frozen_rust_did() {
    let mut link = core_link();
    let mut rng = Lcg(0x5EED_0005);
    for n in 0..10_000 {
        let segs = segs(&mut rng, n);
        let (want, wt) = oracle::coarse(&segs);
        let (got, gt) = coarse(&segs, &mut link).expect("the core answers");
        let tally = |t: &Tally| {
            [
                t.over_cap_segments,
                t.lsh_pairs,
                t.seed_pairs,
                t.hot_bands,
                t.hot_shingles,
            ]
        };
        assert_eq!(got, want.into_iter().collect::<Vec<_>>(), "case {n}: pairs");
        assert_eq!(tally(&gt), tally(&wt), "case {n}: tally");
    }
}

/// One round's shingle sequences: every fourth round a shared run of
/// 40 to 48 shingles (44 to 52 words, the verbatim floor 50 on both
/// sides) inside distinct noise, otherwise short draws over `alphabet`.
fn draw_seqs(rng: &mut Lcg, round: usize, alphabet: &[u64]) -> Vec<Vec<u64>> {
    if round.is_multiple_of(4) {
        let run: Vec<u64> = (0..40 + rng.next(9) as u64).map(|i| 1000 + i).collect();
        (0..2 + rng.next(3))
            .map(|s| {
                let noise = |rng: &mut Lcg, tag: u64| -> Vec<u64> {
                    (0..rng.next(40) as u64).map(|i| (tag << 32) + i).collect()
                };
                let mut v = noise(rng, 2 * s as u64 + 1);
                v.extend(&run);
                v.extend(noise(rng, 2 * s as u64 + 2));
                v
            })
            .collect()
    } else {
        (0..2 + rng.next(6))
            .map(|_| (0..1 + rng.next(14)).map(|_| rng.pick(alphabet)).collect())
            .collect()
    }
}

/// The frozen Rust's answer for one pair: set intersection, set union,
/// longest verbatim run, verdict.
fn frozen_row(a: &[u64], b: &[u64]) -> (u64, u64, u64, bool) {
    let set = |s: &[u64]| -> std::collections::BTreeSet<u64> { s.iter().copied().collect() };
    let (sa, sb) = (set(a), set(b));
    let inter = sa.intersection(&sb).count() as u64;
    let union = sa.union(&sb).count() as u64;
    let run = oracle::run_words(&oracle::indexed(a.to_vec()), &oracle::indexed(b.to_vec()));
    (inter, union, run, oracle::is_dup(inter, union, run))
}

#[test]
fn ten_thousand_seeded_pairs_measure_runs_and_verdicts_as_the_frozen_rust_did() {
    let core = crate::daemon::judge::core_bin().expect("a core");
    let fam = super::super::wire::family(&core);
    let mut link = crate::lockstep::open_family(fam.core, fam.cap).expect("open core");
    let mut rng = Lcg(0x5EED_0006);
    let alphabet = [SHINGLES[0], SHINGLES[4], SHINGLES[7], 42];
    let (mut compared, mut round) = (0usize, 0usize);
    // [verbatim-only dup, run 46..=49 without Jaccard, Jaccard dup]
    let mut seen = [0u32; 3];
    while compared < 10_000 {
        round += 1;
        let seqs = draw_seqs(&mut rng, round, &alphabet);
        let pairs: Vec<(usize, usize)> = (0..seqs.len())
            .flat_map(|a| (a + 1..seqs.len()).map(move |b| (a, b)))
            .collect();
        let (rows, ..) = crate::lockstep::lockstep_scores(
            &mut link,
            &fam,
            &pairs,
            |chunk| super::super::wire::chunk_request(chunk, |g| &seqs[g]),
            super::super::wire::parse_result,
        )
        .expect("the core judges");
        for (a, b, (inter, union, run, verdict)) in rows {
            assert_eq!(
                (inter, union, run, verdict),
                frozen_row(&seqs[a], &seqs[b]),
                "round {round} pair ({a},{b}): {:?} / {:?}",
                seqs[a],
                seqs[b]
            );
            let jaccard = inter * 100 >= 80 * union;
            seen[0] += u32::from(verdict && !jaccard);
            seen[1] += u32::from((46..50).contains(&run) && !jaccard);
            seen[2] += u32::from(jaccard);
            compared += 1;
        }
    }
    assert!(
        seen.iter().all(|&c| c > 0),
        "verbatim-only / near-floor / Jaccard never met: {seen:?}"
    );
}
