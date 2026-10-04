//! FROZEN ORACLE — the advisor's ranking as the measuring side computed
//! it at 093aede4, copied statement for statement from
//! cli/src/similar/bm25.rs (`QueryTerm`, `Hit`, `Postings`, `top_k`,
//! `Corpus`, `query_of`, `shape_terms`, `role`, `idf_fp`,
//! `contribution`, `log2_fp` and the constants), similar/ppmi.rs
//! (`capped_words`, `Cooc`, `ppmi_fp`, `neighbours`, `expand`, `Table`
//! and the constants), terms.rs `Channel::weight` and wire.rs
//! `query_terms` (the bag similar/1 was sent). Plan v2.33 W3 moved the
//! ranking into the core (rank/1); this copy stays as the differential
//! oracle (unit/similar/rank_differential.rs) and as the tuning
//! instrument's formulas (it/similar_tune_parts/mirror.rs mounts this
//! same file), so it names the library through its parent's `lib`
//! alias — `crate` in the unit tree, `codeeraser` in the it crate.
//! Never edit the bodies.

use super::lib::similar::bag::UnitBag;
use super::lib::similar::corpus::Doc;
use super::lib::similar::terms::Channel;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub const IDF_FRAC_BITS: u32 = 8;
pub const SCORE_FRAC_BITS: u32 = 16;
pub const W_UNIT: i128 = 256;
pub const TOP_M: usize = 3;
pub const MIN_COOC: u32 = 2;
pub const MIN_PPMI: i128 = 2 << IDF_FRAC_BITS;
pub const PPMI_CAP: i128 = 4 << IDF_FRAC_BITS;
pub const PPMI_SCALE: i128 = 8 << IDF_FRAC_BITS;
pub const TERM_CAP: usize = 96;

/// terms.rs `Channel::weight`.
pub fn weight(ch: Channel) -> u32 {
    match ch {
        Channel::Name => 3,
        Channel::Callee => 2,
        _ => 1,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryTerm {
    pub term: u64,
    pub channel: Channel,
    pub weight: i128,
    pub spelled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub doc: usize,
    pub score: i64,
    pub score_fp: i64,
    pub hits: [u32; 6],
    pub shape_equal: bool,
    pub role: bool,
}

pub trait Postings {
    fn n_docs(&self) -> usize;
    fn avg_len(&self) -> i128;
    fn df(&self, term: u64) -> Result<usize>;
    fn posting(&self, term: u64) -> Result<Vec<(usize, u32)>>;
    fn len(&self, seat: usize) -> u32;
    fn shape(&self, seat: usize) -> Result<Vec<u64>>;
    fn identity(&self, seat: usize) -> (&str, &str, i64);
}

pub fn top_k(
    p: &impl Postings,
    query: &[QueryTerm],
    k: usize,
    exclude: Option<usize>,
) -> Result<Vec<Hit>> {
    let avg = p.avg_len();
    let mut acc: HashMap<usize, (i128, [u32; 6])> = HashMap::new();
    for q in query {
        let idf = idf_fp(p.n_docs(), p.df(q.term)?);
        if idf == 0 {
            continue;
        }
        for (seat, tf) in p.posting(q.term)? {
            let e = acc.entry(seat).or_insert((0, [0; 6]));
            let len = i128::from(p.len(seat));
            e.0 += contribution(q.weight, idf, i128::from(tf), len, avg);
            if q.spelled {
                e.1[q.channel.index()] += 1;
            }
        }
    }
    let mut ranked: Vec<(usize, i128, [u32; 6])> = acc
        .into_iter()
        .filter(|(seat, _)| Some(*seat) != exclude)
        .map(|(seat, (score, hits))| (seat, score, hits))
        .collect();
    ranked.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| p.identity(a.0).cmp(&p.identity(b.0)))
    });
    ranked.truncate(k);
    let shape = shape_terms(query);
    ranked
        .into_iter()
        .map(|(doc, score, hits)| {
            let shape_equal = p.shape(doc)? == shape;
            Ok(Hit {
                doc,
                score: i64::try_from(score >> SCORE_FRAC_BITS).expect("score fits i64"),
                score_fp: i64::try_from(score).expect("fixed-point score fits i64"),
                hits,
                shape_equal,
                role: role(&hits, shape_equal),
            })
        })
        .collect()
}

