//! The check face's sim table over a tree where one file pair is
//! BOTH a verified clone pair and a verified docdup pair. `measure`
//! concatenates the two families' pair sets, each ascending on its
//! own, and the verdict wire's identity for that table is the pair
//! alone — so such a pair arrived twice and the core refused the
//! whole request by name (`contract: sim 1: not strictly ascending`),
//! which is `ce check` refusing to judge the tree at all. The case
//! could not arise before 7.0.0 (only the clone family sent rows);
//! it arises wherever two files share code AND prose, the ordinary
//! shape of sibling modules — ten such pairs in one 55-file
//! directory of the first repository that hit it.
//!
//! The fixture's own validity is pinned in the same test: if either
//! family stopped judging this pair the regression would pass on a
//! tree that no longer has the defect's shape.

use crate::common;
use crate::common::core_bin;
use std::path::PathBuf;

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

#[test]
fn a_pair_both_families_judged_rides_the_sim_table_once() {
    let dir = fixture("check-sim-table");
    // (1) the fixture IS the defect's shape — both families, one pair
    let found = common::analyze(&dir, 1, 1);
    let block = &found.blocks[0];
    assert_eq!(
        (block.a_file.as_str(), block.b_file.as_str()),
        ("a.rs", "b.rs"),
        "the clone family judges the pair"
    );
    let doc = codeeraser::faces::docdup(&dir, &core_bin()).expect("docdup");
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

    // (2) the request the core refused: judged, one row, one candidate
    let o = common::judged(&dir);
    assert_eq!(o.files, 2, "both files are in the verdict universe");
    assert_eq!(o.sim_pairs, 1, "the pair rides once, not once per family");
    assert_eq!(
        o.reply.candidates.len(),
        1,
        "one join candidate per sim row: {:?}",
        o.reply.candidates
    );
}

/// Two files the T3 family alone relates (plan v2.30 step 5b-9): no
/// T1/T2 block, one near-miss pair — and the check seats it as one
/// sim row of kind 1, charging the clone axis exactly as the block
/// pair above charges it (two touched files over two code files) and
/// the docdup axis nothing. Until 5b-9 the table took the free T1/T2
/// blocks and kind 1 was dead on every live road.
#[test]
fn a_t3_only_pair_rides_the_sim_table_as_kind_one() {
    let dir = common::tmp("check-sim-t3");
    for (name, seed) in [("c.rs", 1), ("d.rs", 2)] {
        std::fs::write(dir.join(name), common::rust_near_miss(seed)).expect(name);
    }
    // (1) the fixture IS the shape: no block, one T3 clone, c <-> d
    common::analyze(&dir, 0, 0);
    let t3 = codeeraser::dedup::t3::run(&dir, None, &core_bin()).expect("clone");
    let pair: Vec<(&str, &str)> = t3.file_pairs().collect();
    assert!(
        matches!(pair[..], [("c.rs", "d.rs")]),
        "one near-miss pair, c <-> d: {pair:?}"
    );
    // (2) one sim row, one candidate, the clone axis charged as the
    // block pair's is, the docdup axis not at all
    let (t3_only, block_pair) = (
        common::judged(&dir),
        common::judged(&fixture("check-sim-twin")),
    );
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
        (axis(&block_pair, 2), Some(0)),
        "kind 1 reads as kind 0 on the clone axis: {:?}",
        t3_only.reply.axes
    );
    assert!(axis(&t3_only, 2) > Some(0), "{:?}", t3_only.reply.axes);
}
