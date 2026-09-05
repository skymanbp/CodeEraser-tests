//! ROI instrument for the same-role advisor (plan v2.29 step 2; spec
//! `.ccm/similar-spec-2026-09-05.md` §七, to be published as booklet
//! 15): every code unit of five corpora — this repository through its
//! own walk, and the four crosscheck fixture trees each as a corpus
//! of its own, because ce.toml excludes them from the self walk —
//! queries the rest of its corpus under two arms, bare and
//! PPMI-widened, and the top-5 of each arm rides out with its evidence
//! integers. A deterministic stratified sample of queries (sha256
//! rank, quotas in `parts`) is frozen for arbitration as
//! `contracts/eval/similar-sample-v1.json` under CE_BLESS=1; the
//! arbitration record and the metrics gate live beside it once the
//! oracle is frozen (eval_similar_precision). Standing `--ignored`
//! leg: the measurement it exercises is live product code, so the
//! sample regenerates from the tree (EVAL-SET retirement rule).
//!
//!   cargo test --release --test it -- --ignored similar_replay --nocapture
//!
//! CE_BLESS=1 writes the sample doc; CE_SIMILAR_PACKET=<file> also
//! writes the arbiter's packet (verbatim source of every sampled
//! query and candidate — RM18: texts only, no judgment fields).

use crate::common;
use crate::similar_replay_parts as parts;
use codeeraser::dedup;
use codeeraser::similar::bm25::{Corpus, Doc, Hit};
use codeeraser::similar::file_bags;
use codeeraser::similar::ppmi::Table;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Candidates kept per arm.
pub const K: usize = 5;

/// The five corpora: the self walk, then the four crosscheck fixture
/// trees (SOURCES.md pins their upstream commits; here they are
/// files of this tree, so the self tip identifies them).
pub const CORPORA: [(&str, &str); 5] = [
    ("self", ""),
    ("go", "contracts/fixtures/crosscheck/go"),
    ("python", "contracts/fixtures/crosscheck/python"),
    ("rust", "contracts/fixtures/crosscheck/rust"),
    ("typescript", "contracts/fixtures/crosscheck/typescript"),
];

/// One corpus measured: its index, its PPMI table, every unit's two
/// top-K lists (bare, widened), and the texts for the packet.
pub struct Measured {
    pub name: &'static str,
    pub corpus: Corpus,
    pub table: Table,
    pub ranked: Vec<(Vec<Hit>, Vec<Hit>)>,
    pub texts: BTreeMap<String, String>,
}

/// Index one corpus through the product's own walk and the product's
/// own unit universe, then bag every unit from the text the index
/// saw. The identity multiset of the bags must equal the unitsig
/// rows — the drift ensure that keeps "the bag universe is the T3
/// universe" a fact rather than a sentence.
fn corpus_docs(root: &Path, name: &str) -> (Vec<Doc>, BTreeMap<String, String>) {
    let scratch = common::tmp(&format!("similar-replay-{name}"));
    let (idx, _db) =
        dedup::refreshed_index(root, Some(scratch.join("index.db"))).expect("scratch index");
    let rows = dedup::unitcache::unit_rows(&idx).expect("unit rows");
    let mut paths: Vec<&str> = rows.iter().map(|r| r.path.as_str()).collect();
    paths.dedup();
    let (mut docs, mut texts) = (Vec::new(), BTreeMap::new());
    for path in paths {
        let (text, lang) = dedup::walked_text(root, path).expect("walked text");
        for bag in file_bags(&text, lang) {
            docs.push(Doc {
                path: path.to_string(),
                bag,
            });
        }
        texts.insert(path.to_string(), text);
    }
    let want: Vec<(&str, &str, i64)> = rows
        .iter()
        .map(|r| (r.path.as_str(), r.key.as_str(), r.nth))
        .collect();
    let mut got: Vec<(&str, &str, i64)> = docs
        .iter()
        .map(|d| (d.path.as_str(), d.bag.key.as_str(), d.bag.nth))
        .collect();
    got.sort_unstable();
    assert_eq!(
        got, want,
        "{name}: bag universe drifted from the unitsig universe"
    );
    (docs, texts)
}

/// Every unit of a corpus against the rest under both arms.
pub fn measure(root: &Path, name: &'static str) -> Measured {
    let (docs, texts) = corpus_docs(root, name);
    let corpus = Corpus::build(docs);
    let table = Table::build(&corpus);
    let ranked = (0..corpus.docs.len())
        .map(|i| {
            let bare = corpus.query_of(i);
            let mut widened = bare.clone();
            table.expand(&mut widened);
            (
                corpus.top_k(&bare, K, Some(i)),
                corpus.top_k(&widened, K, Some(i)),
            )
        })
        .collect();
    Measured {
        name,
        corpus,
        table,
        ranked,
        texts,
    }
}

/// The corpus tally: how often the bare / widened top-1 carries the
/// role bit, how often the two arms agree at top-1, how often the
/// top-1 sits in the query's own file — the numbers the sample
/// quotas and the spec's thresholds are read against.
pub fn tally(m: &Measured) -> Value {
    let n = m.corpus.docs.len();
    let mut c = [0usize; 5];
    for (i, (bare, wide)) in m.ranked.iter().enumerate() {
        let (Some(b), Some(w)) = (bare.first(), wide.first()) else {
            continue;
        };
        c[0] += 1;
        c[1] += usize::from(b.role);
        c[2] += usize::from(w.role);
        c[3] += usize::from(b.doc == w.doc);
        c[4] += usize::from(m.corpus.docs[b.doc].path == m.corpus.docs[i].path);
    }
    json!({
        "units": n,
        "avg_len": m.corpus.avg_len(),
        "ppmi_capped_units": m.table.capped_units,
        "answered": c[0],
        "role_top1_bare": c[1],
        "role_top1_widened": c[2],
        "top1_agree": c[3],
        "top1_same_file_bare": c[4],
    })
}

fn root_of(rel: &str) -> PathBuf {
    let root = common::repo_root();
    if rel.is_empty() { root } else { root.join(rel) }
}

/// The instrument: measure the five corpora, print each tally as one
/// JSON line, draw the sample, and freeze it under CE_BLESS=1.
#[test]
#[ignore]
fn similar_replay() {
    let measured: Vec<Measured> = CORPORA
        .iter()
        .map(|(name, rel)| measure(&root_of(rel), name))
        .collect();
    let mut summary = serde_json::Map::new();
    for m in &measured {
        let t = tally(m);
        println!("{}", json!({"corpus": m.name, "tally": t}));
        summary.insert(m.name.to_string(), t);
    }
    let (rows, drawn) = parts::sample(&measured);
    println!("{}", json!({"sample": drawn, "rows": rows.len()}));
    let doc = parts::document(&measured, Value::Object(summary), rows);
    if crate::facts::blessing() {
        let path = crate::eval_support::eval_doc("similar-sample");
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&doc).expect("json") + "\n",
        )
        .expect("write sample");
        println!("wrote {path}");
    }
    if let Some(packet) = std::env::var_os("CE_SIMILAR_PACKET") {
        std::fs::write(&packet, parts::packet(&measured, &doc)).expect("write packet");
        println!("wrote packet {}", Path::new(&packet).display());
    }
}