pub struct Corpus<'d> {
    pub docs: &'d [Doc],
    postings: HashMap<u64, Vec<(usize, u32)>>,
    total_len: u64,
}

impl<'d> Corpus<'d> {
    pub fn build(docs: &'d [Doc]) -> Corpus<'d> {
        let mut postings: HashMap<u64, Vec<(usize, u32)>> = HashMap::new();
        let mut total_len = 0u64;
        for (i, d) in docs.iter().enumerate() {
            total_len += u64::from(d.bag.len());
            for (term, (_, tf)) in &d.bag.terms {
                postings.entry(*term).or_default().push((i, *tf));
            }
        }
        Corpus {
            docs,
            postings,
            total_len,
        }
    }
}

impl Postings for Corpus<'_> {
    fn n_docs(&self) -> usize {
        self.docs.len()
    }

    fn avg_len(&self) -> i128 {
        (self.total_len / self.docs.len().max(1) as u64).max(1) as i128
    }

    fn df(&self, term: u64) -> Result<usize> {
        Ok(self.postings.get(&term).map_or(0, Vec::len))
    }

    fn posting(&self, term: u64) -> Result<Vec<(usize, u32)>> {
        Ok(self.postings.get(&term).cloned().unwrap_or_default())
    }

    fn len(&self, seat: usize) -> u32 {
        self.docs[seat].bag.len()
    }

    fn shape(&self, seat: usize) -> Result<Vec<u64>> {
        Ok(self.docs[seat].bag.channel(Channel::Shape))
    }

    fn identity(&self, seat: usize) -> (&str, &str, i64) {
        let d = &self.docs[seat];
        (&d.path, &d.bag.key, d.bag.nth)
    }
}

pub fn query_of(bag: &UnitBag) -> Vec<QueryTerm> {
    bag.terms
        .iter()
        .map(|(term, (channel, tf))| QueryTerm {
            term: *term,
            channel: *channel,
            weight: i128::from(*tf) * i128::from(weight(*channel)) * W_UNIT,
            spelled: true,
        })
        .collect()
}

fn shape_terms(query: &[QueryTerm]) -> Vec<u64> {
    let mut v: Vec<u64> = query
        .iter()
        .filter(|q| q.spelled && q.channel == Channel::Shape)
        .map(|q| q.term)
        .collect();
    v.sort_unstable();
    v
}

pub fn role(hits: &[u32; 6], shape_equal: bool) -> bool {
    let (n, c) = (hits[Channel::Name.index()], hits[Channel::Callee.index()]);
    (n >= 1 && c >= 1) || (n >= 2 && shape_equal)
}

pub fn idf_fp(n_docs: usize, df: usize) -> i128 {
    let num = 2 * n_docs as u128 + 1 - 2 * df as u128;
    let den = 2 * df as u128 + 1;
    if num <= den { 0 } else { log2_fp(num, den) }
}

pub fn contribution(w: i128, idf: i128, tf: i128, len: i128, avg: i128) -> i128 {
    let num = (w * idf * 22 * tf * avg) << SCORE_FRAC_BITS;
    let den = 10 * tf * avg + 3 * avg + 9 * len;
    num / den
}

