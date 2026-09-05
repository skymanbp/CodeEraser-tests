//! Full-corpus field lengths, collection frequencies, and cached association.
use crate::similar_replay::Measured;
use codeeraser::similar::bm25;
use codeeraser::similar::{Channel, UnitBag};
use std::collections::BTreeMap;

pub struct Stats {
    pub lengths: Vec<[i128; 6]>,
    pub totals: [i128; 6],
    pub cf: BTreeMap<u64, i128>,
    pub channels: BTreeMap<u64, Channel>,
    pub neighbours: BTreeMap<u64, Vec<(u64, i128)>>,
    pub idfs: BTreeMap<u64, [i128; 3]>,
}

impl Stats {
    pub fn build(m: &Measured) -> Self {
        let mut s = Self {
            lengths: Vec::new(),
            totals: [0; 6],
            cf: BTreeMap::new(),
            channels: BTreeMap::new(),
            neighbours: BTreeMap::new(),
            idfs: BTreeMap::new(),
        };
        for d in &m.corpus.docs {
            let lens = channel_lengths(&d.bag);
            for (i, n) in lens.iter().enumerate() {
                s.totals[i] += n;
            }
            s.lengths.push(lens);
            for (t, (ch, tf)) in &d.bag.terms {
                *s.cf.entry(*t).or_default() += i128::from(*tf);
                s.channels.insert(*t, *ch);
            }
        }
        for (&t, ch) in &s.channels {
            if ch.is_words() {
                s.neighbours.insert(t, m.table.neighbours(t));
            }
            let (n, df) = (m.corpus.docs.len(), m.corpus.df(t));
            s.idfs.insert(
                t,
                [
                    bm25::idf_fp(n, df),
                    bm25::log2_fp(2 * n as u128 + 2, 2 * df as u128 + 1),
                    bm25::log2_fp((n + 1) as u128, (df + 1) as u128),
                ],
            );
        }
        s
    }

    pub fn selected(&self, weights: &[i128; 6], doc: Option<usize>) -> i128 {
        let lens = doc.map_or(&self.totals, |i| &self.lengths[i]);
        lens.iter()
            .zip(weights)
            .filter(|(_, w)| **w > 0)
            .map(|(n, _)| n)
            .sum()
    }
}

pub fn channel_lengths(bag: &UnitBag) -> [i128; 6] {
    let mut out = [0; 6];
    for (ch, tf) in bag.terms.values() {
        out[ch.index()] += i128::from(*tf);
    }
    out
}
