//! The tuning instrument's mirror of the advisor's ranking: the integer
//! BM25, idf, fixed-point log2, role conjunction and PPMI widening as
//! the measuring side spelled them before plan v2.33 W3 moved them into
//! the core (CE.Similar.Rank). The formulas are the frozen oracle's
//! (unit/w3_oracle/similar.rs, mounted here as `oracle`) — one copy for
//! the differential gate and this instrument — and this file adapts them
//! to the library's in-memory corpus and co-occurrence table, which the
//! instrument re-ranks frozen pools over; `validate` makes the mirror
//! earn its baseline against what the core answered for every bare and
//! widened arm.
use super::oracle;
use anyhow::Result;
use codeeraser::similar::Channel;
use codeeraser::similar::corpus::Corpus;
use codeeraser::similar::ppmi::Cooc;
use codeeraser::similar::rank::Postings;
use std::collections::HashMap;

pub use oracle::{
    MIN_PPMI, PPMI_CAP, PPMI_SCALE, QueryTerm, SCORE_FRAC_BITS, W_UNIT, contribution, idf_fp,
    log2_fp, query_of, role,
};

/// One ranked seat and its score's integer part.
pub struct Hit {
    pub doc: usize,
    pub score: i64,
}

/// Names ×3, callees ×2, everything else ×1, at the instrument's width.
pub fn weight(ch: Channel) -> i128 {
    i128::from(oracle::weight(ch))
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
    strongest(acc.into_iter().filter(|(s, _)| Some(*s) != exclude), k)
        .into_iter()
        .map(|(doc, score)| Hit {
            doc,
            score: (score >> SCORE_FRAC_BITS) as i64,
        })
        .collect()
}

/// The `cap` strongest entries, by score descending then key ascending
/// — the one order the ranking, the neighbours and the feedback centroid
/// all cut by.
pub fn strongest<K: Ord>(items: impl IntoIterator<Item = (K, i128)>, cap: usize) -> Vec<(K, i128)> {
    let mut out: Vec<(K, i128)> = items.into_iter().collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    out.truncate(cap);
    out
}

/// The library's co-occurrence counts answer the oracle's three
/// questions unchanged (the trait the oracle froze is the library's).
impl<T: Cooc> oracle::Cooc for T {
    fn n_units(&self) -> u32 {
        Cooc::n_units(self)
    }
    fn n_term(&self, a: u64) -> Result<u32> {
        Cooc::n_term(self, a)
    }
    fn pairs(&self, a: u64) -> Result<Vec<(u64, u32)>> {
        Cooc::pairs(self, a)
    }
}

/// The top-m neighbours of `a` at or above MIN_PPMI.
pub fn neighbours(c: &impl Cooc, a: u64) -> Vec<(u64, i128)> {
    oracle::neighbours(c, a).expect("in-memory")
}

/// Widen a query in place by every spelled word term's neighbours.
pub fn expand(c: &impl Cooc, query: &mut Vec<QueryTerm>) {
    oracle::expand(c, query).expect("in-memory")
}
