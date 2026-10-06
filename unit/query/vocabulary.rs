//! The live legend's vocabulary (plan v2.33 W1 moved the fact schema's
//! names into the core; the schema's own legs read the frozen legend,
//! unit/query/legend.rs).

use super::*;
use crate::fourclass::kinds::{KIND_FN, KIND_IMPL, KIND_NAMED, KIND_SECTION};
use crate::graph::wire::{
    EDGE_ASSET, EDGE_CONTAIN, EDGE_DOC_LINK, EDGE_DOC_REF, EDGE_IMPORT, EDGE_REFDEF_UNUSED,
};
use std::collections::BTreeSet;

/// The enum words the assembler spells are the graph's own codes
/// by position, and no two hash alike.
#[test]
fn the_vocabulary_mirrors_the_graphs_codes_and_hashes_apart() {
    assert_eq!(
        [
            EDGE_IMPORT,
            EDGE_DOC_LINK,
            EDGE_DOC_REF,
            EDGE_ASSET,
            EDGE_CONTAIN,
            EDGE_REFDEF_UNUSED
        ]
        .map(|c| REF_KINDS[c as usize]),
        [
            "import", "doc_link", "doc_ref", "asset", "contain", "refdef"
        ]
    );
    assert_eq!(
        [KIND_FN, KIND_NAMED, KIND_IMPL, KIND_SECTION].map(|k| UNIT_KINDS[k as usize - 1]),
        ["fn", "named", "impl", "section"]
    );
    assert_eq!(&ROLE_NAMES[6..], ["declared", "foreign", "unit", "asset"]);
    assert_eq!(
        (NODE_KINDS[KIND_ASSET], NODE_KINDS[KIND_PROSE]),
        ("asset", "prose")
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
