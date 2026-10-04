//! Differential gate for the T3 candidate pass's move into the core
//! (plan v2.33 W3, user instruction 2026-10-03): seeded random unit
//! sets, fingerprint instances and near runs through (a) the frozen
//! Rust oracle (unit/w3_oracle/t3.rs, the 093aede4 code) and (b) this
//! lowering over the real core wire — the kept pairs with their source
//! bits and every tally field equal, with and without S5. The generator
//! walks the edges: no units, one unit, all-equal keys / languages /
//! node counts (sort and owner ties), node counts on both sides of the
//! 85/100 bound, u64 hashes and kind labels at and above 2^63 and at
//! u64::MAX, hash groups and band groups at the hot cap and one past
//! it, duplicate instances, anchors in no unit and in unknown files.
//! The clone verdict mirror (`is_clone`) is checked against clone/1
//! `decide` in the same file.

use super::{Facts, pass};
use crate::dedup::candidates::{PairRow, Tally, Unit};
use crate::dedup::index::Instance;
use crate::w3_oracle::{Lcg, core_link as core, t3 as oracle};

/// Hashes and labels: small, at and above 2^63, the maximum.
const WORDS: [u64; 10] = [
    0,
    1,
    2,
    7,
    1 << 40,
    1 << 63,
    (1 << 63) + 1,
    u64::MAX - 1,
    u64::MAX,
    0x9E37_79B9_7F4A_7C15,
];
const PATHS: [&str; 5] = ["a.rs", "b.rs", "c.py", "d/e.go", "z.md"];
const LANGS: [&str; 3] = ["rust", "python", "go"];
const KEYS: [&str; 4] = ["f/0", "g/1", "h/0", "k/2"];
/// Node counts around the 85/100 size bound and the 24-node floor.
const NODES: [i64; 9] = [24, 28, 84, 85, 99, 100, 101, 117, 120];

/// `total` nodes spread over the labels, each at least one: a unit's
/// kind histogram counts its nodes (the candidates contract refuses a
/// histogram that does not sum to the unit's node count).
fn split(rng: &mut Lcg, labels: &[u64], total: i64) -> Vec<(u64, u32)> {
    let mut counts = vec![1u32; labels.len()];
    for _ in 0..total - labels.len() as i64 {
        counts[rng.next(labels.len())] += 1;
    }
    labels.iter().copied().zip(counts).collect()
}

/// Sorted distinct labels, at least one.
fn labels(rng: &mut Lcg, most: usize) -> Vec<u64> {
    let mut labels: Vec<u64> = (0..1 + rng.next(most)).map(|_| rng.pick(&WORDS)).collect();
    labels.sort_unstable();
    labels.dedup();
    labels
}

fn unit(rng: &mut Lcg, same: bool) -> Unit {
    let start = 1 + rng.next(40) as i64;
    let labels = labels(rng, 5);
    let nodes = if same { 100 } else { rng.pick(&NODES) };
    Unit {
        path: rng.pick(&PATHS[..4]).to_string(),
        key: if same {
            "f/0".into()
        } else {
            rng.pick(&KEYS).into()
        },
        nth: 0,
        lang: if same {
            "rust".into()
        } else {
            rng.pick(&LANGS).into()
        },
        nodes,
        start_line: start,
        end_line: start + rng.next(12) as i64,
        sig: (0..1 + rng.next(8)).map(|_| rng.pick(&WORDS)).collect(),
        hist: split(rng, &labels, nodes),
    }
}

fn instance(rng: &mut Lcg, hash: u64) -> Instance {
    Instance {
        hash,
        file: rng.pick(&PATHS).to_string(),
        start_tok: rng.next(5),
        start_line: 1 + rng.next(55),
        end_line: 0,
    }
}

/// One generated pass input: units, fingerprint instances, near
/// anchors, exhaustive.
type Case = (Vec<Unit>, Vec<Instance>, Vec<[(String, i64); 2]>, bool);

/// One generated pass input, by case shape.
fn case(rng: &mut Lcg, n: usize) -> Case {
    let shape = n % 8;
    let count = match shape {
        0 => 0,
        1 => 1,
        2 => 64 + rng.next(4), // band groups at and past the hot cap
        _ => rng.next(25),
    };
    let same = shape == 3 || shape == 2;
    let mut units: Vec<Unit> = (0..count).map(|_| unit(rng, same)).collect();
    if shape == 2 {
        for u in &mut units {
            u.sig = vec![WORDS[5], WORDS[8]]; // one bucket per band
        }
    }
    let mut instances: Vec<Instance> = (0..rng.next(30))
        .map(|_| {
            let h = rng.pick(&WORDS);
            instance(rng, h)
        })
        .collect();
    if shape == 4 || shape == 5 {
        // a hash group at the cap (64) or one past it (65)
        let h = rng.pick(&WORDS);
        instances.extend((0..63 + shape - 3).map(|_| instance(rng, h)));
    }
    if shape == 6 && !instances.is_empty() {
        let dup = instances[0].clone();
        instances.push(dup);
    }
    let near = (0..rng.next(6))
        .map(|_| {
            let mut anchor = || (rng.pick(&PATHS).to_string(), 1 + rng.next(55) as i64);
            [anchor(), anchor()]
        })
        .collect();
    (units, instances, near, rng.next(2) == 0)
}

fn pairs_of(rows: &[PairRow]) -> Vec<(usize, usize, u8)> {
    rows.iter().map(|p| (p.a, p.b, p.sources)).collect()
}

