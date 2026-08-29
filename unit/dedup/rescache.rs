use super::super::{Params, analyze, index::Index};
use super::*;
use crate::testutil::scratch;
use std::path::{Path, PathBuf};

fn seeded(tag: &str) -> PathBuf {
    let root = scratch(tag);
    std::fs::write(root.join("a.rs"), "fn a() { let x = 1; }\n").unwrap();
    std::fs::write(root.join("b.rs"), "fn b() { let y = 2; }\n").unwrap();
    root
}

fn open(root: &Path) -> Index {
    Index::open(&root.join(".ce/index.db"), Params::default()).unwrap()
}

fn poison() -> Blocks {
    Blocks {
        blocks: vec![crate::dedup::pairs::Block {
            a_file: "poison.rs".into(),
            a_start: 1,
            a_end: 2,
            b_file: "poison.rs".into(),
            b_start: 3,
            b_end: 4,
            tokens: 99,
            distinct: 99,
        }],
        groups: Vec::new(),
        hot_chained: 0,
        stale_skipped: 0,
        low_diversity_suppressed: 0,
        distincts: Vec::new(),
    }
}

/// The decisive counterfactual pair: a poisoned slot under the
/// CURRENT digest comes back verbatim (so the hit path really
/// serves the cache, not a recompute that happens to agree), and
/// one content change makes the real pipeline run again.
#[test]
fn the_hit_path_serves_the_slot_and_a_content_move_invalidates_it() {
    let root = seeded("rescache-poison");
    let (found, _) = analyze(&root, None, None, None).unwrap();
    assert!(found.blocks.is_empty(), "two tiny files share nothing");
    let f = Filter {
        min_tokens: Params::default().guarantee(),
        min_distinct: crate::dedup::pairs::DEFAULT_MIN_DISTINCT,
    };
    let idx = open(&root);
    let d = digest(idx.raw()).unwrap();
    store(idx.raw(), d, f, &poison()).unwrap();
    drop(idx);
    let (served, _) = analyze(&root, None, None, None).unwrap();
    assert_eq!(served.blocks.len(), 1, "the poisoned slot must be served");
    assert_eq!(served.blocks[0].a_file, "poison.rs");
    std::fs::write(root.join("a.rs"), "fn a() { let x = 3; }\n").unwrap();
    let (fresh, _) = analyze(&root, None, None, None).unwrap();
    assert!(fresh.blocks.is_empty(), "a moved digest recomputes");
    let _ = std::fs::remove_dir_all(&root);
}

/// A run that recomputed must leave the slot describing ITS
/// result — the store rides every miss, which is the invariant
/// the poison test's third act depends on.
#[test]
fn keys_partition_by_digest_and_by_filter() {
    let root = seeded("rescache-keys");
    analyze(&root, None, None, None).unwrap();
    let idx = open(&root);
    let f = Filter {
        min_tokens: 40,
        min_distinct: 7,
    };
    let d = digest(idx.raw()).unwrap();
    store(idx.raw(), d, f, &poison()).unwrap();
    assert!(load(idx.raw(), d, f).unwrap().is_some());
    assert!(load(idx.raw(), d ^ 1, f).unwrap().is_none(), "digest keys");
    let other = Filter {
        min_tokens: 41,
        ..f
    };
    assert!(load(idx.raw(), d, other).unwrap().is_none(), "filter keys");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_unparseable_slot_is_a_miss_not_an_error() {
    let root = seeded("rescache-corrupt");
    let idx = open(&root);
    idx.raw()
        .execute(
            "INSERT OR REPLACE INTO result_cache
                 (k, digest, min_tokens, min_distinct, blocks)
                 VALUES (1, 7, 40, 7, 'not json')",
            (),
        )
        .unwrap();
    let f = Filter {
        min_tokens: 40,
        min_distinct: 7,
    };
    assert!(load(idx.raw(), 7, f).unwrap().is_none());
    let _ = std::fs::remove_dir_all(&root);
}
