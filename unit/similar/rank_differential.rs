//! Differential gate for the advisor's ranking move into the core (plan
//! v2.33 W3, user instruction 2026-10-03): seeded random corpora and
//! queries through (a) the frozen Rust oracle (unit/w3_oracle/
//! similar.rs, the 093aede4 code) and (b) rank/1 then similar/1 over
//! the real core wire — the kept hits (seat, score, fixed-point score,
//! channel evidence, shape bit), the weighted bag, the expansion, the
//! widened arm and the role bits, all equal. Edges: one unit, identical
//! bags (score ties broken by identity), a query that is its own seat
//! and one that is no seat, k from 1 to past the corpus, u64 term
//! hashes at and above 2^63, a unit with more word terms than the PPMI
//! cap, words in more than a quarter of the units (no neighbours).

use super::{Ask, bare, cooc_rows, widened};
use crate::similar::bag::UnitBag;
use crate::similar::corpus::{Corpus, Doc, query_of};
use crate::similar::ppmi::Table;
use crate::similar::terms::Channel;
use crate::w3_oracle::{Lcg, core_link, similar as oracle};

/// Term hashes, each bound to one channel (a term hash carries its
/// channel tag in the product).
fn term(i: usize) -> (u64, Channel) {
    let base = [3u64, 1 << 63, u64::MAX - 7, 0x9E37_79B9_7F4A_7C15, 1 << 40];
    (
        base[i % 5].wrapping_add(i as u64 * 1_000_003),
        Channel::ALL[i % 6],
    )
}

fn bag(rng: &mut Lcg, key: String, terms: usize, wide: bool) -> UnitBag {
    let mut b = UnitBag::empty(key, 0);
    let count = if wide { 120 } else { 1 + rng.next(9) };
    for _ in 0..count {
        let i = if wide { rng.next(400) } else { rng.next(terms) };
        let (t, ch) = term(i);
        b.terms.insert(t, (ch, 1 + rng.next(4) as u32));
    }
    b
}

fn corpus(rng: &mut Lcg, n: usize) -> Vec<Doc> {
    let size = match n % 6 {
        0 => 1,
        1 => 8 + rng.next(30),
        _ => 2 + rng.next(30),
    };
    let terms = 4 + rng.next(30);
    let same = n % 6 == 2;
    let shared = bag(rng, String::new(), terms, false);
    (0..size)
        .map(|i| {
            let key = format!("u{i:03}");
            let mut b = if same {
                shared.clone()
            } else {
                bag(rng, key.clone(), terms, n % 17 == 5 && i == 0)
            };
            b.key = key;
            Doc {
                path: "a.rs".into(),
                bag: b,
            }
        })
        .collect()
}

type Got = Vec<(usize, i64, i64, [u32; 6], bool)>;

fn got_hits(arm: &super::Arm) -> Got {
    arm.hits
        .iter()
        .map(|h| (h.doc, h.score, h.score_fp, h.hits, h.shape_equal))
        .collect()
}

fn want_hits(hits: &[oracle::Hit]) -> Got {
    hits.iter()
        .map(|h| (h.doc, h.score, h.score_fp, h.hits, h.shape_equal))
        .collect()
}

#[test]
fn ten_thousand_seeded_queries_rank_as_the_frozen_rust_did() {
    let mut link = core_link();
    let mut rng = Lcg(0x5EED_0007);
    // [score ties among the kept, an expansion, a widened hit, a role,
    // exactly k kept]
    let mut met = [0u32; 5];
    for n in 0..10_000 {
        let docs = corpus(&mut rng, n);
        let size = docs.len();
        let own = rng.next(2) == 0;
        let seat = rng.next(size);
        let qbag = if own {
            docs[seat].bag.clone()
        } else {
            bag(&mut rng, String::new(), 30, false)
        };
        let exclude = own.then_some(seat);
        let k = 1 + rng.next(size + 2);
        let (fq, ft) = (oracle::Corpus::build(&docs), oracle::Table::build(&docs));
        let mut want_q = oracle::query_of(&qbag);
        let want_bare = oracle::top_k(&fq, &want_q, k, exclude).expect("oracle");
        let copy: Vec<Doc> = docs
            .iter()
            .map(|d| Doc {
                path: d.path.clone(),
                bag: d.bag.clone(),
            })
            .collect();
        let c = Corpus::build(copy).expect("seats ascending");
        let table = Table::build(&c);
        let query = query_of(&qbag);
        let ask = Ask {
            query: &query,
            k,
            exclude,
        };
        let cooc = cooc_rows(&table, &query).expect("cooc rows");
        let ranked = bare(&mut link, &c, &ask, Some(&cooc));
        let arm = checked(&mut link, ranked, (&want_bare, &want_q), n, "bare");
        let spelled = want_q.len();
        oracle::expand(&ft, &mut want_q).expect("oracle expansion");
        let want_added: Vec<u64> = want_q[spelled..].iter().map(|q| q.term).collect();
        assert_eq!(arm.added, want_added, "case {n}: expansion");
        let seen: Vec<usize> = arm.hits.iter().map(|h| h.doc).collect();
        let want_wide: Vec<oracle::Hit> = oracle::top_k(&fq, &want_q, k, exclude)
            .expect("oracle")
            .into_iter()
            .filter(|h| !seen.contains(&h.doc))
            .collect();
        let ranked = widened(&mut link, &c, &ask, &cooc, &arm.added, &seen);
        checked(&mut link, ranked, (&want_wide, &want_q), n, "widened");
        met[0] += u32::from(want_bare.windows(2).any(|w| w[0].score_fp == w[1].score_fp));
        met[1] += u32::from(!want_added.is_empty());
        met[2] += u32::from(!want_wide.is_empty());
        met[3] += u32::from(want_bare.iter().any(|h| h.role));
        met[4] += u32::from(want_bare.len() == k);
    }
    assert!(met.iter().all(|&c| c > 0), "an edge never met: {met:?}");
}

/// One ranked arm against the oracle's hits and query: the kept hits,
/// the weighted bag similar/1 is asked about, and the role bits.
fn checked(
    link: &mut crate::corelink::Link,
    ranked: super::Ranked,
    (want, want_q): (&[oracle::Hit], &[oracle::QueryTerm]),
    n: usize,
    what: &str,
) -> super::Arm {
    let arm = ranked.expect("source").expect("ranked");
    assert_eq!(got_hits(&arm), want_hits(want), "case {n}: {what}");
    assert_eq!(arm.bag, oracle::query_terms(want_q), "case {n}: {what} bag");
    roles(link, &arm, want, n);
    arm
}

/// similar/1's role bits over the arm against the frozen conjunction.
fn roles(link: &mut crate::corelink::Link, arm: &super::Arm, want: &[oracle::Hit], n: usize) {
    if arm.hits.is_empty() {
        return;
    }
    let judged = crate::similar::wire::judge(link, &arm.bag, &arm.hits).expect("judged");
    let frozen: Vec<bool> = want.iter().map(|h| h.role).collect();
    assert_eq!(judged.roles, frozen, "case {n}: roles");
}
