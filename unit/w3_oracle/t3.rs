//! FROZEN ORACLE — the T3 candidate pass as the measuring side computed
//! it at 093aede4, copied statement for statement: the four sources of
//! cli/src/dedup/sources.rs (`Gen`: `by_file`, `union`, `owner`, `push`,
//! `push_lines`, `same_key`, `fingerprint_pairs`, `structural_pairs`,
//! `band_group`; `near_pairs` takes the near runs the production side
//! still measures, sources.rs `near_runs`, instead of re-deriving them),
//! `pairs::each_hash_pair`, `minhash::{signature, band_keys}`,
//! `struct_fp::label_intersection`, candidates.rs `verdict` / `prune` /
//! `extend_exhaustive` / `s5_bucket`, and t3/mod.rs `is_clone`, with the
//! constants they read at the time (TSED 85/100, LSH (128, 32, 4), the
//! hot cap 64). Plan v2.33 W3 moved all of it into the core
//! (candidates/1, clone/1 `decide`); this copy stays as the
//! differential oracle (unit/dedup/candidate_wire.rs). Never edit the
//! bodies.

use crate::dedup::candidates::{PairRow, Tally, Unit};
use crate::dedup::index::Instance;
use crate::dedup::tokens::fnv1a;
use std::collections::{BTreeMap, BTreeSet};

pub const TSED_NUM: i64 = 85;
pub const TSED_DEN: i64 = 100;
pub const LSH_SHAPE: (usize, usize, usize) = (128, 32, 4);
pub const HOT_GROUP_CAP: usize = 64;

/// t3/mod.rs `is_clone`.
pub fn is_clone(ted: i64, n1: i64, n2: i64) -> bool {
    let mx = n1.max(n2);
    (mx - ted) * TSED_DEN >= TSED_NUM * mx
}

/// struct_fp.rs `label_intersection`.
pub fn label_intersection(h1: &[(u64, u32)], h2: &[(u64, u32)]) -> i64 {
    let (mut i, mut j, mut inter) = (0, 0, 0i64);
    while i < h1.len() && j < h2.len() {
        match h1[i].0.cmp(&h2[j].0) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                inter += h1[i].1.min(h2[j].1) as i64;
                i += 1;
                j += 1;
            }
        }
    }
    inter
}

/// minhash.rs `signature`.
pub fn signature(set: &[u64], perms: usize) -> Vec<u64> {
    (0..perms as u32)
        .map(|i| {
            set.iter()
                .map(|x| {
                    let mut bytes = [0u8; 12];
                    bytes[..8].copy_from_slice(&x.to_le_bytes());
                    bytes[8..].copy_from_slice(&i.to_le_bytes());
                    fnv1a(&bytes)
                })
                .min()
                .unwrap_or(u64::MAX)
        })
        .collect()
}

/// minhash.rs `band_keys`.
pub fn band_keys(sig: &[u64], bands: usize, rows: usize) -> Vec<(usize, u64)> {
    assert_eq!(
        sig.len(),
        bands * rows,
        "bands*rows must cover the signature"
    );
    (0..bands)
        .map(|b| {
            let mut bytes = Vec::with_capacity(8 * rows);
            for v in &sig[b * rows..(b + 1) * rows] {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            (b, fnv1a(&bytes))
        })
        .collect()
}

pub enum GroupEvent<'a> {
    Chained,
    Pair(&'a Instance, &'a Instance),
}

/// pairs.rs `each_hash_pair`.
pub fn each_hash_pair<'a>(instances: &'a [Instance], mut f: impl FnMut(GroupEvent<'a>)) {
    let mut by_hash: BTreeMap<u64, Vec<&Instance>> = BTreeMap::new();
    for inst in instances {
        by_hash.entry(inst.hash).or_default().push(inst);
    }
    for group in by_hash.values_mut().filter(|g| g.len() > 1) {
        if group.len() > HOT_GROUP_CAP {
            f(GroupEvent::Chained);
            group.sort_by(|x, y| (&x.file, x.start_tok).cmp(&(&y.file, y.start_tok)));
            for w in group.windows(2) {
                f(GroupEvent::Pair(w[0], w[1]));
            }
        } else {
            for (i, a) in group.iter().enumerate() {
                for b in &group[i + 1..] {
                    f(GroupEvent::Pair(a, b));
                }
            }
        }
    }
}

pub struct Gen<'u> {
    units: &'u [Unit],
    by_file: BTreeMap<&'u str, Vec<usize>>,
    pub tally: Tally,
}

