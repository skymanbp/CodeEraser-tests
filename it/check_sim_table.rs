//! The check face's sim table over a tree where one file pair is
//! BOTH a verified clone pair and a verified docdup pair. Until 1.7.4
//! the pair arrived twice under a pair-wide identity and the core
//! refused the whole request (`contract: sim 1: not strictly
//! ascending`); from 1.7.4 to 9.0.0 a merge kept the clone row alone,
//! so the docdup axis never saw the pair (and read 0 in a tree with no
//! Markdown). Since 9.1.0 (v2.35 V2) the identity is (u, v, kind): the
//! pair rides once per family, axis 3 charges it over the files that
//! own a live docdup segment, and the join still reads one candidate
//! per pair. It arises wherever two files share code AND prose, the
//! ordinary shape of sibling modules.
//!
//! The fixture's own validity is pinned in the same test: if either
//! family stopped judging this pair the regression would pass on a
//! tree that no longer has the defect's shape.

use crate::common;
use crate::common::core_bin;
use std::path::{Path, PathBuf};

/// 75 words of prose, verbatim in both files: past the admission
/// floor AND past the verbatim hard hit (both 50 words), carrying no
/// license marker — a first block that had one would be exempted as
/// a header and judged by nobody.
const SHARED_DOC: &str = "// The ledger this module keeps is append only: every row that arrives is
// written once, in arrival order, and nothing already written is ever
// rewritten in place. A reader therefore sees a prefix of the truth and
// never a torn row, which is what lets the summary pass run without a
// lock. Rotation moves the whole file aside and starts a new one, so the
// oldest rows leave together rather than one at a time.
";

/// Two sibling modules: the shared doc block above each of the T2
/// seed's two renamings. Comments never enter the token stream, so
/// the clone pair is the seed's own calibrated one and the doc pair
/// is the block, on the same two files.
fn fixture(tag: &str) -> PathBuf {
    let dir = common::tmp(tag);
    for (name, seed) in [("a.rs", 1), ("b.rs", 2)] {
        let src = format!("{SHARED_DOC}{}", common::rust_fn(seed));
        std::fs::write(dir.join(name), src).expect(name);
    }
    dir
}

/// The file pairs the T3 family relates in `dir`, `a <-> b`.
fn t3_pairs(dir: &Path) -> Vec<String> {
    let t3 = codeeraser::dedup::t3::run(dir, None, &core_bin()).expect("clone");
    t3.file_pairs()
        .map(|(a, b)| format!("{a} <-> {b}"))
        .collect()
}

#[test]
fn a_pair_both_families_judged_rides_once_per_family() {
    let dir = fixture("check-sim-table");
    // (1) the fixture IS the defect's shape — both families, one pair
    let found = common::analyze(&dir, 1, 1);
    let block = &found.blocks[0];
    assert_eq!(
        (block.a_file.as_str(), block.b_file.as_str()),
        ("a.rs", "b.rs"),
        "the clone family judges the pair"
    );
    let doc = codeeraser::faces::doc(codeeraser::faces::docdup(
        &dir,
        &core_bin(),
        Default::default(),
    ))
    .expect("docdup");
    let [hit] = doc["dups"].as_array().expect("dups").as_slice() else {
        panic!("one doc pair, got {}", doc["dups"]);
    };
    let (a, b) = (hit["a"].as_str().expect("a"), hit["b"].as_str().expect("b"));
    assert!(
        a.starts_with("a.rs:") && b.starts_with("b.rs:"),
        "the docdup family judges the SAME pair: {a} <-> {b}"
    );
    assert!(
        hit["verbatim"].as_u64() >= Some(50),
        "the shared block is a hard hit, not a Jaccard accident: {}",
        hit["verbatim"]
    );

    // the T3 family judges the same two renamings a near-miss pair too
    assert_eq!(
        t3_pairs(&dir),
        ["a.rs <-> b.rs"],
        "the T3 family judges the pair"
    );

    // (2) judged: one row per family (T1/T2, T3, docdup), one candidate
    // per pair, and both the clone and the docdup axis name the pair's
    // lines (the extent law, 10.0.0) though the tree holds no Markdown
    // file: each file is the 6-line doc block over the 11-line seed
    // function, L = 2 · 17; the clone axis names both functions (2 · 11
    // lines), the docdup axis both doc blocks (2 · 6)
    let o = common::judged(&dir);
    assert_eq!(o.files, 2, "both files are in the verdict universe");
    assert_eq!(o.sim_pairs, 3, "the pair rides once per family");
    assert_eq!(
        o.reply.candidates.len(),
        1,
        "one join candidate per pair: {:?}",
        o.reply.candidates
    );
    let axis = |c: i64| o.reply.axes.iter().find(|a| a[0] == c).map(|a| a[1]);
    let (doc, func) = (
        SHARED_DOC.lines().count(),
        common::rust_fn(1).lines().count(),
    );
    let (doc, func) = (doc as i64, func as i64);
    let len = 2 * (doc + func);
    assert_eq!(
        (axis(2), axis(3)),
        (Some(1000 * 2 * func / len), Some(1000 * 2 * doc / len)),
        "{:?}",
        o.reply.axes
    );
}

/// Two files the T3 family alone relates (plan v2.30 step 5b-9): no
/// T1/T2 block, one near-miss pair — and the check seats it as one
/// sim row of kind 1 and its two units' spans as clone lines (the
/// extent law, 10.0.0), the docdup axis naming nothing. Until 5b-9 the
/// table took the free T1/T2 blocks and kind 1 was dead on every live
/// road.
#[test]
fn a_t3_only_pair_rides_the_sim_table_as_kind_one() {
    let dir = common::tmp("check-sim-t3");
    for (name, seed) in [("c.rs", 1), ("d.rs", 2)] {
        std::fs::write(dir.join(name), common::rust_near_miss(seed)).expect(name);
    }
    // (1) the fixture IS the shape: no block, one T3 clone, c <-> d
    common::analyze(&dir, 0, 0);
    assert_eq!(t3_pairs(&dir), ["c.rs <-> d.rs"], "one near-miss pair");
    // (2) one sim row, one candidate, the clone axis naming the pair's
    // unit lines — each file is its one function, so every line of both
    // (1000 per 1000) — the docdup axis not at all
    let t3_only = common::judged(&dir);
    let axis = |o: &codeeraser::score::Outcome, c: i64| {
        o.reply.axes.iter().find(|a| a[0] == c).map(|a| a[1])
    };
    assert_eq!(
        (
            t3_only.files,
            t3_only.sim_pairs,
            t3_only.reply.candidates.len()
        ),
        (2, 1, 1)
    );
    assert_eq!(
        (axis(&t3_only, 2), axis(&t3_only, 3)),
        (Some(1000), Some(0)),
        "kind 1 names clone lines: {:?}",
        t3_only.reply.axes
    );
}
