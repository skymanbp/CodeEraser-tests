//! The tuning instrument's own mirror of the advisor's ranking: the
//! integer BM25, idf, fixed-point log2, role conjunction and PPMI
//! widening as the measuring side spelled them before plan v2.33 W3
//! moved them into the core (CE.Similar.Rank). The instrument re-ranks
//! frozen pools under variants of exactly these formulas, so it keeps
//! them here — and `validate` makes the mirror earn its baseline against
//! what the core answered for every bare and widened arm.
use codeeraser::similar::corpus::{Corpus, query_of as spelled};
use codeeraser::similar::ppmi::Cooc;
use codeeraser::similar::rank::Postings;
use codeeraser::similar::{Channel, UnitBag};
use std::collections::{BTreeSet, HashMap};

pub const IDF_FRAC_BITS: u32 = 8;
pub const SCORE_FRAC_BITS: u32 = 16;
pub const W_UNIT: i128 = 256;
pub const TOP_M: usize = 3;
pub const MIN_COOC: u32 = 2;
pub const MIN_PPMI: i128 = 2 << IDF_FRAC_BITS;
pub const PPMI_CAP: i128 = 4 << IDF_FRAC_BITS;
pub const PPMI_SCALE: i128 = 8 << IDF_FRAC_BITS;

/// One query term with its weight in 1/W_UNIT; `spelled` = false marks
/// an expansion (score, never evidence).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryTerm {
    pub term: u64,
    pub channel: Channel,
    pub weight: i128,
    pub spelled: bool,
}

/// One ranked seat and its score's integer part.
pub struct Hit {
    pub doc: usize,
    pub score: i64,
}

/// Names ×3, callees ×2, everything else ×1.
pub fn weight(ch: Channel) -> i128 {
    match ch {
        Channel::Name => 3,
        Channel::Callee => 2,
        _ => 1,
    }
}

/// A bag as a weighted query: tf × channel weight × W_UNIT.
pub fn query_of(bag: &UnitBag) -> Vec<QueryTerm> {
    spelled(bag)
        .into_iter()
        .map(|q| QueryTerm {
            term: q.term,
            channel: q.channel,
            weight: i128::from(q.tf) * weight(q.channel) * W_UNIT,
            spelled: true,
        })
        .collect()
}

/// The top `k` seats for `query` over the in-memory corpus, the query's
/// own seat out, by score then seat (seat order is identity order).
pub fn top_k(c: &Corpus, query: &[QueryTerm], k: usize, exclude: Option<usize>) -> Vec<Hit> {
    let avg = c.avg_len();
    let mut acc: HashMap<usize, i128> = HashMap::new();
    for q in query {
        let idf = idf_fp(c.n_docs(), c.df(q.term).expect("in-memory"));
        if idf == 0 {
            continue;
        }
        for (seat, tf) in c.posting(q.term).expect("in-memory") {
            let len = i128::from(c.len(seat));
            *acc.entry(seat).or_insert(0) += contribution(q.weight, idf, i128::from(tf), len, avg);
        }
    }
    let mut ranked: Vec<(usize, i128)> = acc
        .into_iter()
        .filter(|(s, _)| Some(*s) != exclude)
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked.truncate(k);
    ranked
        .into_iter()
        .map(|(doc, score)| Hit {
            doc,
            score: (score >> SCORE_FRAC_BITS) as i64,
        })
        .collect()
}

/// The same-role conjunction over an evidence row.
pub fn role(hits: &[u32; 6], shape_equal: bool) -> bool {
    let (n, c) = (hits[0], hits[2]);
    (n >= 1 && c >= 1) || (n >= 2 && shape_equal)
}

/// idf in IDF_FRAC_BITS fixed point, floored at zero.
pub fn idf_fp(n_docs: usize, df: usize) -> i128 {
    let num = 2 * n_docs as u128 + 1 - 2 * df as u128;
    let den = 2 * df as u128 + 1;
    if num <= den { 0 } else { log2_fp(num, den) }
}

/// One term's contribution: k1 = 6/5 and b = 3/4 expanded to
/// 22·tf·avg / (10·tf·avg + 3·avg + 9·len), floored in fixed point.
pub fn contribution(w: i128, idf: i128, tf: i128, len: i128, avg: i128) -> i128 {
    ((w * idf * 22 * tf * avg) << SCORE_FRAC_BITS) / (10 * tf * avg + 3 * avg + 9 * len)
}

/// floor(2^IDF_FRAC_BITS · log2(num/den)) by integer squaring, operands
/// halved together below 2^62.
pub fn log2_fp(num: u128, den: u128) -> i128 {
    let (mut n, mut d, mut int_part) = (num, den, 0i128);
    while n >= 2 * d {
        d <<= 1;
        int_part += 1;
    }
    let mut frac = 0i128;
    for _ in 0..IDF_FRAC_BITS {
        while n >= 1 << 62 || d >= 1 << 62 {
            n >>= 1;
            d >>= 1;
        }
        n *= n;
        d *= d;
        frac <<= 1;
        if n >= 2 * d {
            d <<= 1;
            frac |= 1;
        }
    }
    (int_part << IDF_FRAC_BITS) | frac
}

fn ppmi_fp(n: u32, n_ab: u32, n_a: u32, n_b: u32) -> i128 {
    let (num, den) = (
        u128::from(n_ab) * u128::from(n),
        u128::from(n_a) * u128::from(n_b),
    );
    if n_ab < MIN_COOC || num <= den {
        0
    } else {
        log2_fp(num, den)
    }
}

/// The top-m neighbours of `a` at or above MIN_PPMI.
pub fn neighbours(c: &impl Cooc, a: u64) -> Vec<(u64, i128)> {
    let (n, n_a) = (c.n_units(), c.n_term(a).expect("in-memory"));
    if n_a == 0 || 4 * n_a > n {
        return Vec::new();
    }
    let mut out: Vec<(u64, i128)> = c
        .pairs(a)
        .expect("in-memory")
        .into_iter()
        .filter(|(_, n_ab)| *n_ab >= MIN_COOC)
        .map(|(b, n_ab)| (b, ppmi_fp(n, n_ab, n_a, c.n_term(b).expect("in-memory"))))
        .filter(|(_, p)| *p >= MIN_PPMI)
        .collect();
    out.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
    out.truncate(TOP_M);
    out
}

/// Widen a query in place by every spelled word term's neighbours.
pub fn expand(c: &impl Cooc, query: &mut Vec<QueryTerm>) {
    let spelled: BTreeSet<u64> = query.iter().map(|q| q.term).collect();
    let mut added: Vec<QueryTerm> = Vec::new();
    for q in query.iter().filter(|q| q.spelled && q.channel.is_words()) {
        for (term, ppmi) in neighbours(c, q.term) {
            if spelled.contains(&term) || added.iter().any(|a| a.term == term) {
                continue;
            }
            added.push(QueryTerm {
                term,
                channel: q.channel,
                weight: q.weight * ppmi.min(PPMI_CAP) / PPMI_SCALE,
                spelled: false,
            });
        }
    }
    query.extend(added);
}
