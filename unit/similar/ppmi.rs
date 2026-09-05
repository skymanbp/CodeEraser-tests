use super::*;
use crate::similar::bag::UnitBag;
use crate::similar::bm25::{Corpus, Doc, W_UNIT, query_of};
use crate::similar::terms::{Channel, word_term};
use std::collections::BTreeMap;

fn doc(words: &[&str]) -> Doc {
    let mut terms = BTreeMap::new();
    for w in words {
        terms.insert(word_term(Channel::Name, w), (Channel::Name, 1));
    }
    Doc {
        path: "x.rs".into(),
        bag: UnitBag {
            key: words.join("_"),
            nth: 0,
            start_line: 1,
            end_line: 1,
            terms,
        },
    }
}

/// `fetch` and `load` always travel together across four units while
/// `render` never meets either — so `fetch` widens to `load` and to
/// nothing else, at a fraction of its own weight, and the expansion
/// is unspelled (adds score, never evidence).
#[test]
fn a_query_widens_to_its_co_occurring_terms_only() {
    let mut docs = vec![
        doc(&["fetch", "load", "user"]),
        doc(&["fetch", "load", "post"]),
        doc(&["fetch", "load", "item"]),
        doc(&["fetch", "load"]),
        doc(&["render", "draw"]),
        doc(&["render", "paint"]),
        doc(&["user", "name"]),
        doc(&["post", "body"]),
    ];
    // eight more units without either word: N = 16, so
    // PPMI(fetch, load) = log2(4·16 / (4·4)) = 2 bits, exactly the floor
    docs.extend((0..8).map(|i| doc(&[&format!("w{i}")])));
    let corpus = Corpus::build(docs);
    let table = Table::build(&corpus);
    let (fetch, load, render) = (
        word_term(Channel::Name, "fetch"),
        word_term(Channel::Name, "load"),
        word_term(Channel::Name, "render"),
    );
    assert_eq!(table.ppmi(fetch, render), 0, "never co-occur");
    let n = neighbours(&table, fetch).expect("in-memory");
    assert_eq!(
        n.len(),
        1,
        "user / post / item co-occur once each: below MIN_COOC"
    );
    assert_eq!(n[0].0, load);
    assert_eq!(n[0].1, 2 << IDF_FRAC_BITS, "4·16 / (4·4) = 4 → two bits");

    let mut q = query_of(&doc(&["fetch"]).bag);
    let before = q.len();
    expand(&table, &mut q).expect("in-memory");
    let added: Vec<&QueryTerm> = q.iter().filter(|t| !t.spelled).collect();
    assert_eq!(added.len(), 1, "load, at the floor, is appended");
    assert_eq!(q.len(), before + added.len());
    assert!(table.capped_units == 0);
}

#[test]
fn expansion_weight_is_a_capped_fraction_of_the_parent() {
    let corpus = Corpus::build(vec![
        doc(&["a", "b"]),
        doc(&["a", "b"]),
        doc(&["c"]),
        doc(&["d"]),
        doc(&["e"]),
        doc(&["f"]),
        doc(&["g"]),
        doc(&["h"]),
    ]);
    let table = Table::build(&corpus);
    let (a, b) = (word_term(Channel::Name, "a"), word_term(Channel::Name, "b"));
    assert_eq!(
        table.ppmi(a, b),
        2 << IDF_FRAC_BITS,
        "2·8 / (2·2) = 4 → two bits"
    );
    let mut q = query_of(&doc(&["a"]).bag);
    expand(&table, &mut q).expect("in-memory");
    let added = q.iter().find(|t| !t.spelled).expect("b appended");
    assert_eq!(added.term, b);
    assert_eq!(added.weight, 3 * W_UNIT * (2 << IDF_FRAC_BITS) / PPMI_SCALE);
    assert!(
        added.weight < q[0].weight / 2 + 1,
        "at most half the parent"
    );
}
