//! The live legend's vocabulary (plan v2.33 W1 moved the fact schema's
//! names into the core; the schema's own legs read the frozen legend,
//! unit/query/legend.rs).

use super::*;
use crate::fourclass::kinds::{KIND_FN, KIND_IMPL, KIND_NAMED, KIND_NAMES, KIND_SECTION};
use crate::graph::deadcode::ROLE_NAMES;
use crate::graph::wire as w;
use std::collections::BTreeSet;

/// The enum words the assembler spells are named by code at their
/// owners (each table indexed by its family's codes, which run dense
/// from the first), the words a query program writes are the frozen
/// ones, and no two hash alike.
#[test]
fn the_vocabulary_names_the_owners_codes_and_hashes_apart() {
    let edges = [
        w::EDGE_IMPORT,
        w::EDGE_DOC_LINK,
        w::EDGE_DOC_REF,
        w::EDGE_ASSET,
    ];
    let tail = [w::EDGE_CONTAIN, w::EDGE_REFDEF_UNUSED];
    let dense: Vec<i64> = edges.into_iter().chain(tail).collect();
    assert_eq!(dense, (0..w::EDGE_NAMES.len() as i64).collect::<Vec<_>>());
    let grans = [w::GRAN_FILE, w::GRAN_PACKAGE, w::GRAN_SECTION];
    assert_eq!(grans, [0, 1, 2]);
    let kinds = [KIND_FN, KIND_NAMED, KIND_IMPL, KIND_SECTION].map(|k| k - KIND_FN);
    assert_eq!(kinds, [0, 1, 2, 3]);
    assert_eq!(named(&w::EDGE_NAMES, w::EDGE_REFDEF_UNUSED), Some("refdef"));
    assert_eq!(named(&KIND_NAMES, KIND_IMPL - KIND_FN), Some("impl"));
    assert_eq!(named(&ROLE_NAMES, 8), Some("unit"));
    assert_eq!(
        (named(&ROLE_NAMES, -1), named(&ROLE_NAMES, 10)),
        (None, None)
    );
    let kinds = [0, 1, 2, KIND_ASSET, KIND_PROSE, 5].map(node_kind);
    assert_eq!(
        kinds,
        ["file", "pkg", "section", "asset", "prose", "unknown"]
    );
    let words: Vec<&str> = vocabulary().collect();
    let distinct: BTreeSet<&str> = words.iter().copied().collect();
    let spelled = |word: &str| words.iter().filter(|w| **w == word).count();
    assert_eq!(
        (spelled("asset"), spelled("section")),
        (3, 2),
        "`asset` is a node kind, a role and a reference kind; `section` a node kind and a unit kind"
    );
    assert_eq!(
        words.len(),
        distinct.len() + 2 + 1,
        "no other word is spelled twice"
    );
    let hashes: BTreeSet<u64> = distinct.iter().map(|w| sym(w)).collect();
    assert_eq!(hashes.len(), distinct.len());
    assert_eq!(sym("foo"), crate::dedup::tokens::fnv1a(b"foo"));
}
