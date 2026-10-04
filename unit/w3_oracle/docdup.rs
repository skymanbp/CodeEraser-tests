//! FROZEN ORACLE — the docdup coarse filter and verbatim-run measure as
//! the measuring side computed them at 093aede4, copied statement for
//! statement from cli/src/docdup/judge/candidates.rs (`collect`'s
//! coarse half, `group_pairs`, the over-cap tally), candidates/runs.rs
//! (`indexed`, `run_words`), docdup/judge/mod.rs (`is_dup`) and the
//! constants of docdup/judge/wire.rs (DOC_SET_CAP 8192, Jaccard
//! 80/100); MinHash and banding are the T3 oracle's (t3.rs, one
//! implementation then as now). Plan v2.33 W3 moved them into the core
//! (docpairs/1, the sequence shape of docdup/1); this copy stays as the
//! differential oracle (unit/docdup/judge/candidates.rs). Never edit
//! the bodies.

use super::t3::{HOT_GROUP_CAP, LSH_SHAPE, band_keys, signature};
use crate::docdup::judge::candidates::{SegRow, Tally};
use crate::docdup::spec;
use std::collections::{BTreeMap, BTreeSet};

pub const DOC_SET_CAP: usize = 8192;
pub const JACCARD_NUM: u64 = 80;
pub const JACCARD_DEN: u64 = 100;

/// `collect` up to the verbatim runs: the candidate pairs and the tally.
pub fn coarse(segs: &[SegRow]) -> (BTreeSet<(usize, usize)>, Tally) {
    let mut tally = Tally::default();
    let mut cand: BTreeSet<(usize, usize)> = BTreeSet::new();
    let (perms, bands, rows_per) = LSH_SHAPE;
    let mut buckets: BTreeMap<(usize, u64), Vec<usize>> = BTreeMap::new();
    let sendable: Vec<usize> = (0..segs.len())
        .filter(|&i| {
            let ok = segs[i].set.len() <= DOC_SET_CAP;
            tally.over_cap_segments += u64::from(!ok);
            ok
        })
        .collect();
    for &i in &sendable {
        let sig = signature(&segs[i].set, perms);
        for key in band_keys(&sig, bands, rows_per) {
            buckets.entry(key).or_default().push(i);
        }
    }
    tally.lsh_pairs = group_pairs(buckets.values(), &mut tally.hot_bands, &mut cand);
    let mut inverted: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for &i in &sendable {
        for h in &segs[i].set {
            inverted.entry(*h).or_default().push(i);
        }
    }
    tally.seed_pairs = group_pairs(inverted.values(), &mut tally.hot_shingles, &mut cand);
    (cand, tally)
}

fn group_pairs<'v>(
    groups: impl Iterator<Item = &'v Vec<usize>>,
    hot: &mut u64,
    cand: &mut BTreeSet<(usize, usize)>,
) -> u64 {
    let mut added = 0;
    for list in groups {
        let pairs: Vec<(usize, usize)> = if list.len() <= HOT_GROUP_CAP {
            (0..list.len())
                .flat_map(|a| ((a + 1)..list.len()).map(move |b| (list[a], list[b])))
                .collect()
        } else {
            *hot += 1;
            list.windows(2).map(|w| (w[0], w[1])).collect()
        };
        for (a, b) in pairs {
            if a != b && cand.insert((a.min(b), a.max(b))) {
                added += 1;
            }
        }
    }
    added
}

type Seq = (Vec<u64>, BTreeMap<u64, Vec<usize>>);

pub fn indexed(seq: Vec<u64>) -> Seq {
    let mut pos = BTreeMap::<u64, Vec<usize>>::new();
    for (j, &y) in seq.iter().enumerate() {
        pos.entry(y).or_default().push(j);
    }
    (seq, pos)
}

pub fn run_words((a, _): &Seq, (b, pos): &Seq) -> u64 {
    let mut best = 0usize;
    for (i, &x) in a.iter().enumerate() {
        for &j in pos.get(&x).map_or(&Vec::new(), |v| v) {
            if i > 0 && j > 0 && a[i - 1] == b[j - 1] {
                continue; // not a run start; counted from its start
            }
            let n = a[i..]
                .iter()
                .zip(&b[j..])
                .take_while(|(x2, y2)| x2 == y2)
                .count();
            best = best.max(n);
        }
    }
    if best == 0 {
        0
    } else {
        (best + spec::table().doc_shingle - 1) as u64
    }
}

pub fn is_dup(inter: u64, union: u64, verbatim: u64) -> bool {
    inter * JACCARD_DEN >= JACCARD_NUM * union || verbatim >= spec::table().verbatim_floor as u64
}
