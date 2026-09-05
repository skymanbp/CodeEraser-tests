use super::*;
use crate::dedup::Params;
use crate::scan::lang::Lang;
use crate::similar::bm25::{Corpus, query_of, top_k};
use crate::similar::file_bags;
use crate::similar::ppmi::{Table, expand};
use crate::similar::terms::{feature_term, word_term};
use std::path::PathBuf;

const A: &str = "/// Fetch the user row by id.\nfn fetch_user(id: u64) -> User {\n    query(id)\n}\n\n/// Load the user row by id.\nfn load_user(id: u64) -> User {\n    query(id)\n}\n";
const B: &str = "fn render_page(p: &Page) {\n    draw(p);\n    paint(p);\n}\n\nfn user_name(u: &User) -> String {\n    u.name.clone()\n}\n";
/// Eight units sharing nothing with the four above, so no shared
/// term of theirs sits in more than half the corpus (idf 0).
const C: &str = "fn z0() {}\nfn z1() {}\nfn z2() {}\nfn z3() {}\nfn z4() {}\nfn z5() {}\nfn z6() {}\nfn z7() {}\n";
const FILES: [(&str, &str); 3] = [("a.rs", A), ("b.rs", B), ("c.rs", C)];

fn indexed(tag: &str) -> (PathBuf, Index) {
    let dir = crate::testutil::scratch(tag);
    let p = Params::default();
    let mut idx = Index::open(&dir.join("index.db"), p).expect("open");
    for (rel, text) in FILES {
        idx.refresh_file(rel, text.as_bytes(), Lang::Rust, p, false)
            .expect("refresh");
    }
    (dir, idx)
}

fn seat_named(reader: &Reader<'_>, name: &str) -> usize {
    reader
        .seats()
        .iter()
        .position(|s| s.key.starts_with(name))
        .unwrap_or_else(|| panic!("{name} indexed"))
}

/// Both arms of every query rank the same off the tables as off the
/// in-memory corpus built from the same stored bags — one ranking
/// road — and the fixture says what it was built to say.
#[test]
fn the_persisted_road_ranks_like_the_in_memory_one() {
    let (dir, idx) = indexed("similar-reader-rank");
    let reader = Reader::open(&idx).expect("reader");
    let corpus = Corpus::build(reader.docs().expect("stored bags"));
    let table = Table::build(&corpus);
    assert_eq!(reader.n_docs(), 12);
    assert_eq!(reader.avg_len(), corpus.avg_len());
    assert_eq!(
        reader.df(word_term(Channel::Callee, "query")).expect("df"),
        2
    );
    for seat in 0..reader.n_docs() {
        let bare = query_of(&reader.bag(seat).expect("bag"));
        let (mut wide, mut wide_mem) = (bare.clone(), bare.clone());
        expand(&reader, &mut wide).expect("cooc rows");
        expand(&table, &mut wide_mem).expect("in-memory");
        assert_eq!(wide, wide_mem, "seat {seat}: widened query");
        for q in [&bare, &wide] {
            assert_eq!(
                top_k(&reader, q, 5, Some(seat)).expect("tables"),
                top_k(&corpus, q, 5, Some(seat)).expect("in-memory"),
                "seat {seat}"
            );
        }
    }
    let fetch = seat_named(&reader, "fetch_user");
    let hits = top_k(
        &reader,
        &query_of(&reader.bag(fetch).expect("bag")),
        3,
        Some(fetch),
    )
    .expect("tables");
    assert!(reader.seats()[hits[0].doc].key.starts_with("load_user"));
    assert!(hits[0].role, "user + query + shape p:1 ⇒ same role");
    drop(idx);
    std::fs::remove_dir_all(&dir).ok();
}

/// The stored bag of every unit equals the bag the term road builds
/// from the same text — identity, span and term for term — and the
/// pinned words hold on that ONE bag: a change to splitting, stemming,
/// stop words or channel tags moves the index side and the query side
/// together, because there is no second road for either to keep.
#[test]
fn index_and_query_take_one_term_road() {
    let (dir, idx) = indexed("similar-reader-road");
    let reader = Reader::open(&idx).expect("reader");
    for (path, text) in FILES {
        for fresh in file_bags(text, Lang::Rust) {
            let seat = reader
                .seat_of(path, &fresh.key, fresh.nth)
                .unwrap_or_else(|| panic!("{path} {} indexed", fresh.key));
            let stored = reader.bag(seat).expect("bag");
            assert_eq!(stored.terms, fresh.terms, "{path} {}", fresh.key);
            assert_eq!(
                (stored.start_line, stored.end_line),
                (fresh.start_line, fresh.end_line),
                "{path} {}: span from symbols",
                fresh.key
            );
        }
    }
    let bag = reader.bag(seat_named(&reader, "fetch_user")).expect("bag");
    for (ch, w) in [
        (Channel::Name, "fetch"),
        (Channel::Name, "user"),
        (Channel::Callee, "query"),
        (Channel::Doc, "row"),
    ] {
        assert!(bag.terms.contains_key(&word_term(ch, w)), "{ch:?} {w}");
    }
    assert!(
        bag.terms
            .contains_key(&feature_term(Channel::Shape, b"p:1"))
    );
    assert!(
        !bag.terms.contains_key(&word_term(Channel::Doc, "the")),
        "stop word never enters"
    );
    drop(idx);
    std::fs::remove_dir_all(&dir).ok();
}

/// A bag row can only seat on a unitsig row (foreign key), and a bag
/// row seated on a FOREIGN file's unit — outside the own universe the
/// reader ranks — is a corrupt cache named at open, not a candidate
/// with no lines.
#[test]
fn a_bag_unit_outside_the_universe_is_refused() {
    let (dir, mut idx) = indexed("similar-reader-ghost");
    let ghost = idx
        .raw()
        .execute(
            "INSERT INTO bag (unit, term_hash, tf, channel) VALUES (-1, 7, 1, 0)",
            [],
        )
        .expect_err("no unitsig row -1")
        .to_string();
    assert!(ghost.contains("FOREIGN KEY"), "{ghost}");
    idx.refresh_file("d.rs", B.as_bytes(), Lang::Rust, Params::default(), true)
        .expect("foreign file");
    idx.raw()
        .execute(
            "INSERT INTO bag (unit, term_hash, tf, channel)
             SELECT u.id, 7, 1, 0 FROM unitsig u JOIN files f ON f.id = u.file_id
             WHERE f.path = 'd.rs' LIMIT 1",
            [],
        )
        .expect("a row on a foreign unit");
    let err = Reader::open(&idx).err().expect("refused").to_string();
    assert!(err.contains("outside the own universe"), "{err}");
    drop(idx);
    std::fs::remove_dir_all(&dir).ok();
}
