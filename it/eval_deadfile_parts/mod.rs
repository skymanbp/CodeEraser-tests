//! The dead-file exam (lane v235_licence, item L3; design notes in the
//! lane's DESIGN.md): the exam material for `ce deadcode`'s file-tier
//! REPORT, measured before any rule changes how it words a file nothing
//! references. Per corpus a frozen universe (`deadfile-slice-<corpus>`):
//! every file-tier dead row at the pinned tip with measured features —
//! the file's distinct top-level node kinds, its kept out-edges, whether
//! another file spells its basename / its stem as a word, the test /
//! fixture / data path conventions it sits under, its document class.
//! Per language a hash-ranked sample (`deadfile-sample-<lang>`). ONE
//! binding for the `#[ignore]` generator (generate.rs, features.rs) and
//! the CI gate (eval_deadfile.rs): constants, summary, draw. No RNG, no
//! clock. Stage Reviewed: one review doc per sample language is filed
//! (part 3A, the gate is eval_deadfile_review.rs); no precision doc exists.

pub mod audit;
pub mod classes;
pub mod features;
pub mod gate;
pub mod generate;
pub mod k0_sets;
pub mod k1;
pub mod k1_bind;
pub mod k1_lang;
pub mod k1_tables;

use crate::eval_lang_parts::draw::{quotas, ranked_by};
use crate::eval_support::UniverseFamily;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The corpora and their pinned tips, one `name tip` per line: the
/// twelve exam corpora as the scratch copies of lane v235_graphtrust
/// hold them (koreader at that lane's scratch commit dropping its
/// unseated submodules — ce refuses an unseated submodule), and this
/// repository at the lane's base.
const CORPORA_TEXT: &str = "cobra adbc8813901bba65827259daa8e22ff94ec1f30e
codeeraser af1c29958f2ee4d2322594784a4f9b7524559194
covid19model fcc30e2b8d046ddf3ef10dfc222e42b5cd732622
fmt 6d71f74624be5daa548073ff8e4e0c8aa5476010
gson 854c8255b625cf1e13c701a83ea9ccb4caaa576a
jsoup 093e2f58492c531667e551e8793513a41b22443e
koreader dc76367495d00cd30b0e934bcc6f9a167ac5154b
lua 0b29f408433e92953cc72b1d3e06c7ac8139e439
luarocks 2d2cc8eff2f03c23d142f8059146fb241dcf56b5
requests 8068356288978c4f54661ae6f95afe0e0831885e
ripgrep 3fce3b5bb0236da2df6d99672afb8a719642eca7
stringr ae054b1d28f630fee22ddb3cb7525396e62af4fe
zod 912f0f51b0ced654d0069741e7160834dca742ee";

/// (name, tip) of every exam corpus, in table order.
pub fn corpora() -> Vec<(&'static str, &'static str)> {
    let words: Vec<&'static str> = CORPORA_TEXT.split_whitespace().collect();
    words.chunks(2).map(|pair| (pair[0], pair[1])).collect()
}

/// Every doc of the exam carries this generation.
pub const GENERATION: u32 = 1;
/// (slice, sample)
pub const SCHEMAS: (&str, &str) = (
    "ce.eval-deadfile-slice/1.0.0",
    "ce.eval-deadfile-sample/1.0.0",
);
/// The per-language sample family.
pub const SAMPLES: &str = "deadfile-sample";
/// The doc family the exam files later, absent at stage Reviewed.
pub const LATER: [&str; 1] = ["deadfile-precision"];

/// Pre-registered: per language at most TOTAL primary rows by the
/// language exams' quota rule over corpora (lang draw `quotas`: each
/// corpus first min(15, pool), the rest by largest remainder) plus
/// RESERVE rows, the audit-domain rank over the unpicked rest; a
/// language whose universe holds fewer than WHOLE_BELOW rows is taken
/// whole, with no reserve.
pub const TOTAL: u64 = 100;
pub const RESERVE: usize = 40;
pub const WHOLE_BELOW: usize = 30;
/// (rank, audit) hash domains.
pub const DOMAINS: (&str, &str) = ("ce-deadfile-rank-v1", "ce-deadfile-audit-v1");
/// The rank payload: one row per file, so (corpus, commit, path) is
/// its identity.
pub const FIELDS: [&str; 3] = ["corpus", "commit", "path"];

