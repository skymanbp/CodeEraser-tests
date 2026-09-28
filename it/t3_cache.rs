//! The T3 verdict cache end to end (plan v2.30 step 5b-9). A second
//! `ce clone` over an unchanged tree replays every sendable pair and
//! asks the core nothing, reporting the same clones; a poisoned row
//! is refused by the mirror ensure — a replayed bit is judged, never
//! trusted; a new tree re-sends its own pairs and no other, and the
//! same tree in four files is one slot.

use crate::common::{self, core_bin};
use codeeraser::dedup::t3;
use std::path::{Path, PathBuf};

/// One tree thrice under a fresh index, judged cold: three T3 pairs at
/// ted 0, every sendable pair on the wire, nothing replayed.
fn cold(tag: &str) -> (PathBuf, String, t3::Report) {
    let dir = common::tmp(tag);
    common::seed_clone_trio(&dir);
    let core = core_bin();
    let first = t3::run(&dir, None, &core).expect("clone");
    let c = &first.counts;
    assert!(
        c.clones >= 1 && c.judged >= 1,
        "judged pairs: {} / {}",
        c.clones,
        c.judged
    );
    assert_eq!(
        (c.cached, c.requests),
        (0, 1),
        "cold: every pair rode the wire"
    );
    (dir, core, first)
}

fn names(r: &t3::Report) -> Vec<(String, String)> {
    r.hits.iter().map(|h| (h.a.clone(), h.b.clone())).collect()
}

fn index(dir: &Path) -> rusqlite::Connection {
    rusqlite::Connection::open(dir.join(".ce/index.db")).expect("index")
}

fn slots(dir: &Path) -> i64 {
    index(dir)
        .query_row("SELECT count(*) FROM t3ted", [], |r| r.get(0))
        .expect("count")
}

#[test]
fn a_second_run_replays_every_verdict_and_a_poisoned_row_is_refused() {
    let (dir, core, first) = cold("t3-cache-replay");
    let second = t3::run(&dir, None, &core).expect("clone again");
    let c2 = &second.counts;
    assert_eq!(
        (c2.cached, c2.judged, c2.prefiltered, c2.requests),
        (first.counts.sent, 0, 0, 0),
        "warm: every sendable pair replayed, nothing asked"
    );
    assert_eq!(
        names(&first),
        names(&second),
        "the replayed report is the judged one"
    );
    let flipped = index(&dir)
        .execute(
            "UPDATE t3ted SET clone = 1 - clone WHERE ted IS NOT NULL",
            [],
        )
        .expect("poison");
    assert!(flipped >= 1, "a scored row to poison");
    let refused = match t3::run(&dir, None, &core) {
        Ok(_) => panic!("a poisoned bit is refused"),
        Err(e) => format!("{e:#}"),
    };
    assert!(
        refused.contains("disagrees with the pinned mirror"),
        "{refused}"
    );
}

#[test]
fn a_new_tree_re_sends_its_own_pairs_and_no_other() {
    let (dir, core, first) = cold("t3-cache-change");
    assert_eq!(slots(&dir), 1, "one tree thrice is one slot");
    // d.rs is the seed with one statement more: a new tree, whose
    // pairs with the three copies are new to the cache
    let reshaped =
        common::rust_fn(4).replace("    total_4\n}", "    total_4 += 1;\n    total_4\n}");
    assert_ne!(reshaped, common::rust_fn(4), "the reshape landed");
    std::fs::write(dir.join("d.rs"), reshaped).expect("d.rs");
    let third = t3::run(&dir, None, &core).expect("clone after the change");
    let c = &third.counts;
    assert_eq!(c.cached, first.counts.sent, "the unchanged pairs replay");
    assert!(
        c.judged >= 1,
        "the new tree's pairs were judged: {}",
        c.judged
    );
    assert_eq!(
        c.cached + c.judged + c.prefiltered,
        c.sent,
        "every sendable pair is replayed, judged or prefiltered"
    );
    assert_eq!(
        slots(&dir),
        2,
        "the old tree's slot and the new pair of trees"
    );
}
