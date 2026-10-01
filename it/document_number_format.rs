//! The core writes the console's numbers (plan v2.32 step 5; design
//! booklet docs/reference/authority-track.md §6): a split candidate's
//! ROI is benefit over cost at one decimal, which the structure face
//! formatted with Rust's `{:.1}` (`cli/src/structure/report.rs`,
//! `let roi = |b: i64, c: i64| format!("{:.1}x", b as f64 / c as f64);`).
//! The document golden's number battery (request id 401: seven
//! candidates over a cost of 1000, the halfway cases among them) holds
//! the core's text; this leg formats the same seven pairs Rust's way
//! and finds each in its line, in order.
use crate::fixture_contract::golden_pairs;
use serde_json::Value;

#[test]
fn the_core_writes_the_roi_as_rust_formats_it() {
    let battery = golden_pairs("document/golden.ndjson")
        .into_iter()
        .map(|(q, a)| (json(&q), json(&a)))
        .find(|(q, _)| q["id"] == 401)
        .expect("the document golden's number battery (id 401)");
    let (request, reply) = battery;
    let rows = request["rows"]["splitCandidates"].as_array().expect("rows");
    let texts: Vec<&str> = reply["lines"]
        .as_array()
        .expect("lines")
        .iter()
        .filter_map(|l| l[1].as_str())
        .filter(|t| t.starts_with("split "))
        .collect();
    assert_eq!(rows.len(), 7, "the battery's seven candidates");
    assert_eq!(texts.len(), rows.len(), "one split line per candidate");
    for (row, text) in rows.iter().zip(&texts) {
        let (b, c) = (row[2].as_i64().unwrap(), row[3].as_i64().unwrap());
        let roi = format!("{:.1}x", b as f64 / c as f64);
        let tail = format!("— ROI {roi} (recover {b} vs cost {c})");
        assert!(
            text.ends_with(&tail),
            "{b}/{c}: {text:?} does not end {tail:?}"
        );
    }
}

fn json(line: &str) -> Value {
    serde_json::from_str(line).expect("golden line")
}
