//! Typed readers of core-laid documents (plan v2.33 step 11a, audit A
//! row 36): the join and structure documents read back into structs
//! for the legs that assert on their fields. The product never read
//! them typed — the core lays the document and its console lines — so
//! the readers live with their only readers, here. A `Bound` reader
//! keeps the document itself beside it.
//!
//! Every field is deserialized — reading the document into the struct
//! is the shape check — while each leg asserts on a few of them, so
//! the fields no leg reads are allowed here by name of that reason.
#![allow(dead_code)]

use codeeraser::join::Pos;
use codeeraser::join::churn_unit::{Lines, UnitRow};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// A document read back into a typed reader that keeps the document.
pub trait Bound: DeserializeOwned {
    fn keep(&mut self, doc: Value);
}

/// The bound document read back, the document kept beside it.
pub fn read_bound<T: Bound>(doc: Value, family: &str) -> anyhow::Result<T> {
    let mut r: T = codeeraser::document::read(&doc, family)?;
    r.keep(doc);
    Ok(r)
}

/// The join document read typed; `doc` is the document itself.
#[derive(Debug, Deserialize)]
pub struct JoinReport {
    pub days: u32,
    pub commits: usize,
    pub files: Vec<JoinFileRow>,
    pub units: Vec<UnitRow>,
    pub degraded: Option<String>,
    #[serde(skip)]
    pub doc: Value,
}

/// One Tier F row: a similar file pair with all three legs and the
/// core's verdict on it.
#[derive(Debug, Deserialize)]
pub struct JoinFileRow {
    pub a: String,
    pub b: String,
    pub blocks: usize,
    pub tokens: usize,
    /// T3 near-miss unit pairs between the two files (5b-9).
    pub near_miss: usize,
    pub graph_a: Option<Pos>,
    pub graph_b: Option<Pos>,
    pub churn_a: Lines,
    pub churn_b: Lines,
    /// None = the pair is outside the churn report's co-change table.
    pub cochange: Option<usize>,
    /// The core's verdict for this pair (2.33.0, H4); None for a
    /// self-pair (the wire's u < v contract cannot carry it) and when
    /// the judgment degraded.
    pub verdict: Option<String>,
    pub severity: Option<i64>,
    pub confidence: Option<i64>,
}

/// The structure document read typed; `doc` is the document itself.
#[derive(Debug, Deserialize)]
pub struct StructureReport {
    pub score: i64,
    #[serde(rename = "scoreScale")]
    pub scale: i64,
    pub entropy: Vec<[i64; 2]>,
    pub axes: Vec<[i64; 2]>,
    pub findings: Vec<Finding>,
    pub dirs: usize,
    pub divergence: Option<i64>,
    pub deviations: Vec<Deviation>,
    #[serde(rename = "declaredDirs")]
    pub declared: usize,
    pub deep: bool,
    pub days: Option<u32>,
    pub split: bool,
    #[serde(rename = "splitCandidates", default)]
    pub split_candidates: Vec<Candidate>,
    #[serde(rename = "sizeExempt", default)]
    pub size_exempt: Vec<Exempt>,
    #[serde(skip)]
    pub doc: Value,
}

#[derive(Debug, Deserialize)]
pub struct Finding {
    pub dir: String,
    pub axis: i64,
}

#[derive(Debug, Deserialize)]
pub struct Deviation {
    pub dir: String,
    pub kind: i64,
}

/// A viable seam: the file, the line it falls after, the unit before
/// it, and both sides of the price.
#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub path: String,
    #[serde(rename = "afterLine")]
    pub after: u64,
    pub unit: String,
    #[serde(rename = "benefitMilli")]
    pub benefit: i64,
    #[serde(rename = "costMilli")]
    pub cost: i64,
}

/// A file past the soft line with no viable seam (0/0 = no seam).
#[derive(Debug, Deserialize)]
pub struct Exempt {
    pub path: String,
    #[serde(rename = "benefitMilli")]
    pub benefit: i64,
    #[serde(rename = "costMilli")]
    pub cost: i64,
}

impl Bound for JoinReport {
    fn keep(&mut self, doc: Value) {
        self.doc = doc;
    }
}

impl Bound for StructureReport {
    fn keep(&mut self, doc: Value) {
        self.doc = doc;
    }
}
