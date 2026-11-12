//! BARS.md §4 and the review table builder on a synthetic fixture (lane
//! v235_licence, phase 2A (b)): the verdicts under K1 / K1-strict and
//! populations A / B, the per-project floor, the class floor, the
//! weights, the named-only note, the owner questions; the builder
//! blocking until every row is settled. Synthetic rows only.

use crate::eval_deadfile_bars::{answer, tip};
use crate::eval_deadfile_parts::audit::bars::{Row, evaluate, render};
use crate::eval_deadfile_parts::audit::page_bars::outcome_sentence;
use crate::eval_deadfile_parts::audit::review::clone_facts;
use crate::eval_deadfile_parts::audit::tables::{ByRow, bars_rows, review_doc};
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Rows from `project k1-class strict-class rows in-use; …`: in-use
/// rows first, every other one `named`, weight 1.
fn population(spec: &str) -> Vec<Row> {
    let mut out = Vec::new();
    for part in spec.split(';').map(str::trim).filter(|p| !p.is_empty()) {
        let w: Vec<&str> = part.split_whitespace().collect();
        let (n, hits): (usize, usize) = (w[3].parse().expect("n"), w[4].parse().expect("hits"));
        out.extend((0..n).map(|i| Row {
            corpus: w[0].into(),
            path: format!("{}/{}/{i}", w[1], w[2]),
            lang: "python".into(),
            k1: w[1].into(),
            k1_strict: w[2].into(),
            weight: 1.0,
            answer: if i < hits { "in_use" } else { "unused" }.into(),
            named: i % 2 == 0,
        }));
    }
    out
}

fn every(rows: &[Row]) -> BTreeSet<(String, String)> {
    rows.iter()
        .map(|r| (r.corpus.clone(), r.path.clone()))
        .collect()
}

/// The `all` judgment of one class under one reading.
fn class<'a>(report: &'a Value, reading: &str, c: &str) -> &'a Value {
    &report["readings"][reading]["classes"][c]["all"]
}

const SPLIT: &str = "p1 runs-alone runs-alone 20 20; p2 runs-alone runs-alone 20 18; p1 declares declares 20 10; p2 declares declares 20 10";
const STRICT_DIFFERS: &str = "p1 runs-alone runs-alone 20 20; p2 runs-alone runs-alone 20 18; p1 declares runs-alone 20 10; p2 declares runs-alone 20 10";

#[test]
fn the_split_ships_only_when_every_reading_agrees() {
    let rows = population(SPLIT);
    let report = evaluate(&rows, &BTreeSet::new(), Some(&every(&rows)));
    assert_eq!(
        report["code"],
        json!({"outcome": "the split ships: runs-alone advisory, declares keeps failing", "agreed": true})
    );
    assert_eq!(class(&report, "k1/A", "document")["verdict"], "not judged");
    let k1a = &report["readings"]["k1/A"];
    assert!(
        k1a["questions"]
            .to_string()
            .contains("declares: FAILING with in = 0.500"),
        "{}",
        k1a["questions"]
    );
    assert!(
        k1a["named_notes"].to_string().contains("runs-alone"),
        "half the in_use answers are named"
    );
    assert!(render(&report).contains("leave one out"));
    let rows = population(STRICT_DIFFERS);
    let report = evaluate(&rows, &BTreeSet::new(), Some(&every(&rows)));
    assert_eq!(report["code"]["agreed"], false);
    assert_eq!(
        report["code"]["by_reading"]["k1_strict/A"],
        "nothing ships: runs-alone failing"
    );
    let rows = population(SPLIT);
    assert_eq!(
        evaluate(&rows, &BTreeSet::new(), None)["code"]["outcome"],
        "owner question: population B not computed"
    );
}

/// The registry page's outcome sentence opens "ships no split": with
/// every reading agreeing that the split ships it has no words and
/// refuses; with the readings differing it renders.
#[test]
fn the_outcome_sentence_refuses_an_agreed_split() {
    let sentence = |spec: &str| {
        let rows = population(spec);
        let report = evaluate(&rows, &BTreeSet::new(), Some(&every(&rows)));
        outcome_sentence(&report, &["setup.py"])
    };
    let refused = sentence(SPLIT).expect_err("an agreed split has no sentence");
    assert!(refused.contains("the split ships"), "{refused}");
    let said = sentence(STRICT_DIFFERS).expect("the readings differ");
    assert!(
        said.contains("the code outcome is an owner question"),
        "{said}"
    );
}

#[test]
fn floors_weights_and_population_a() {
    let rows = population("p1 runs-alone runs-alone 30 30; p2 runs-alone runs-alone 10 5");
    let got = class(
        &evaluate(&rows, &BTreeSet::new(), Some(&every(&rows))),
        "k1/A",
        "runs-alone",
    )
    .clone();
    assert_eq!(
        (got["verdict"].as_str(), got["in"].as_f64()),
        (Some("FAILING"), Some(0.875)),
        "a project under 0.60"
    );
    let rows = population("p1 runs-alone runs-alone 29 29");
    assert_eq!(
        class(
            &evaluate(&rows, &BTreeSet::new(), None),
            "k1/A",
            "runs-alone"
        )["verdict"],
        "not judged"
    );
    let mut rows = population("p1 declares declares 2 1");
    rows[1].weight = 3.0;
    assert_eq!(
        class(&evaluate(&rows, &BTreeSet::new(), None), "k1/A", "declares")["in"].as_f64(),
        Some(0.25)
    );
    let gone = every(&rows[..1]);
    assert_eq!(
        class(&evaluate(&rows, &gone, None), "k1/A", "declares")["rows"],
        1,
        "a passing convention's rows leave A"
    );
}

fn class_doc() -> Value {
    let r = |path: &str, sample: &str, k1: &str| json!({"corpus": "p1", "commit": "c0", "path": path, "sample": sample, "k1": k1, "k1_strict": k1});
    json!({
        "lang": "python",
        "per_corpus": {"p1": {"pool": 4, "primary": 2}},
        "rows": [r("tool.py", "primary", "runs-alone"), r("lib.py", "primary", "declares"), r("spare.py", "reserve", "declares")],
    })
}

#[test]
fn the_builder_blocks_until_every_row_is_settled() {
    let key = |p: &str| ("p1".to_string(), p.to_string());
    let used = answer("in_use ~ Makefile:3 ~ make ~ ");
    let first: ByRow = [
        (key("tool.py"), used.clone()),
        (
            key("lib.py"),
            answer("unused ~  ~  ~ git grep -w lib over the tree and CI"),
        ),
    ]
    .into();
    let second: ByRow = [(key("lib.py"), used)].into();
    let doc = review_doc(
        &clone_facts(&tip),
        &class_doc(),
        &first,
        &second,
        &ByRow::new(),
    );
    assert_eq!(
        doc["rows"].as_array().map(Vec::len),
        Some(2),
        "the unanswered reserve row stays out"
    );
    let blocked = bars_rows(&[doc], &[class_doc()]).expect_err("lib.py is unsettled");
    assert_eq!(blocked, ["p1 lib.py: unsettled: two different answers"]);
    let settled: ByRow = [(key("lib.py"), json!({"answer": "unsure"}))].into();
    let doc = review_doc(&clone_facts(&tip), &class_doc(), &first, &second, &settled);
    let got = bars_rows(&[doc], &[class_doc()]).expect("settled");
    let read: Vec<String> = got
        .iter()
        .map(|r| format!("{} {} {} {}", r.path, r.answer, r.weight, r.named))
        .collect();
    assert_eq!(read, ["tool.py in_use 2 true", "lib.py unsure 2 false"]);
}