pub fn by_file(units: &[Unit]) -> BTreeMap<&str, Vec<usize>> {
    let mut out: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (id, u) in units.iter().enumerate() {
        out.entry(&u.path).or_default().push(id);
    }
    out
}

impl<'u> Gen<'u> {
    pub fn new(units: &'u [Unit]) -> Self {
        Gen {
            units,
            by_file: by_file(units),
            tally: Tally::default(),
        }
    }

    pub fn union(
        &mut self,
        near: &[[(String, i64); 2]],
        instances: &[Instance],
    ) -> BTreeMap<(usize, usize), u8> {
        let sets = [
            self.near_pairs(near),
            self.same_key(),
            self.fingerprint_pairs(instances),
            self.structural_pairs(),
        ];
        let mut union: BTreeMap<(usize, usize), u8> = BTreeMap::new();
        for (i, set) in sets.into_iter().enumerate() {
            for p in set {
                *union.entry(p).or_insert(0) |= 1 << i;
            }
        }
        self.tally.union_pairs = union.len() as u64;
        union
    }

    fn owner(&self, file: &str, line: i64) -> Option<usize> {
        self.by_file
            .get(file)?
            .iter()
            .copied()
            .filter(|&id| self.units[id].start_line <= line && line <= self.units[id].end_line)
            .min_by_key(|&id| self.units[id].end_line - self.units[id].start_line)
    }

    fn push(&mut self, set: &mut BTreeSet<(usize, usize)>, x: usize, y: usize, src: &str) {
        if x == y {
            self.tally.self_pair_dropped += 1;
            return;
        }
        let (a, b) = (x.min(y), x.max(y));
        if self.units[a].lang != self.units[b].lang {
            self.tally.cross_lang_dropped += 1;
            return;
        }
        if set.insert((a, b)) {
            let key = format!("{src}/{}", self.units[a].lang);
            *self.tally.raw_by.entry(key).or_insert(0) += 1;
        }
    }

    fn push_lines(&mut self, set: &mut BTreeSet<(usize, usize)>, ab: [(&str, i64); 2], src: &str) {
        match (self.owner(ab[0].0, ab[0].1), self.owner(ab[1].0, ab[1].1)) {
            (Some(x), Some(y)) => self.push(set, x, y, src),
            _ => self.tally.unowned_dropped += 1,
        }
    }

    fn same_key(&mut self) -> BTreeSet<(usize, usize)> {
        let mut by_key: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
        for (id, u) in self.units.iter().enumerate() {
            by_key.entry(&u.key).or_default().push(id);
        }
        let mut out = BTreeSet::new();
        for ids in by_key.values().filter(|ids| ids.len() > 1) {
            for (i, &a) in ids.iter().enumerate() {
                for &b in &ids[i + 1..] {
                    if self.units[a].path != self.units[b].path {
                        self.push(&mut out, a, b, "s2");
                    }
                }
            }
        }
        out
    }

    fn fingerprint_pairs(&mut self, instances: &[Instance]) -> BTreeSet<(usize, usize)> {
        let mut out = BTreeSet::new();
        each_hash_pair(instances, |ev| match ev {
            GroupEvent::Chained => self.tally.s3_hot_chained += 1,
            GroupEvent::Pair(a, b) => self.push_lines(
                &mut out,
                [
                    (&a.file, a.start_line as i64),
                    (&b.file, b.start_line as i64),
                ],
                "s3",
            ),
        });
        out
    }

    /// `near_pairs` over the near runs as measured.
    fn near_pairs(&mut self, near: &[[(String, i64); 2]]) -> BTreeSet<(usize, usize)> {
        let mut out = BTreeSet::new();
        for [(fa, la), (fb, lb)] in near {
            self.push_lines(&mut out, [(fa, *la), (fb, *lb)], "s1");
        }
        out
    }

    fn structural_pairs(&mut self) -> BTreeSet<(usize, usize)> {
        let mut buckets: BTreeMap<(usize, u64), Vec<usize>> = BTreeMap::new();
        for (id, u) in self.units.iter().enumerate() {
            assert!(
                !u.sig.is_empty(),
                "{}: admitted unit with empty shingles",
                u.path
            );
            let sig = signature(&u.sig, LSH_SHAPE.0);
            for key in band_keys(&sig, LSH_SHAPE.1, LSH_SHAPE.2) {
                buckets.entry(key).or_default().push(id);
            }
        }
        let mut out = BTreeSet::new();
        for ids in buckets.values().filter(|ids| ids.len() > 1) {
            self.band_group(&mut out, ids);
        }
        out
    }

