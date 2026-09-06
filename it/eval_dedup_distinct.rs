//! The `min_distinct` calibration, re-measured by the product (plan
//! v2.29 step 9, O45). Booklet 01 §7 cited the floor's calibration —
//! arbitrated data-row false positives at `distinct <= 6`, arbitrated
//! clones at `>= 7` — through a source comment pointing at
//! DEDUP-CALIBRATION.md, and that record's per-corpus breakdown had no
//! executor: typed on 2026-08-07, re-read by nothing. This leg
//! re-measures it with `dedup::analyze` at the floor turned OFF
//! (`min_distinct = 0`), so every block the operating point reports
//! keeps its `distinct`: the histogram, the count the shipped floor
//! suppresses and the exact blocks it suppresses fall out of one run
//! per corpus, and the arbitration's families are read off the file
//! names. The instrument reproduces the numbers, not the reading — the
//! true/false labels stay the 2026-08-07 record's.
//!
//! Two roads. The CI leg measures the tracked crosscheck fixtures live
//! and holds the frozen doc's fixtures row, every row's arithmetic and
//! the table DEDUP-CALIBRATION.md renders from the doc (CE_BLESS=1
//! rewrites the table). The ignored leg re-measures the four pinned
//! external corpora too (`.ce-eval/corpora`, tips checked) and writes
//! the doc and the table:
//!   CE_BLESS=1 cargo test --release --test it -- --ignored eval_dedup_distinct::regenerate --nocapture

use crate::common;
use crate::eval_support::corpus::PINNED_CORPORA;
use codeeraser::dedup::pairs::DEFAULT_MIN_DISTINCT;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

const DOC: &str = "contracts/eval/dedup-distinct-v1.json";
const SCHEMA: &str = "ce.eval-dedup-distinct/1.0.0";
const RECORD: &str = "contracts/fixtures/crosscheck/DEDUP-CALIBRATION.md";
const FIXTURES: &str = "contracts/fixtures/crosscheck";
/// The fixtures are tracked files, not a pinned checkout: their
/// provenance is SOURCES.md beside them.
const TRACKED: &str = "tracked (SOURCES.md)";
const BEGIN: &str = "<!-- distinct:begin -->";
const END: &str = "<!-- distinct:end -->";

/// One corpus with the floor off: every block at the operating point's
/// token line, its `distinct` kept — the histogram and, below the
/// shipped floor, the block itself by both ends.
fn measure(name: &str, tip: &str, root: &Path) -> Value {
    let db = common::tmp(&format!("eval-distinct-{name}")).join("index.db");
    let (found, summary) =
        codeeraser::dedup::analyze(root, Some(db), None, Some(0)).expect("analyze");
    let mut histogram: BTreeMap<String, usize> = BTreeMap::new();
    for b in &found.blocks {
        *histogram.entry(format!("{:02}", b.distinct)).or_default() += 1;
    }
    let suppressed: Vec<Value> = found
        .blocks
        .iter()
        .filter(|b| b.distinct < DEFAULT_MIN_DISTINCT)
        .map(|b| {
            json!({
                "a": format!("{}:{}", b.a_file, b.a_start),
                "b": format!("{}:{}", b.b_file, b.b_start),
                "tokens": b.tokens,
                "distinct": b.distinct,
            })
        })
        .collect();
    json!({
        "name": name,
        "tip": tip,
        "files": serde_json::to_value(&summary).expect("summary")["files"],
        "blocks": found.blocks.len(),
        "suppressed": suppressed.len(),
        "histogram": histogram,
        "suppressed_blocks": suppressed,
    })
}

/// A block's two ends as the reader knows the family: the file names,
/// one when the block is a file's own repetition.
fn family(block: &Value) -> String {
    let name = |k: &str| {
        let spelled = block[k].as_str().expect("an end");
        let file = spelled.rsplit_once(':').map_or(spelled, |(f, _)| f);
        file.rsplit('/').next().unwrap_or(file).to_string()
    };
    let (a, b) = (name("a"), name("b"));
    if a == b { a } else { format!("{a} ↔ {b}") }
}

/// How many families the table names per corpus before it counts the
/// rest: zod's 623 suppressed blocks fall in 50-odd files, which is a
/// column for the doc, not for a table a reader scans.
const FAMILIES_SHOWN: usize = 6;

/// The suppressed blocks by family, largest first, capped for the
/// table; every block stays in the doc.
fn families(c: &Value) -> String {
    let mut by: BTreeMap<String, usize> = BTreeMap::new();
    for b in c["suppressed_blocks"].as_array().expect("blocks") {
        *by.entry(family(b)).or_default() += 1;
    }
    if by.is_empty() {
        return "—".to_string();
    }
    let mut ranked: Vec<(&String, &usize)> = by.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    let mut shown: Vec<String> = ranked
        .iter()
        .take(FAMILIES_SHOWN)
        .map(|(f, n)| format!("{f} ×{n}"))
        .collect();
    if ranked.len() > FAMILIES_SHOWN {
        shown.push(format!("… 另 {} 族", ranked.len() - FAMILIES_SHOWN));
    }
    shown.join(", ")
}

