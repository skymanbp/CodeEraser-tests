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
use codeeraser::score::{self, Opts};
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
fn fixture() -> PathBuf {
    let dir = common::tmp("check-sim-table");
    for (name, seed) in [("a.rs", 1), ("b.rs", 2)] {
        let src = format!("{SHARED_DOC}{}", common::rust_fn(seed));
        std::fs::write(dir.join(name), src).expect(name);
    }
    dir
}

#[test]
fn a_pair_both_families_judged_rides_the_sim_table_once() {
    let dir = fixture();
    // (1) the fixture IS the defect's shape — both families, one pair
    let found = common::analyze(&dir, 1, 1);
    let block = &found.blocks[0];
    assert_eq!(
        (block.a_file.as_str(), block.b_file.as_str()),
        ("a.rs", "b.rs"),
        "the clone family judges the pair"
    );
    let doc = codeeraser::docdup::judge::run(&dir, None, &core_bin()).expect("docdup");
    let [hit] = doc.hits.as_slice() else {
        panic!("one doc pair, got {}", doc.hits.len());
    };
    assert!(
        hit.a.starts_with("a.rs:") && hit.b.starts_with("b.rs:"),
        "the docdup family judges the SAME pair: {} <-> {}",
        hit.a,
        hit.b
    );
    assert!(
        hit.m.verbatim >= 50,
        "the shared block is a hard hit, not a Jaccard accident: {}",
        hit.m.verbatim
    );

    // (2) the request the core refused: judged, one row, one candidate
    let opts = Opts {
        db: None,
        core: core_bin(),
        days: None,
        floor: None,
        establish: false,
        pinned_soft: None,
        baseline: score::baseline::read(&dir).expect("baseline read"),
    };
    let o = score::run(&dir, opts).expect("the core judges the request");
    assert!(o.reply.degraded.is_none(), "healthy judgment");
    assert_eq!(o.files, 2, "both files are in the verdict universe");
    assert_eq!(o.sim_pairs, 1, "the pair rides once, not once per family");
    assert_eq!(
        o.reply.candidates.len(),
        1,
        "one join candidate per sim row: {:?}",
        o.reply.candidates
    );
}
