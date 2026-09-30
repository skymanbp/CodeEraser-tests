use super::*;
use crate::fourclass::kinds::{KIND_FN, KIND_IMPL, KIND_NAMED, KIND_SECTION};
use crate::graph::wire::{
    EDGE_ASSET, EDGE_CONTAIN, EDGE_DOC_LINK, EDGE_DOC_REF, EDGE_IMPORT, EDGE_REFDEF_UNUSED,
};
use std::collections::BTreeSet;

/// The schema this side assembles by is the one the core judges by:
/// the golden reply that asked for the echo carries CE.Query.Schema
/// row for row (contracts/fixtures/query/golden.ndjson, id 1).
#[test]
fn the_schema_rows_are_the_cores_echo() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../contracts/fixtures/query/golden.ndjson"
    );
    let text = std::fs::read_to_string(path).expect("the query goldens");
    let echoed: Vec<Vec<i64>> = text
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .find_map(|d| {
            d["schema"]
                .as_array()
                .map(|_| serde_json::from_value(d["schema"].clone()).unwrap())
        })
        .expect("one golden reply echoes the schema");
    assert_eq!(schema_rows(), echoed);
    assert_eq!(schema_rows().len(), 27);
}

#[test]
fn every_schema_row_answers_by_name_and_by_code() {
    for (i, row) in schema_rows().iter().enumerate() {
        let p = pred_of(i as u32).expect("a code");
        assert_eq!((p.code as usize, row[1] as usize), (i, p.sorts.len()));
        assert_eq!(pred(p.name).map(|q| q.code), Some(p.code));
    }
    assert_eq!(pred("ref").map(|p| p.sorts.clone()), Some(vec![0, 0, 4, 3]));
    assert_eq!(
        pred_of(26).map(|p| (p.name, p.sorts.clone())),
        Some(("set", vec![5, 0]))
    );
    assert_eq!(
        pred(SET_PRED).map(|p| p.sorts[SET_ARG]),
        Some(sort_code("set"))
    );
    assert!(pred("nope").is_none() && pred_of(27).is_none());
    assert_eq!(IDB_FLOOR, 1000);
}

#[test]
fn the_sorts_round_trip_and_the_open_sort_reads_open() {
    for (code, name) in ["node", "dir", "unit", "int", "sym", "set"]
        .iter()
        .enumerate()
    {
        assert_eq!(sort_name(code as i64), *name);
        assert_eq!(sort_code(name), code as i64);
    }
    assert_eq!(
        (sort_name(-1), sort_name(6), sort_name(99)),
        ("open", "open", "open")
    );
}

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
