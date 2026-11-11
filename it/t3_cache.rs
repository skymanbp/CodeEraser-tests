//! The T3 verdict cache end to end (plan v2.30 step 5b-9). A second
//! `ce clone` over an unchanged tree replays every sendable pair and
//! asks the core no pair, reporting the same clones; a poisoned row
//! is refused by the core's decision (clone/1 `decide`, plan v2.33
//! W3) — a replayed bit is judged, never trusted; a new tree re-sends its own pairs and no other, and the
//! same tree in four files is one slot.

use crate::common::{self, core_bin};
use codeeraser::dedup::t3;
use std::path::{Path, PathBuf};

/// One tree thrice under a fresh index, judged cold: three T3 pairs at
/// ted 0, every sendable pair on the wire, nothing replayed.
fn cold(tag: &str) -> (PathBuf, String, t3::Judged) {
    let dir = common::tmp(tag);
    common::seed_clone_trio(&dir);
    let core = core_bin();
    let first = t3::run(&dir, None, &core).expect("clone");
    let c = &first.counts;
    assert!(
        !first.clones.is_empty() && c.judged >= 1,
        "judged pairs: {} / {}",
        first.clones.len(),
        c.judged
    );
    assert_eq!(
        (c.cached, c.requests),
        (0, 1),
        "cold: every pair rode the wire"
    );
    (dir, core, first)
}

fn names(r: &t3::Judged) -> Vec<(String, String)> {
    let name = |i: usize| {
        let u = &r.units[i];
        format!("{}:{}#{}", u.path, u.key, u.nth)
    };
    r.clones
        .iter()
        .map(|&(a, b, _)| (name(a), name(b)))
        .collect()
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
        refused.contains("disagrees with the core's decision"),
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

/// One function body in two statement orders: the same nodes, so the
/// size and label bounds both pass, and a postorder label sequence that
/// differs by whole loops, far past the clone ceiling.
fn reordered(loops_first: [&str; 2]) -> String {
    let head = "fn tally(input: &[i64], limit: i64) -> i64 {\n    let mut total = 0;\n    let mut count = 0;\n";
    let tail = "    match total % 4 {\n        0 => total + 1,\n        1 => total - 1,\n        _ => total,\n    }\n}\n";
    format!("{head}{}{}{tail}", loops_first[0], loops_first[1])
}

/// Plan v2.35 C1 and C2: a pair the core's label-sequence bound decides
/// has no score and is counted prefiltered, and the verdict cache keeps
/// it as a no-distance row — the warm run over the same index sends no
/// pair, and both reports agree on everything but the counters of where
/// a pair's answer came from (judged / prefiltered / cached, and the
/// request count that follows from them).
#[test]
fn a_pair_the_sequence_bound_decides_is_remembered_and_never_resent() {
    let for_loop = "    for value in input {\n        if *value > limit {\n            total += value * 3 + 7;\n        } else {\n            total -= value / 3;\n        }\n    }\n";
    let while_loop = "    while count < limit {\n        count += 1;\n        total = total * 2 - count;\n        if total > 1000 {\n            break;\n        }\n    }\n";
    let dir = common::tmp("t3-cache-sequence");
    std::fs::write(dir.join("a.rs"), reordered([for_loop, while_loop])).expect("a.rs");
    std::fs::write(dir.join("b.rs"), reordered([while_loop, for_loop])).expect("b.rs");
    let core = core_bin();
    let runs = [0, 1].map(|_| t3::run(&dir, None, &core).expect("clone"));
    let c = runs.each_ref().map(|r| &r.counts);
    assert!(
        c[0].prefiltered >= 1 && c[0].cached == 0,
        "cold: the core decided a pair without a distance: {} prefiltered",
        c[0].prefiltered
    );
    assert_eq!(
        (c[1].cached, c[1].judged, c[1].prefiltered, c[1].requests),
        (c[0].sent, 0, 0, 0),
        "warm: every sendable pair replayed, nothing asked"
    );
    for k in c {
        assert_eq!(
            k.judged + k.prefiltered + k.cached,
            k.sent,
            "sent = judged + prefiltered + cached"
        );
    }
    let rest = |k: &t3::Counts| {
        let sizes = [k.units, k.over_cap_units, k.forest_units].map(|n| n as u64);
        let pass = [
            k.survivors,
            k.s5_windowed,
            k.s5_pruned_label,
            k.s5_already,
            k.s5_new,
        ];
        let dropped = [k.pairs_dropped_over_cap, k.pairs_dropped_forest, k.sent];
        (sizes, pass, dropped)
    };
    assert_eq!(rest(c[0]), rest(c[1]), "every other counter agrees");
    assert_eq!(names(&runs[0]), names(&runs[1]), "the same clones");
}
