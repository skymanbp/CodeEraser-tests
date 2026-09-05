//! Row-stochastic PPMI translation, evaluated as a smoothed query likelihood.
use super::config::Config;
use super::score::likelihood;
use super::stats::Stats;
use crate::similar_replay::Measured;
use codeeraser::similar::bm25::QueryTerm;
use std::collections::BTreeMap;

const UNIT: i128 = 4096;
pub struct Kernel(BTreeMap<u64, Vec<(u64, i128)>>);

impl Kernel {
    pub fn build(s: &Stats, mix: i128) -> Self {
        let mut incoming = BTreeMap::<u64, Vec<(u64, i128)>>::new();
        for (&source, neighbours) in &s.neighbours {
            let total: i128 = neighbours.iter().map(|(_, p)| p).sum();
            let mut sent = 0;
            for &(target, p) in neighbours {
                let weight = UNIT * p / (mix * total);
                sent += weight;
                incoming.entry(target).or_default().push((source, weight));
            }
            incoming
                .entry(source)
                .or_default()
                .push((source, UNIT - sent));
            assert!(sent <= UNIT / mix);
        }
        Self(incoming)
    }

    pub fn query(&self, s: &Stats, q: &[QueryTerm]) -> Vec<Term> {
        q.iter()
            .map(|t| {
                let sources = self.0[&t.term].clone();
                let cf = sources.iter().map(|(t, w)| s.cf[t] * w).sum();
                Term {
                    weight: t.weight,
                    cf,
                    sources,
                }
            })
            .collect()
    }
}

pub struct Term {
    weight: i128,
    cf: i128,
    sources: Vec<(u64, i128)>,
}

pub fn score(m: &Measured, s: &Stats, q: &[Term], doc: usize, c: &Config) -> i128 {
    let terms = q.iter().map(|q| {
        let mass: i128 = q
            .sources
            .iter()
            .map(|(t, w)| {
                m.corpus.docs[doc]
                    .bag
                    .terms
                    .get(t)
                    .map_or(0, |(_, tf)| i128::from(*tf) * w)
            })
            .sum();
        (q.weight, mass, q.cf)
    });
    likelihood(s, doc, c, terms)
}
