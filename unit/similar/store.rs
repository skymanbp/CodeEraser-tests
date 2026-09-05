use super::*;
use crate::dedup::Params;
use crate::dedup::index::Index;
use crate::scan::lang::Lang;
use crate::similar::reader::Reader;
use crate::similar::terms::word_term;
use std::collections::BTreeSet;

const FETCH: &str = "/// Fetch the user row by id.\nfn fetch_user(id: u64) -> User {\n    query(id)\n}\n\nfn render_page(p: &Page) {\n    draw(p);\n}\n";
const LOAD: &str =
    "/// Load the user row by id.\nfn load_user(id: u64) -> User {\n    query(id)\n}\n";
const ITEM: &str = "fn fetch_item(id: u64) -> Item {\n    query(id)\n}\n";

/// The aggregate recounted from the bag rows alone — what any
/// sequence of differential moves must leave behind.
fn recounted(idx: &Index) -> Delta {
    let reader = Reader::open(idx).expect("reader");
    let mut d = Delta::default();
    for doc in reader.docs().expect("stored bags") {
        d.tally(&doc.bag, 1);
    }
    d
}

/// The aggregate as the table holds it.
fn stored(idx: &Index) -> Delta {
    let mut d = Delta::default();
    let rows = crate::graph::load::rows(idx.raw(), "SELECT term_hash, df, marg FROM df", |r| {
        Ok((r.get::<_, i64>(0)? as u64, (r.get(1)?, r.get(2)?)))
    })
    .expect("df rows");
    d.df.extend(rows);
    d
}

fn agree(idx: &Index, stage: &str) {
    let (s, r) = (stored(idx), recounted(idx));
    assert_eq!(s.df, r.df, "{stage}: df and marginal");
    assert!(
        s.df.values().all(|(df, marg)| *df > 0 && *marg <= *df),
        "{stage}: a zero row survived the sweep, or a marginal outgrew its df"
    );
}

fn bag_rows(idx: &Index, path: &str) -> i64 {
    idx.raw()
        .query_row(
            "SELECT COUNT(*) FROM bag b JOIN unitsig u ON u.id = b.unit
             JOIN files f ON f.id = u.file_id WHERE f.path = ?1",
            (path,),
            |r| r.get(0),
        )
        .expect("count")
}

/// Add, add, rewrite, flip to foreign and back, remove both: after
/// every step the stored df and marginal equal a recount from the bag
/// rows, and no row sits at zero. The rewrite retires `fetch_user`
/// and `render_page` (with its `draw` callee) and adds `load_user`;
/// the foreign flip retires a file's rows without a byte changing.
#[test]
fn aggregates_follow_every_refresh_by_difference() {
    let dir = crate::testutil::scratch("similar-store");
    let p = Params::default();
    let mut idx = Index::open(&dir.join("index.db"), p).expect("open");
    let put = |idx: &mut Index, rel: &str, text: &str, foreign: bool| {
        idx.refresh_file(rel, text.as_bytes(), Lang::Rust, p, foreign)
            .expect("refresh");
    };
    put(&mut idx, "a.rs", FETCH, false);
    agree(&idx, "one file");
    put(&mut idx, "b.rs", ITEM, false);
    agree(&idx, "two files");
    let (fetch, draw) = (
        word_term(Channel::Name, "fetch"),
        word_term(Channel::Callee, "draw"),
    );
    assert_eq!(
        stored(&idx).df[&word_term(Channel::Callee, "query")],
        (2, 2)
    );
    assert_eq!(stored(&idx).df[&fetch], (2, 2), "fetch_user + fetch_item");

    put(&mut idx, "a.rs", LOAD, false);
    agree(&idx, "a.rs rewritten");
    assert_eq!(
        stored(&idx).df[&fetch],
        (1, 1),
        "only fetch_item spells fetch now"
    );
    assert!(!stored(&idx).df.contains_key(&draw), "render_page is gone");

    put(&mut idx, "b.rs", ITEM, true);
    agree(&idx, "b.rs foreign");
    assert_eq!(bag_rows(&idx, "b.rs"), 0, "a foreign file has no bag rows");
    assert!(!stored(&idx).df.contains_key(&fetch));
    put(&mut idx, "b.rs", ITEM, false);
    agree(&idx, "b.rs own again");
    assert_eq!(stored(&idx).df[&fetch], (1, 1));
    both_removed(idx);
    std::fs::remove_dir_all(&dir).ok();
}

/// The last step of the sequence: both files leave the tree, and the
/// aggregate and the bag table are empty behind them.
fn both_removed(mut idx: Index) {
    let seen: BTreeSet<String> = ["a.rs", "b.rs"].iter().map(|s| s.to_string()).collect();
    assert_eq!(
        idx.remove_missing(&BTreeSet::new(), &seen).expect("reap"),
        2
    );
    agree(&idx, "both removed");
    assert!(stored(&idx).df.is_empty(), "nothing left to count");
    let left: i64 = idx
        .raw()
        .query_row("SELECT COUNT(*) FROM bag", [], |r| r.get(0))
        .expect("count");
    assert_eq!(left, 0, "the cascade took every bag row");
}

/// A unit the edit did not touch cancels itself out of the delta —
/// the property that makes the upkeep cost the change, not the file —
/// and only words carry a marginal.
#[test]
fn an_untouched_unit_cancels_out_of_the_delta() {
    let bags = file_bags(FETCH, Lang::Rust);
    assert_eq!(bags.len(), 2);
    let mut d = Delta::default();
    for b in &bags {
        d.tally(b, -1);
    }
    assert!(
        d.df.iter().all(|(t, (df, marg))| *df < 0
            && (*marg < 0) == bags.iter().any(|b| capped_words(b).0.contains(t))),
        "every term moved down, every capped word's marginal with it"
    );
    d.tally(&bags[0], 1);
    d.tally(&bags[1], 1);
    assert!(!d.df.is_empty(), "the terms were tallied");
    assert!(d.df.values().all(|(df, marg)| (*df, *marg) == (0, 0)));
}
