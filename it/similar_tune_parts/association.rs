//! Association supplies score only; no synthetic term enters spelled evidence.
use super::config::Config;
use super::feedback;
use super::stats::Stats;
use crate::similar_replay::Measured;
use codeeraser::similar::{Channel, bm25::QueryTerm, ppmi};
use std::collections::{BTreeMap, BTreeSet};

pub fn expanded(
    m: &Measured,
    s: &Stats,
    doc: usize,
    bare: &[QueryTerm],
    c: &Config,
) -> Vec<QueryTerm> {
    let mut q = bare.to_vec();
    let kind = c.text("assoc", "none");
    if kind == "v1" {
        ppmi::expand(&m.table, &mut q).expect("in-memory");
        return q;
    }
    if kind == "none" || !gate(s, bare, c.text("gate", "always")) {
        return q;
    }
    let mut added = match kind {
        "rocchio" | "rm3" => feedback::terms(m, s, doc, bare, c),
        "second" => second(s, bare, c),
        "ppmi" => first(s, bare, c),
        _ => panic!("unknown association {kind}"),
    };
    if c.int("mass", 0) > 0 {
        let limit = word_mass(s, bare) / c.int("mass", 1);
        let mass: i128 = added.iter().map(|t| t.weight).sum();
        if mass > limit {
            for t in &mut added {
                t.weight = t.weight * limit / mass;
            }
        }
    }
    q.extend(added.into_iter().filter(|t| t.weight > 0));
    q
}

fn word_mass(s: &Stats, q: &[QueryTerm]) -> i128 {
    q.iter()
        .filter(|t| t.channel.is_words() && s.idfs[&t.term][0] > 0)
        .map(|t| t.weight)
        .sum()
}

fn gate(s: &Stats, q: &[QueryTerm], name: &str) -> bool {
    let count = q
        .iter()
        .filter(|t| t.channel.is_words() && s.idfs[&t.term][0] > 0)
        .count();
    let total: i128 = q
        .iter()
        .filter(|t| s.idfs[&t.term][0] > 0)
        .map(|t| t.weight)
        .sum();
    match name {
        "terms2" => count <= 2,
        "terms5" => count <= 5,
        "lowmass" => word_mass(s, q) * 2 <= total,
        "highmass" => word_mass(s, q) * 2 > total,
        _ => true,
    }
}

fn first(s: &Stats, q: &[QueryTerm], c: &Config) -> Vec<QueryTerm> {
    let mut seen: BTreeSet<_> = q.iter().map(|t| t.term).collect();
    let mut out = Vec::new();
    for parent in q.iter().filter(|t| t.channel.is_words()) {
        for &(term, p) in s.neighbours[&parent.term]
            .iter()
            .filter(|(_, p)| *p >= c.int("min", ppmi::MIN_PPMI))
            .take(c.int("m", 3) as usize)
        {
            if seen.insert(term) {
                out.push(QueryTerm {
                    term,
                    channel: s.channels[&term],
                    spelled: false,
                    weight: parent.weight * p.min(ppmi::PPMI_CAP)
                        / c.int("scale", ppmi::PPMI_SCALE),
                });
            }
        }
    }
    out
}

fn second(s: &Stats, q: &[QueryTerm], c: &Config) -> Vec<QueryTerm> {
    let spelled: BTreeSet<_> = q.iter().map(|t| t.term).collect();
    let mut added = BTreeMap::<u64, i128>::new();
    for parent in q.iter().filter(|t| t.channel.is_words()) {
        let mut paths = BTreeMap::<u64, i128>::new();
        for &(via, a) in &s.neighbours[&parent.term] {
            if s.channels[&via] != Channel::Callee {
                continue;
            }
            for &(term, b) in &s.neighbours[&via] {
                if spelled.contains(&term) {
                    continue;
                }
                let w = parent.weight * a.min(ppmi::PPMI_CAP) * b.min(ppmi::PPMI_CAP)
                    / (ppmi::PPMI_SCALE * ppmi::PPMI_SCALE);
                let best = paths.entry(term).or_default();
                *best = (*best).max(w);
            }
        }
        for (term, weight) in feedback::strongest(paths, c.int("m", 3) as usize) {
            let best = added.entry(term).or_default();
            *best = (*best).max(weight);
        }
    }
    added
        .into_iter()
        .map(|(term, weight)| QueryTerm {
            term,
            weight,
            channel: s.channels[&term],
            spelled: false,
        })
        .collect()
}