    fn band_group(&mut self, out: &mut BTreeSet<(usize, usize)>, ids: &[usize]) {
        *self
            .tally
            .s4_band_groups
            .entry(ids.len() as u64)
            .or_insert(0) += 1;
        if ids.len() > HOT_GROUP_CAP {
            self.tally.s4_hot_chained += 1;
            for w in ids.windows(2) {
                self.push(out, w[0], w[1], "s4");
            }
        } else {
            for (i, &a) in ids.iter().enumerate() {
                for &b in &ids[i + 1..] {
                    self.push(out, a, b, "s4");
                }
            }
        }
    }
}

enum Verdict {
    Size,
    Label,
    Keep,
}

fn verdict(a: &Unit, b: &Unit) -> Verdict {
    let mx = a.nodes.max(b.nodes);
    let bounds = [
        (a.nodes.min(b.nodes), Verdict::Size),
        (label_intersection(&a.hist, &b.hist), Verdict::Label),
    ];
    for (q, v) in bounds {
        if q * TSED_DEN < TSED_NUM * mx {
            return v;
        }
    }
    Verdict::Keep
}

pub fn prune(
    units: &[Unit],
    union: BTreeMap<(usize, usize), u8>,
    tally: &mut Tally,
) -> Vec<PairRow> {
    let mut out = Vec::new();
    for ((a, b), sources) in union {
        match verdict(&units[a], &units[b]) {
            Verdict::Size => tally.pruned_size += 1,
            Verdict::Label => tally.pruned_label += 1,
            Verdict::Keep => out.push(PairRow { a, b, sources }),
        }
    }
    tally.survivors = out.len() as u64;
    out
}

/// candidates.rs `Candidates`, the frame `extend_exhaustive` read (the
/// units borrowed rather than owned — the oracle never builds an index).
pub struct Candidates<'u> {
    pub units: &'u [Unit],
    pub pairs: Vec<PairRow>,
    pub tally: Tally,
}

/// candidates.rs `extend_exhaustive`.
pub fn extend_exhaustive(c: &mut Candidates) {
    let have: std::collections::BTreeSet<(usize, usize)> =
        c.pairs.iter().map(|p| (p.a, p.b)).collect();
    let mut by_lang: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (id, u) in c.units.iter().enumerate() {
        by_lang.entry(&u.lang).or_default().push(id);
    }
    let mut fresh = Vec::new();
    for ids in by_lang.values_mut() {
        ids.sort_by_key(|&id| c.units[id].nodes);
        s5_bucket(c.units, ids, &have, &mut c.tally, &mut fresh);
    }
    fresh.sort_unstable();
    fresh.dedup();
    c.tally.s5_new = fresh.len() as u64;
    c.pairs.extend(fresh.into_iter().map(|(a, b)| PairRow {
        a,
        b,
        sources: 1 << 4,
    }));
    // the clone wire refuses non-ascending pair rows (the frozen
    // boundary contract) — appended S5 rows must fold back into the
    // canonical order
    c.pairs.sort_by_key(|p| (p.a, p.b));
}

fn s5_bucket(
    units: &[Unit],
    ids: &[usize],
    have: &BTreeSet<(usize, usize)>,
    tally: &mut Tally,
    fresh: &mut Vec<(usize, usize)>,
) {
    for (i, &a) in ids.iter().enumerate() {
        for &b in &ids[i + 1..] {
            if units[a].nodes * TSED_DEN < TSED_NUM * units[b].nodes {
                break;
            }
            tally.s5_windowed += 1;
            let pair = (a.min(b), a.max(b));
            if have.contains(&pair) {
                tally.s5_already += 1;
            } else if matches!(verdict(&units[a], &units[b]), Verdict::Label) {
                tally.s5_pruned_label += 1;
            } else {
                fresh.push(pair);
            }
        }
    }
}

/// The whole frozen pass: union, prune, then S5 when asked.
pub fn pass(
    units: &[Unit],
    near: &[[(String, i64); 2]],
    instances: &[Instance],
    exhaustive: bool,
) -> (Vec<PairRow>, Tally) {
    let mut g = Gen::new(units);
    let union = g.union(near, instances);
    let mut tally = g.tally;
    let pairs = prune(units, union, &mut tally);
    let mut c = Candidates {
        units,
        pairs,
        tally,
    };
    if exhaustive {
        extend_exhaustive(&mut c);
    }
    (c.pairs, c.tally)
}