/// Path segments a test / fixture / data convention of SOME language
/// names (brief L3 (d)): any of these as a whole segment, compared
/// case-insensitively; `resources` only below a test segment.
pub const CONVENTION_SEGMENTS: [&str; 8] = [
    "test", "tests", "testes", "spec", "fixtures", "testdata", "bench", "examples",
];

pub fn constants() -> Value {
    json!({
        "total": TOTAL, "reserve": RESERVE, "whole_below": WHOLE_BELOW,
        "domains": {"rank": DOMAINS.0, "audit": DOMAINS.1},
        "convention_segments": CONVENTION_SEGMENTS,
    })
}

pub const SLICE: UniverseFamily = UniverseFamily {
    family: "deadfile-slice",
    constants,
    summarize: summary,
};

/// A slice's summary, re-derivable from its rows alone (G1): rows per
/// language, and per language how many rows carry each feature.
pub fn summary(rows: &[Value]) -> Value {
    let by = rows.iter().fold(BTreeMap::new(), |mut by: Cells, r| {
        let lang = r["lang"].as_str().expect("lang").to_string();
        add_flags(by.entry(lang).or_default(), r);
        by
    });
    json!({"rows": rows.len(), "by_lang": by})
}

type Cells = BTreeMap<String, BTreeMap<&'static str, u64>>;

fn add_flags(cell: &mut BTreeMap<&'static str, u64>, r: &Value) {
    for (name, on) in feature_flags(r) {
        *cell.entry(name).or_insert(0) += u64::from(on);
    }
}

/// One row's features as named flags (the summary's columns and
/// DESIGN.md's census table read the same names).
pub fn feature_flags(r: &Value) -> [(&'static str, bool); 8] {
    let n = |k: &str| r[k].as_u64().expect(k);
    [
        ("rows", true),
        ("vouched", r["confidence"] == json!(2)),
        (
            "private",
            r["verdict"].as_str().expect("verdict").ends_with("private"),
        ),
        ("out_edges", n("out_edges") > 0),
        ("basename_spelled", n("basename_spelled") > 0),
        ("stem_spelled", n("stem_spelled") > 0),
        (
            "convention",
            !r["conventions"].as_array().expect("conventions").is_empty(),
        ),
        ("document", !r["doc_class"].is_null()),
    ]
}

/// The frozen draw of one language: allocation per corpus, the primary
/// rows (audit-domain order), the reserve.
pub fn draw(pool: &[Value]) -> (BTreeMap<String, u64>, Vec<Value>, Vec<Value>) {
    let mut ranked: Vec<Value> = pool
        .iter()
        .map(|r| ranked_by(r, DOMAINS, &FIELDS))
        .collect();
    let key = |r: &Value, k: &str| r[k].as_str().expect(k).to_string();
    if ranked.len() < WHOLE_BELOW {
        ranked.sort_by_cached_key(|r| key(r, "audit"));
        let mut all = BTreeMap::new();
        for r in &ranked {
            *all.entry(key(r, "corpus")).or_insert(0) += 1;
        }
        return (all, ranked, Vec::new());
    }
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for r in &ranked {
        *counts.entry(key(r, "corpus")).or_insert(0) += 1;
    }
    let allocation = quotas_of(&counts);
    ranked.sort_by_cached_key(|r| (key(r, "corpus"), key(r, "rank")));
    let (mut primary, mut rest) = (Vec::new(), Vec::new());
    let mut taken: BTreeMap<String, u64> = BTreeMap::new();
    for r in ranked {
        let n = taken.entry(key(&r, "corpus")).or_insert(0);
        if *n < allocation[&key(&r, "corpus")] {
            *n += 1;
            primary.push(r);
        } else {
            rest.push(r);
        }
    }
    primary.sort_by_cached_key(|r| key(r, "audit"));
    rest.sort_by_cached_key(|r| key(r, "audit"));
    rest.truncate(RESERVE);
    (allocation, primary, rest)
}

/// The language exams' quota rule over corpora (TOTAL = 100 there too).
fn quotas_of(counts: &BTreeMap<String, u64>) -> BTreeMap<String, u64> {
    assert_eq!(crate::eval_lang_parts::TOTAL, TOTAL, "one quota rule");
    quotas(counts)
}

/// The doc of a family at this exam's generation.
pub fn doc_path(family: &str, key: &str) -> String {
    crate::eval_support::eval_doc_v(&format!("{family}-{key}"), GENERATION)
}
