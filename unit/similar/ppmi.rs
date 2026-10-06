use super::*;
use crate::similar::frozen::word_term;
use crate::similar::terms::Channel;
use crate::testutil::{fetch_load_docs, word_doc as doc};

/// `fetch` and `load` travel together across four units, `fetch` meets
/// `user` once and `render` never: the table counts each word's units
/// and each pair once per unit, in both directions (the PPMI over these
/// counts is the core's — RankProps works the same corpus by hand).
#[test]
fn the_table_counts_units_and_pairs_once_per_unit() {
    let mut docs = fetch_load_docs();
    docs.push(doc("e.rs", &["render", "draw"]));
    let corpus = Corpus::build(docs).expect("ascending");
    let table = Table::build(&corpus);
    let w = |s| word_term(Channel::Name, s);
    assert_eq!(table.n_units(), 5);
    assert_eq!(table.n_term(w("fetch")).expect("n"), 4);
    let mut want = vec![(w("load"), 4), (w("user"), 1)];
    want.sort_unstable();
    assert_eq!(table.pairs(w("fetch")).expect("pairs"), want);
    assert_eq!(table.pairs(w("user")).expect("pairs").len(), 2);
    assert!(
        table
            .pairs(w("render"))
            .expect("pairs")
            .iter()
            .all(|(b, _)| *b != w("fetch"))
    );
    assert_eq!(table.capped_units, 0);
}

/// A unit past TERM_CAP word terms counts only its first TERM_CAP in
/// term order and is ledgered.
#[test]
fn a_unit_past_the_cap_counts_its_first_words_and_is_ledgered() {
    let words: Vec<String> = (0..TERM_CAP + 4).map(|i| format!("w{i}")).collect();
    let refs: Vec<&str> = words.iter().map(String::as_str).collect();
    let corpus = Corpus::build(vec![doc("a.rs", &refs)]).expect("ascending");
    let (kept, capped) = capped_words(&corpus.docs[0].bag);
    assert!(capped && kept.len() == TERM_CAP);
    assert_eq!(Table::build(&corpus).capped_units, 1);
}
