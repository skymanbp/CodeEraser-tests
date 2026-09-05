//! Test-only integer estimators. Production bags and statistics remain authoritative.
use super::config::Config;
use super::stats::Stats;
use crate::similar_replay::Measured;
use codeeraser::similar::bm25::{self, Postings, QueryTerm};

pub fn query(m: &Measured, s: &Stats, doc: usize, c: &Config) -> Vec<QueryTerm> {
    let weights = c.weights();
    let mut q = m.corpus.query_of(doc);
    for t in &mut q {
        let tf = t.weight / (i128::from(t.channel.weight()) * bm25::W_UNIT);
        t.weight = tf.min(c.int("qcap", i128::MAX)) * weights[t.channel.index()] * bm25::W_UNIT;
    }
    q.retain(|t| {
        t.weight > 0
            && m.corpus.df(t.term).expect("in-memory") as i128 * c.int("dfdiv", 0)
                <= m.corpus.docs.len() as i128
    });
    if c.int("select", 0) > 0 {
        q.sort_by_key(|t| (std::cmp::Reverse(t.weight * idf(s, t.term, c)), t.term));
        q.truncate(c.int("select", 0) as usize);
    }
    q.sort_by_key(|t| t.term);
    q
}

fn idf(s: &Stats, term: u64, c: &Config) -> i128 {
    let i = match c.text("idf", "rsi") {
        "positive" => 1,
        "ndf" => 2,
        _ => 0,
    };
    s.idfs[&term][i]
}

pub fn bm_fraction(c: &Config, tf: i128, len: i128, avg: i128) -> (i128, i128) {
    let ((kn, kd), (bn, bd)) = (c.ratio("k", "6/5"), c.ratio("b", "3/4"));
    (
        (kn + kd) * tf * bd * avg,
        tf * kd * bd * avg + kn * ((bd - bn) * avg + bn * len),
    )
}

pub fn signed_log(num: i128, den: i128) -> i128 {
    assert!(num > 0 && den > 0);
    if num >= den {
        bm25::log2_fp(num as u128, den as u128)
    } else {
        -bm25::log2_fp(den as u128, num as u128)
    }
}

pub fn score(m: &Measured, s: &Stats, q: &[QueryTerm], doc: usize, c: &Config) -> i128 {
    let model = c.text("model", "bm");
    if model == "lm" {
        return language_model(m, s, q, doc, c);
    }
    if model == "jaccard" || model == "cosine" {
        return similarity(m, s, q, doc, c);
    }
    let weights = c.weights();
    let active = c.text("length", "all") == "active";
    let total = if active {
        s.selected(&weights, None)
    } else {
        s.totals.iter().sum()
    };
    let length = if active {
        s.selected(&weights, Some(doc))
    } else {
        i128::from(m.corpus.docs[doc].bag.len())
    };
    let avg = (total / m.corpus.docs.len().max(1) as i128).max(1);
    let mut score = 0;
    for t in q {
        let Some((_, tf)) = m.corpus.docs[doc].bag.terms.get(&t.term) else {
            continue;
        };
        let ch = s.channels[&t.term].index();
        let (len, av) = if model == "field" {
            (
                s.lengths[doc][ch],
                (s.totals[ch] / m.corpus.docs.len() as i128).max(1),
            )
        } else {
            (length, avg)
        };
        let (num, den) = bm_fraction(c, i128::from(*tf), len, av);
        score += ((t.weight * idf(s, t.term, c) * num) << bm25::SCORE_FRAC_BITS) / den;
    }
    score >> bm25::SCORE_FRAC_BITS
}

fn language_model(m: &Measured, s: &Stats, q: &[QueryTerm], doc: usize, c: &Config) -> i128 {
    let terms = q.iter().map(|t| {
        let tf = m.corpus.docs[doc]
            .bag
            .terms
            .get(&t.term)
            .map_or(0, |(_, tf)| i128::from(*tf));
        (t.weight, tf, s.cf[&t.term])
    });
    likelihood(s, doc, c, terms)
}

pub fn likelihood(
    s: &Stats,
    doc: usize,
    c: &Config,
    terms: impl Iterator<Item = (i128, i128, i128)>,
) -> i128 {
    let weights = c.weights();
    let (total, len) = (s.selected(&weights, None), s.selected(&weights, Some(doc)));
    let mu = c.int("mu", 100);
    terms
        .map(|(weight, tf, cf)| weight * signed_log(tf * total + mu * cf, (len + mu) * cf))
        .sum()
}

fn similarity(m: &Measured, s: &Stats, q: &[QueryTerm], doc: usize, c: &Config) -> i128 {
    let weights = c.weights();
    let terms = &m.corpus.docs[doc].bag.terms;
    let (mut dot, mut qnorm, mut dnorm, mut intersection, mut union) = (0, 0, 0, 0, 0);
    for t in q {
        let weight = idf(s, t.term, c);
        let x = t.weight / bm25::W_UNIT;
        let y = terms
            .get(&t.term)
            .map_or(0, |(ch, tf)| i128::from(*tf) * weights[ch.index()]);
        dot += weight * x * y;
        qnorm += weight * x * x;
        intersection += weight * i128::from(y > 0) * weights[t.channel.index()];
        union += weight * weights[t.channel.index()];
    }
    for (term, (ch, tf)) in terms {
        let y = i128::from(*tf) * weights[ch.index()];
        let w = idf(s, *term, c);
        dnorm += w * y * y;
        if !q.iter().any(|t| t.term == *term) {
            union += w * weights[ch.index()];
        }
    }
    if c.text("model", "") == "jaccard" {
        (intersection << 24) / union.max(1)
    } else {
        ((dot * dot) << 24) / (qnorm * dnorm).max(1)
    }
}