fn tally_of(t: &Tally) -> String {
    format!(
        "raw {:?} union {} cross {} unowned {} self {} size {} label {} survivors {} s3hot {} s4hot {} groups {:?} s5 {} {} {} {}",
        t.raw_by,
        t.union_pairs,
        t.cross_lang_dropped,
        t.unowned_dropped,
        t.self_pair_dropped,
        t.pruned_size,
        t.pruned_label,
        t.survivors,
        t.s3_hot_chained,
        t.s4_hot_chained,
        t.s4_band_groups,
        t.s5_windowed,
        t.s5_pruned_label,
        t.s5_already,
        t.s5_new
    )
}

/// The counters the seeded pass must each move on some case: cross /
/// unowned / self / size / label / survivors / S3 hot / S4 hot / S5
/// label / S5 already / S5 new.
fn moved_counters(t: &Tally) -> [u64; 11] {
    [
        t.cross_lang_dropped,
        t.unowned_dropped,
        t.self_pair_dropped,
        t.pruned_size,
        t.pruned_label,
        t.survivors,
        t.s3_hot_chained,
        t.s4_hot_chained,
        t.s5_pruned_label,
        t.s5_already,
        t.s5_new,
    ]
}

#[test]
fn ten_thousand_seeded_passes_answer_as_the_frozen_rust_did() {
    let mut link = core();
    let mut rng = Lcg(0x5EED_0003);
    let mut seen = [0u64; 11];
    for n in 0..10_000 {
        let (units, instances, near, exhaustive) = case(&mut rng, n);
        let (want_pairs, want_tally) = oracle::pass(&units, &near, &instances, exhaustive);
        for (s, f) in seen.iter_mut().zip(moved_counters(&want_tally)) {
            *s += u64::from(f > 0);
        }
        let facts = Facts {
            instances: &instances,
            near: &near,
            exhaustive,
        };
        let mut tally = Tally::default();
        let got = pass(&mut link, &units, &facts, &mut tally).expect("the core answers");
        assert_eq!(pairs_of(&got), pairs_of(&want_pairs), "case {n}: pairs");
        assert_eq!(tally_of(&tally), tally_of(&want_tally), "case {n}: tally");
    }
    // not vacuous: cross / unowned / self / size / label / survivors /
    // S3 hot / S4 hot / S5 label / S5 already / S5 new each on some case
    assert!(
        seen.iter().all(|&c| c > 0),
        "a tally field never moved: {seen:?}"
    );
}

#[test]
fn the_clone_verdict_answers_as_the_frozen_mirror_did() {
    let mut link = core();
    let mut rng = Lcg(0x5EED_0001);
    let sizes = [1i64, 2, 3, 17, 20, 24, 85, 99, 100, 101, 1 << 20, 1 << 31];
    let rows: Vec<[i64; 3]> = (0..10_000)
        .map(|_| {
            let (n1, n2) = (rng.pick(&sizes), rng.pick(&sizes));
            let mx = n1.max(n2);
            // ted at, around and past the threshold and past the max
            let teds = [
                0,
                mx * 15 / 100,
                mx * 15 / 100 + 1,
                mx * 15 / 100 - 1,
                mx,
                mx + 5,
                rng.next(64) as i64,
            ];
            [rng.pick(&teds).max(0), n1, n2]
        })
        .collect();
    let mut got = Vec::new();
    for chunk in rows.chunks(crate::tables::get().limits.clone.pair_cap) {
        got.extend(crate::dedup::t3::wire::decide(&mut link, chunk).expect("the core decides"));
    }
    let want: Vec<bool> = rows
        .iter()
        .map(|&[t, a, b]| oracle::is_clone(t, a, b))
        .collect();
    assert_eq!(got, want);
}

#[test]
fn the_label_bound_reads_the_frozen_intersection() {
    // the bound the core applies is Σ min(c1, c2) over equal labels;
    // two units that differ only in their histograms are kept or cut
    // exactly where the frozen verdict keeps or cuts them
    let mut link = core();
    let mut rng = Lcg(0x5EED_0002);
    let (mut kept, mut cut) = (0, 0);
    for n in 0..2_000 {
        let mut a = unit(&mut rng, true);
        let mut b = unit(&mut rng, true);
        (a.path, b.path, a.nodes, b.nodes) = ("a.rs".into(), "b.rs".into(), 100, 100);
        // b is a's histogram with up to 30 nodes moved between two of
        // its labels (I near the 85-node floor on both sides), or an
        // unrelated one
        let la = labels(&mut rng, 10);
        a.hist = split(&mut rng, &la, 100);
        b.hist = if rng.next(2) == 0 {
            let mut h = a.hist.clone();
            let (from, to) = (rng.next(h.len()), rng.next(h.len()));
            let moved = (rng.next(31) as u32).min(h[from].1 - 1);
            h[from].1 -= moved;
            h[to].1 += moved;
            h
        } else {
            let lb = labels(&mut rng, 10);
            split(&mut rng, &lb, 100)
        };
        let units = vec![a, b];
        let (want, _) = oracle::pass(&units, &[], &[], false);
        let facts = Facts {
            instances: &[],
            near: &[],
            exhaustive: false,
        };
        let got = pass(&mut link, &units, &facts, &mut Tally::default()).expect("the core answers");
        assert_eq!(
            pairs_of(&got),
            pairs_of(&want),
            "case {n}: {:?}",
            oracle::label_intersection(&units[0].hist, &units[1].hist)
        );
        if want.is_empty() {
            cut += 1;
        } else {
            kept += 1;
        }
    }
    assert!(
        kept > 0 && cut > 0,
        "the bound decided one way only: kept {kept}, cut {cut}"
    );
}
