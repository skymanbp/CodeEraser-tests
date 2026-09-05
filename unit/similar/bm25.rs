use super::*;
use crate::similar::bag::UnitBag;
use std::collections::BTreeMap;

fn bag(key: &str, terms: &[(Channel, &str, u32)]) -> UnitBag {
    let mut b = UnitBag {
        key: key.into(),
        nth: 0,
        start_line: 1,
        end_line: 1,
        terms: BTreeMap::new(),
    };
    for (ch, w, tf) in terms {
        let t = if ch.is_words() {
            crate::similar::terms::word_term(*ch, w)
        } else {
            crate::similar::terms::feature_term(*ch, w.as_bytes())
        };
        b.terms.insert(t, (*ch, *tf));
    }
    b
}

fn doc(path: &str, key: &str, terms: &[(Channel, &str, u32)]) -> Doc {
    Doc {
        path: path.into(),
        bag: bag(key, terms),
    }
}

/// The expanded fraction in `contribution` is derived from K1 and B;
/// re-derive it from the rationals for a grid of inputs.
#[test]
fn expanded_fraction_matches_the_rational_definition() {
    let (k1n, k1d) = K1;
    let (bn, bd) = B;
    for (tf, len, avg) in [(1, 10, 10), (3, 7, 20), (12, 300, 45), (1, 1, 1)] {
        // (k1+1)·tf / (tf + k1·(1 − b + b·len/avg)), cleared of every
        // denominator: numerator (k1n+k1d)·tf·bd·avg·k1d…, one fraction
        let num = (k1n + k1d) * tf * bd * avg;
        let den = tf * k1d * bd * avg + k1n * ((bd - bn) * avg + bn * len);
        let generic = ((1i128 << SCORE_FRAC_BITS) * num) / den;
        assert_eq!(
            contribution(1, 1, tf, len, avg),
            generic,
            "tf {tf} len {len} avg {avg}"
        );
    }
}

#[test]
fn integer_log2_is_exact_on_powers_and_monotone_between() {
    assert_eq!(log2_fp(1, 1), 0);
    assert_eq!(log2_fp(8, 1), 3 << IDF_FRAC_BITS);
    assert_eq!(log2_fp(6, 4), log2_fp(3, 2), "equal ratios, equal bits");
    let (a, b, c) = (log2_fp(3, 2), log2_fp(7, 4), log2_fp(2, 1));
    assert!(a < b && b < c, "1.5 < 1.75 < 2");
    // log2(1.5) = 0.58496… → floor(0.58496 · 256) = 149
    assert_eq!(a, 149);
}

#[test]
fn idf_floors_at_zero_for_common_terms() {
    assert_eq!(idf_fp(10, 6), 0, "a term in more than half the units");
    assert!(idf_fp(10, 1) > idf_fp(10, 3), "rarer is worth more");
    assert_eq!(idf_fp(0, 0), 0);
}

/// Four units over three words and one callee, plus eight units
/// sharing nothing with them: in a four-unit corpus every shared term
/// sits in more than half the units and idf floors all of them to zero.
fn twelve_units() -> Corpus {
    let user = |name: &'static str, callee: Option<&'static str>, p: &'static str| {
        let mut terms = vec![(Channel::Name, name, 1), (Channel::Name, "user", 1)];
        terms.extend(callee.map(|c| (Channel::Callee, c, 1)));
        terms.push((Channel::Shape, p, 1));
        terms
    };
    let mut docs = vec![
        doc("a.rs", "fetch_user/1", &user("fetch", Some("query"), "p:1")),
        doc("b.rs", "load_user/1", &user("load", Some("query"), "p:1")),
        doc(
            "c.rs",
            "render/2",
            &[
                (Channel::Name, "render", 1),
                (Channel::Callee, "draw", 1),
                (Channel::Shape, "p:2", 1),
            ],
        ),
        doc("d.rs", "user_name/0", &user("name", None, "p:0")),
    ];
    docs.extend((0..8).map(|i| {
        let z: &'static str = Box::leak(format!("z{i}").into_boxed_str());
        doc("z.rs", &format!("{z}/0"), &[(Channel::Name, z, 1)])
    }));
    Corpus::build(docs)
}

#[test]
fn ranking_prefers_shared_rare_terms_and_reads_role_evidence() {
    let corpus = twelve_units();
    let q = corpus.query_of(0);
    let hits = top_k(&corpus, &q, 3, Some(0)).expect("in-memory");
    assert_eq!(hits[0].doc, 1, "shares user + query + shape");
    assert_eq!(hits[0].hits, [1, 1, 1, 0, 0, 0]);
    assert!(
        hits[0].shape_equal && hits[0].role,
        "name ∧ callee ⇒ same role"
    );
    assert_eq!(hits[1].doc, 3, "shares only user");
    assert!(!hits[1].role);
    assert!(
        hits.iter().all(|h| h.doc != 0),
        "the query never ranks itself"
    );
    assert_eq!(hits.len(), 2, "c.rs shares nothing and is absent");
}

#[test]
fn the_role_rule_is_the_spec_conjunction() {
    assert!(role(&[1, 0, 1, 0, 0, 0], false));
    assert!(role(&[2, 0, 0, 0, 0, 0], true));
    assert!(!role(&[2, 0, 0, 0, 0, 0], false), "two names without shape");
    assert!(!role(&[0, 5, 5, 5, 5, 5], true), "no name, no role");
}
