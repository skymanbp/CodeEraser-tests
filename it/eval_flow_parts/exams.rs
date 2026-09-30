//! The exam table of the v2.31 flow exams — one text, a row per
//! language in the booklet's language order (analysis-track.md §1
//! item 4): `lang | name=tip … | ext … | stage`. A text rather than ten struct
//! literals: the clone gate reads a run of same-shaped rows as copies
//! of each other (the language exams' table carries its rows in
//! full because each states a different column).

use super::{FlowExam, LADDER_FIRST, LOWERING, Stage};
use std::collections::BTreeSet;
use std::sync::LazyLock;

/// The v2.30 exam registry's first corpus for C, C++, Java, Lua and R,
/// the M1 crosscheck corpora for Python, TypeScript, Go and Rust, zod's
/// `.tsx` files for TSX, and this repository at the commit the lowering
/// landed in (a clone of its own, apart from the HTML exam's) as
/// Rust's second corpus (booklet §5.5).
const TABLE: &str = "\
python     | requests=8068356288978c4f54661ae6f95afe0e0831885e | py | audited
typescript | zod=912f0f51b0ced654d0069741e7160834dca742ee | ts mts cts | audited
tsx        | zod=912f0f51b0ced654d0069741e7160834dca742ee | tsx | audited
rust       | ripgrep=3fce3b5bb0236da2df6d99672afb8a719642eca7 codeeraser-flow=5278e747f65687f8afa457395b82594f5f2978d1 | rs | audited
go         | cobra=adbc8813901bba65827259daa8e22ff94ec1f30e | go | audited
c          | lua=0b29f408433e92953cc72b1d3e06c7ac8139e439 | c | audited
cpp        | fmt=6d71f74624be5daa548073ff8e4e0c8aa5476010 | cpp cc cxx hpp hh hxx h inl | audited
java       | gson=854c8255b625cf1e13c701a83ea9ccb4caaa576a | java | audited
lua        | luarocks=2d2cc8eff2f03c23d142f8059146fb241dcf56b5 | lua | audited
r          | stringr=ae054b1d28f630fee22ddb3cb7525396e62af4fe | R r | audited";

/// Every exam, all at their first generation. The stage column is the
/// one the audit and the scoring commits flip, a word per row
/// (`sampled` -> `audited` in the commit that files the language's
/// `flow-review-<lang>-v1.json`, -> `scored` with its precision doc);
/// the review gate (eval_flow_review.rs) holds the word and the disk
/// equal.
pub static EXAMS: LazyLock<Vec<FlowExam>> = LazyLock::new(|| TABLE.lines().map(parse).collect());

fn leak(s: &str) -> &'static str {
    Box::leak(s.to_owned().into_boxed_str())
}

fn parse(line: &str) -> FlowExam {
    let cols: Vec<&str> = line.split('|').map(str::trim).collect();
    let corpus = |c: &str| {
        let (name, tip) = c.split_once('=').expect("name=tip");
        (leak(name), leak(tip))
    };
    FlowExam {
        lang: leak(cols[0]),
        corpora: cols[1].split_whitespace().map(corpus).collect(),
        exts: cols[2].split_whitespace().map(leak).collect(),
        stage: stage(cols[3]),
        generation: 1,
        ladder: LOWERING,
        ladder_first: LADDER_FIRST,
    }
}

fn stage(word: &str) -> Stage {
    match word {
        "sampled" => Stage::Sampled,
        "audited" => Stage::Audited,
        "scored" => Stage::Scored,
        w => panic!("{w}: no stage"),
    }
}

pub fn exam(lang: &str) -> &'static FlowExam {
    EXAMS
        .iter()
        .find(|e| e.lang == lang)
        .unwrap_or_else(|| panic!("{lang}: no flow exam"))
}

/// Every slice key `<lang>-<corpus>`, sorted — the frozen-set anchor.
pub fn keys() -> BTreeSet<String> {
    EXAMS
        .iter()
        .flat_map(|e| e.corpora.iter().map(|(c, _)| e.key(c)))
        .collect()
}

/// Every exam language, sorted.
pub fn langs() -> BTreeSet<String> {
    EXAMS.iter().map(|e| e.lang.to_string()).collect()
}
