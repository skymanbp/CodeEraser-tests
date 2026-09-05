//! ROI instrument for the same-role advisor (plan v2.29 step 2; spec
//! `.ccm/similar-spec-2026-09-05.md` §七, to be published as booklet
//! 15): every code unit of five corpora — this repository through its
//! own walk, and the four crosscheck fixture trees each as a corpus
//! of its own, because ce.toml excludes them from the self walk —
//! queries the rest of its corpus under two arms, bare and
//! PPMI-widened, and the top-5 of each arm rides out with its evidence
//! integers. Since step 3 the corpus is the PERSISTED one: each corpus
//! is indexed into a scratch `.ce/index.db`, the bags are read back
//! through the product's reader, and every query is ranked off the
//! tables — with the in-memory corpus built from the same stored bags
//! asserted to agree hit for hit, so the frozen numbers below pin the
//! store and the differential upkeep as well as the scoring. A
//! deterministic stratified sample of queries (sha256 rank, quotas in
//! `parts`) is frozen for arbitration as
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
//! CE_SIMILAR_SAMPLE_GEN=<n> draws generation n: the same quotas over
//! the ranks no earlier generation's oracle arbitrated — the holdout
//! the step-2 candidates are retested on (`similar-sample-v<n>.json`).

use crate::common;
use crate::similar_replay_parts as parts;
use codeeraser::dedup;
use codeeraser::similar::bm25::{self, Corpus, Doc, Hit, Postings, QueryTerm};
use codeeraser::similar::file_bags;
use codeeraser::similar::ppmi::{self, Table};
use codeeraser::similar::reader::Reader;
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

/// One corpus measured: its bags (as stored), its PPMI table, every
/// unit's two top-K lists (bare, widened), and the texts for the
/// packet.
pub struct Measured {
    pub name: &'static str,
    pub corpus: Corpus,
    pub table: Table,
    pub ranked: Vec<(Vec<Hit>, Vec<Hit>)>,
    pub texts: BTreeMap<String, String>,
}

/// The stored bags must be exactly what the term road builds from the
/// text the index saw — identity for identity (the bag universe IS the
/// unitsig universe: the reader seats every own unitsig row, the
/// fresh bags come from the same throat) and term for term (index and
/// query share one road, and the store lost nothing on the way in).
/// Returns the texts, read once through the judgment-side decode.
fn round_trip(root: &Path, name: &str, docs: &[Doc]) -> BTreeMap<String, String> {
    let mut texts = BTreeMap::new();
    let mut paths: Vec<&str> = docs.iter().map(|d| d.path.as_str()).collect();
    paths.dedup();
    let mut seen = 0;
    for path in paths {
        let (text, lang) = dedup::walked_text(root, path).expect("walked text");
        let mut fresh = file_bags(&text, lang);
        fresh.sort_by(|a, b| (&a.key, a.nth).cmp(&(&b.key, b.nth)));
        let stored: Vec<&Doc> = docs.iter().filter(|d| d.path == path).collect();
        assert_eq!(fresh.len(), stored.len(), "{name}: {path}: unit count");
        for (f, s) in fresh.iter().zip(&stored) {
            let want = (&f.key, f.nth, f.start_line, f.end_line, &f.terms);
            let got = (
                &s.bag.key,
                s.bag.nth,
                s.bag.start_line,
                s.bag.end_line,
                &s.bag.terms,
            );
            assert_eq!(
                got, want,
                "{name}: {path}: stored bag drifted from the term road"
            );
            seen += 1;
        }
        texts.insert(path.to_string(), text);
    }
    assert_eq!(seen, docs.len(), "{name}: every stored unit has a file");
    texts
}

/// Both arms of one query, ranked off the tables — and off the
/// in-memory corpus built from the same stored bags, which must agree
/// hit for hit: one ranking road, two posting sources.
fn arms(reader: &Reader<'_>, corpus: &Corpus, table: &Table, i: usize) -> (Vec<Hit>, Vec<Hit>) {
    let bare = corpus.query_of(i);
    let (mut widened, mut in_memory) = (bare.clone(), bare.clone());
    ppmi::expand(reader, &mut widened).expect("cooc rows");
    ppmi::expand(table, &mut in_memory).expect("in-memory");
    assert_eq!(widened, in_memory, "seat {i}: widened query");
    let rank = |q: &[QueryTerm]| {
        let hits = bm25::top_k(reader, q, K, Some(i)).expect("tables");
        assert_eq!(
            hits,
            bm25::top_k(corpus, q, K, Some(i)).expect("in-memory"),
            "seat {i}: the persisted and in-memory roads rank apart"
        );
        hits
    };
    (rank(&bare), rank(&widened))
}

/// Index one corpus through the product's own walk into a scratch
/// index, read the bags back, and rank every unit against the rest
/// under both arms.
pub fn measure(root: &Path, name: &'static str) -> Measured {
    let scratch = common::tmp(&format!("similar-replay-{name}"));
    let (idx, _db) =
        dedup::refreshed_index(root, Some(scratch.join("index.db"))).expect("scratch index");
    let reader = Reader::open(&idx).expect("reader");
    let docs = reader.docs().expect("stored bags");
    let texts = round_trip(root, name, &docs);
    let corpus = Corpus::build(docs);
    let table = Table::build(&corpus);
    let ranked = (0..corpus.docs.len())
        .map(|i| arms(&reader, &corpus, &table, i))
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
    let generation: u32 = std::env::var("CE_SIMILAR_SAMPLE_GEN")
        .map(|g| {
            g.parse()
                .expect("CE_SIMILAR_SAMPLE_GEN: a generation number")
        })
        .unwrap_or(1);
    let exclude = parts::arbitrated_before(generation);
    let (rows, drawn) = parts::sample(&measured, &exclude);
    println!(
        "{}",
        json!({"sample": drawn, "rows": rows.len(), "generation": generation, "excluded": exclude.len()})
    );
    let mut doc = parts::document(&measured, Value::Object(summary), rows);
    doc["generation"] = json!(generation);
    if generation > 1 {
        doc["holdout_of"] = json!(format!("similar-oracle-v{}", generation - 1));
    }
    if crate::facts::blessing() {
        let path = crate::eval_support::eval_doc_v("similar-sample", generation);
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
