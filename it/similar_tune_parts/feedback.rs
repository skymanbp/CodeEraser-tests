//! Positive Rocchio and RM3-style feedback from full-corpus bare top-k, never labels.
use super::config::Config;
use super::stats::Stats;
use crate::similar_replay::Measured;
use codeeraser::similar::bm25::QueryTerm;
use std::collections::{BTreeMap, BTreeSet};

pub fn terms(m: &Measured, s: &Stats, doc: usize, q: &[QueryTerm], c: &Config) -> Vec<QueryTerm> {
    let spelled: BTreeSet<_> = q.iter().map(|t| t.term).collect();
    let feedback = &m.ranked[doc].0;
    let cap = c.int("feedback", 3) as usize;
    let mass: i128 = q
        .iter()
        .filter(|t| t.channel.is_words() && s.idfs[&t.term][0] > 0)
        .map(|t| t.weight)
        .sum();
    let mut centroid = BTreeMap::<u64, i128>::new();
    for hit in feedback.iter().take(cap) {
        let len: i128 = m.corpus.docs[hit.doc]
            .bag
            .terms
            .values()
            .filter(|(ch, _)| ch.is_words())
            .map(|(_, tf)| i128::from(*tf))
            .sum();
        for (term, (ch, tf)) in &m.corpus.docs[hit.doc].bag.terms {
            if !ch.is_words() || spelled.contains(term) || s.idfs[term][0] == 0 {
                continue;
            }
            let source = if c.text("assoc", "") == "rm3" {
                i128::from(hit.score).max(0)
            } else {
                s.idfs[term][0]
            };
            *centroid.entry(*term).or_default() += ((source * i128::from(*tf)) << 16) / len.max(1);
        }
    }
    let selected = strongest(centroid, 8);
    let total: i128 = selected.iter().map(|(_, w)| w).sum();
    selected
        .into_iter()
        .map(|(term, w)| QueryTerm {
            term,
            weight: mass * w / total.max(1) / c.int("mass", 4),
            channel: s.channels[&term],
            spelled: false,
        })
        .collect()
}

pub fn strongest(map: BTreeMap<u64, i128>, cap: usize) -> Vec<(u64, i128)> {
    let mut out: Vec<_> = map.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    out.truncate(cap);
    out
}