pub fn log2_fp(num: u128, den: u128) -> i128 {
    debug_assert!(den > 0 && num >= den);
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

pub fn capped_words(bag: &UnitBag) -> (Vec<u64>, bool) {
    let mut words: Vec<u64> = bag
        .terms
        .iter()
        .filter(|(_, (ch, _))| ch.is_words())
        .map(|(term, _)| *term)
        .collect();
    let capped = words.len() > TERM_CAP;
    words.truncate(TERM_CAP);
    (words, capped)
}

pub trait Cooc {
    fn n_units(&self) -> u32;
    fn n_term(&self, a: u64) -> Result<u32>;
    fn pairs(&self, a: u64) -> Result<Vec<(u64, u32)>>;
}

fn ppmi_fp(n_units: u32, n_ab: u32, n_a: u32, n_b: u32) -> i128 {
    if n_ab < MIN_COOC {
        return 0;
    }
    let num = u128::from(n_ab) * u128::from(n_units);
    let den = u128::from(n_a) * u128::from(n_b);
    if num <= den { 0 } else { log2_fp(num, den) }
}

pub fn neighbours(c: &impl Cooc, a: u64) -> Result<Vec<(u64, i128)>> {
    let (n, n_a) = (c.n_units(), c.n_term(a)?);
    if n_a == 0 || 4 * n_a > n {
        return Ok(Vec::new());
    }
    let mut out: Vec<(u64, i128)> = Vec::new();
    for (b, n_ab) in c.pairs(a)? {
        if n_ab < MIN_COOC {
            continue;
        }
        let p = ppmi_fp(n, n_ab, n_a, c.n_term(b)?);
        if p >= MIN_PPMI {
            out.push((b, p));
        }
    }
    out.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
    out.truncate(TOP_M);
    Ok(out)
}

pub fn expand(c: &impl Cooc, query: &mut Vec<QueryTerm>) -> Result<()> {
    let spelled: BTreeSet<u64> = query.iter().map(|q| q.term).collect();
    let mut added: Vec<QueryTerm> = Vec::new();
    for q in query.iter().filter(|q| q.spelled && q.channel.is_words()) {
        for (term, ppmi) in neighbours(c, q.term)? {
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
    Ok(())
}

pub struct Table {
    n_docs: u32,
    n_term: HashMap<u64, u32>,
    n_pair: HashMap<(u64, u64), u32>,
    adjacent: HashMap<u64, BTreeSet<u64>>,
    pub capped_units: u32,
}

impl Table {
    pub fn build(docs: &[Doc]) -> Table {
        let mut t = Table {
            n_docs: docs.len() as u32,
            n_term: HashMap::new(),
            n_pair: HashMap::new(),
            adjacent: HashMap::new(),
            capped_units: 0,
        };
        for d in docs {
            let (words, capped) = capped_words(&d.bag);
            t.capped_units += u32::from(capped);
            t.count(&words);
        }
        t
    }

    fn count(&mut self, words: &[u64]) {
        for (i, &a) in words.iter().enumerate() {
            *self.n_term.entry(a).or_insert(0) += 1;
            for &b in &words[i + 1..] {
                *self.n_pair.entry((a, b)).or_insert(0) += 1;
                self.adjacent.entry(a).or_default().insert(b);
                self.adjacent.entry(b).or_default().insert(a);
            }
        }
    }

    fn n_pair(&self, a: u64, b: u64) -> u32 {
        let key = if a < b { (a, b) } else { (b, a) };
        self.n_pair.get(&key).copied().unwrap_or(0)
    }
}

impl Cooc for Table {
    fn n_units(&self) -> u32 {
        self.n_docs
    }

    fn n_term(&self, a: u64) -> Result<u32> {
        Ok(self.n_term.get(&a).copied().unwrap_or(0))
    }

    fn pairs(&self, a: u64) -> Result<Vec<(u64, u32)>> {
        Ok(self
            .adjacent
            .get(&a)
            .map(|adj| adj.iter().map(|&b| (b, self.n_pair(a, b))).collect())
            .unwrap_or_default())
    }
}

/// wire.rs `query_terms`: the bag similar/1 was sent.
pub fn query_terms(query: &[QueryTerm]) -> Vec<[u64; 2]> {
    let mut bag: BTreeMap<u64, u64> = BTreeMap::new();
    for q in query {
        let w = u64::try_from(q.weight).expect("query weight is positive and small");
        *bag.entry(q.term).or_default() += w;
    }
    bag.into_iter().map(|(term, w)| [term, w]).collect()
}