/// The table the record renders from the doc: one row per corpus.
fn render(doc: &Value) -> String {
    let mut s = String::from(
        "| 语料 @ tip | 文件 | 块（下限关） | distinct ≤ 6 = 出厂 7 抑制 | 落在哪些文件（族 = 两端文件名） |\n|---|---|---|---|---|\n",
    );
    for c in doc["corpora"].as_array().expect("corpora") {
        s += &format!(
            "| {} @ {} | {} | {} | {} | {} |\n",
            c["name"].as_str().expect("name"),
            c["tip"].as_str().expect("tip"),
            c["files"],
            c["blocks"],
            c["suppressed"],
            families(c)
        );
    }
    s
}

/// Every row's arithmetic closes: the histogram sums to the blocks and
/// the suppressed count is exactly the mass below the floor.
fn coherent(c: &Value) {
    let hist = c["histogram"].as_object().expect("histogram");
    let total: u64 = hist.values().map(|v| v.as_u64().expect("count")).sum();
    let below: u64 = hist
        .iter()
        .filter(|(k, _)| k.parse::<usize>().expect("distinct") < DEFAULT_MIN_DISTINCT)
        .map(|(_, v)| v.as_u64().expect("count"))
        .sum();
    assert_eq!(
        Some(total),
        c["blocks"].as_u64(),
        "{}: histogram vs blocks",
        c["name"]
    );
    assert_eq!(
        Some(below),
        c["suppressed"].as_u64(),
        "{}: mass below the floor",
        c["name"]
    );
    assert_eq!(
        c["suppressed_blocks"]
            .as_array()
            .map(Vec::len)
            .map(|n| n as u64),
        Some(below),
        "{}: the suppressed blocks are listed",
        c["name"]
    );
}

/// The record's table between the markers, replaced under CE_BLESS=1,
/// compared otherwise.
fn hold_record(root: &Path, doc: &Value) {
    let path = root.join(RECORD);
    let text = std::fs::read_to_string(&path).expect("the calibration record");
    let (head, rest) = text.split_once(BEGIN).expect("distinct:begin marker");
    let (table, tail) = rest.split_once(END).expect("distinct:end marker");
    let want = format!("\n{}", render(doc));
    if crate::facts::blessing() {
        std::fs::write(&path, format!("{head}{BEGIN}{want}{END}{tail}")).expect("write record");
    } else {
        assert_eq!(
            table, want,
            "{RECORD}: the table is not the doc's rendering (CE_BLESS=1)"
        );
    }
}

fn load(root: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(root.join(DOC)).expect("the frozen doc"))
        .expect("parse the frozen doc")
}

/// CI: the tracked fixtures re-measure to the frozen row, every row
/// closes, and the record restates the doc.
#[test]
fn the_fixtures_reproduce_the_calibration_and_the_record_restates_the_doc() {
    let root = common::repo_root();
    let doc = load(&root);
    assert_eq!(doc["schema"], SCHEMA);
    assert_eq!(doc["floor"].as_u64(), Some(DEFAULT_MIN_DISTINCT as u64));
    let corpora = doc["corpora"].as_array().expect("corpora");
    let frozen = corpora
        .iter()
        .find(|c| c["name"] == "fixtures")
        .expect("a fixtures row");
    assert_eq!(
        *frozen,
        measure("fixtures", TRACKED, &root.join(FIXTURES)),
        "the fixtures row drifted — re-run regenerate"
    );
    for c in corpora {
        coherent(c);
    }
    hold_record(&root, &doc);
}

/// By hand: the fixtures and the four pinned corpora, the doc and the
/// record written.
#[test]
#[ignore = "needs the pinned corpora under .ce-eval/corpora; writes the frozen doc"]
fn regenerate() {
    let root = common::repo_root();
    let mut corpora = vec![measure("fixtures", TRACKED, &root.join(FIXTURES))];
    for (name, tip) in PINNED_CORPORA {
        let checkout = crate::eval_support::corpus::pinned_root(name, tip);
        corpora.push(measure(name, &tip[..7], &checkout));
    }
    let doc = json!({
        "schema": SCHEMA,
        "floor": DEFAULT_MIN_DISTINCT,
        "min_tokens": codeeraser::dedup::Params::default().guarantee(),
        "corpora": corpora,
    });
    std::fs::write(
        root.join(DOC),
        format!("{}\n", serde_json::to_string_pretty(&doc).expect("doc")),
    )
    .expect("write the frozen doc");
    hold_record(&root, &doc);
    print!("{}", render(&doc));
}
