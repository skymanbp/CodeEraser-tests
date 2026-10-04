use super::*;
use crate::similar::corpus::{Corpus, Doc, query_of};
use crate::similar::ppmi::Table;
use crate::similar::terms::word_term;
use crate::testutil::{fetch_load_docs, word_doc as doc};

fn corpus(docs: Vec<Doc>) -> Corpus {
    Corpus::build(docs).expect("seats ascending")
}

/// `user` sits in three of four seats (n ≤ 2·df: no score), `fetch` in
/// one: only fetch's posting rides, only its seat's length, and the
/// asked terms carry their df either way.
#[test]
fn the_request_carries_postings_for_scoring_terms_only() {
    let c = corpus(vec![
        doc("a.rs", &["fetch", "user"]),
        doc("b.rs", &["user"]),
        doc("c.rs", &["user"]),
        doc("d.rs", &["load"]),
    ]);
    let q = c.query_of(0);
    let ask = Ask {
        query: &q,
        k: 3,
        exclude: Some(0),
    };
    let body = request(&c, &ask, &[], None, false, &[2, 1]).expect("in-memory");
    let (fetch, user) = (
        word_term(Channel::Name, "fetch"),
        word_term(Channel::Name, "user"),
    );
    let mut terms = vec![json!([fetch, 1]), json!([user, 3])];
    terms.sort_by_key(|t| t[0].as_u64());
    assert_eq!(body["terms"], json!(terms));
    assert_eq!(body["postings"], json!([[fetch, 0, 1]]));
    assert_eq!(body["lens"], json!([[0, 2]]));
    assert_eq!(body["seen"], json!([1, 2]), "seen rides ascending");
    assert_eq!(body["exclude"], json!(0));
    assert_eq!(
        (&body["n"], &body["avg"], &body["k"]),
        (&json!(4), &json!(1), &json!(3))
    );
    assert!(body.get("words").is_none(), "no cooc rows unless asked");
    let none = Ask {
        exclude: None,
        ..ask
    };
    let free = request(&c, &none, &[], None, false, &[]).expect("in-memory");
    assert!(free.get("exclude").is_none());
}

/// fetch and load share four of sixteen seats; fetch meets user once:
/// the words row carries fetch's count, the pair floor keeps load and
/// drops user, and a word in more than a quarter of the seats asks for
/// no pairs at all.
#[test]
fn cooc_rows_fetch_by_the_package_floor_and_ratio() {
    let mut docs = fetch_load_docs();
    docs.extend((0..12).map(|i| doc(&format!("z{i:02}.rs"), &["wide", &format!("w{i}")])));
    let c = corpus(docs);
    let t = Table::build(&c);
    let (fetch, load, wide) = (
        word_term(Channel::Name, "fetch"),
        word_term(Channel::Name, "load"),
        word_term(Channel::Name, "wide"),
    );
    let rows = cooc_rows(&t, &query_of(&doc("q.rs", &["fetch"]).bag)).expect("in-memory");
    assert_eq!(rows.words, [[fetch, 4]]);
    assert_eq!(rows.pairs, [[fetch, load, 4, 4]]);
    let rows = cooc_rows(&t, &query_of(&doc("q.rs", &["wide"]).bag)).expect("in-memory");
    assert_eq!((rows.words, rows.pairs.len()), (vec![[wide, 12]], 0));
}

/// A hit row naming `seat` and nothing else, as many as `seats` lists.
fn seats(seats: &[u64]) -> Value {
    seats
        .iter()
        .map(|s| json!([s, 0, 0, 0, 0, 0, 0, 0, 0]))
        .collect()
}

fn reply(hits: Value) -> Value {
    json!({"degraded": false, "hits": hits, "scoreDen": 65536, "added": [[5, 0]], "query": [[1, 768]]})
}

#[test]
fn a_well_formed_reply_is_relayed_and_every_skew_is_named() {
    let arm = consume(&reply(json!([[2, 1, 70000, 1, 0, 1, 0, 0, 0]])), 4, 5).expect("relayed");
    assert_eq!(arm.added, [5]);
    assert_eq!(arm.bag, [[1, 768]]);
    assert_eq!(
        arm.hits,
        [Hit {
            doc: 2,
            score: 1,
            score_fp: 70000,
            den: 65536,
            hits: [1, 0, 1, 0, 0, 0],
            shape_equal: false,
        }]
    );
    let degraded = json!({"degraded": true, "reason": "rank_too_large"});
    assert_eq!(consume(&degraded, 1, 1), Err("rank_too_large".into()));
    for (hits, n, k, want) in [
        (seats(&[9]), 4, 5, "out of range"),
        (seats(&[1, 1]), 4, 5, "repeated"),
        (json!([[1, 0, 0]]), 4, 5, "a hit is"),
        (seats(&[1, 2]), 4, 1, "more hits"),
    ] {
        let err = consume(&reply(hits), n, k).expect_err(want);
        assert!(err.contains(want), "{want}: {err}");
    }
}
