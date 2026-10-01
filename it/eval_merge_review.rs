//! The clone-merge family's audit (plan v2.31 step 7, step-7 ruling 2):
//! the sample is the hash-ranked draw of the frozen set, row for row;
//! the audit is blind to the verdict — feasibility, reason, savings and
//! the member kept are withheld — and sees the core's parameterisation,
//! which it may reject. The audit table (`merge-review-v1.json`, filled
//! by independent agents who read the sample alone) answers each
//! sampled row with `{id, feasible, reason, params, note}`, `reason` one
//! of the six names; the precision doc reads, per corpus and overall,
//! how often the core's feasibility and reason agree with the audit's,
//! and `params_agree` — how often the count the audit gives after
//! seeing the core's parameterisation equals the core's. The audit and
//! its precision doc land after the step's code — each gate reads a doc
//! only once it is filed, and says so while it is not.

use crate::eval_merge_parts::sample::{SAMPLE, draw, id_of};
use crate::eval_merge_parts::{DOC, load, write};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const REVIEW: &str = "contracts/eval/merge-review-v1.json";
const PRECISION: &str = "contracts/eval/merge-precision-v1.json";
const PRECISION_SCHEMA: &str = "ce.eval-merge-precision/1.0.0";
/// The six reason names an audited row answers (merge/face.rs).
const REASONS: [&str; 6] = codeeraser::merge::face::REASONS;

fn rows(doc: &Value) -> &Vec<Value> {
    doc["rows"].as_array().expect("rows")
}

/// The sample holds exactly the draw of the frozen set, in its order.
#[test]
fn the_sample_is_the_draw_of_the_frozen_set() {
    let (Some(set), Some(sample)) = (load(DOC), load(SAMPLE)) else {
        panic!("{DOC} and {SAMPLE} are filed together");
    };
    let drawn: Vec<String> = draw(&set)
        .iter()
        .map(|(c, r)| id_of(c, &r["members"]))
        .collect();
    let held: Vec<&str> = rows(&sample)
        .iter()
        .map(|r| r["id"].as_str().expect("id"))
        .collect();
    assert_eq!(held, drawn);
    let verdicts = ["feasible", "reason", "params", "savings"];
    for r in rows(&sample) {
        assert!(
            verdicts.iter().all(|k| r.get(*k).is_none()),
            "a sampled row shows no verdict"
        );
    }
}

/// The readings: per corpus and overall, the feasibility, reason and
/// parameter-count agreements over the rows audited.
fn readings(set: &Value, sample: &Value, review: &Value) -> Value {
    let core: BTreeMap<String, &Value> = set["corpora"]
        .as_array()
        .expect("corpora")
        .iter()
        .flat_map(|c| {
            rows(c)
                .iter()
                .map(move |r| (id_of(c["name"].as_str().expect("name"), &r["members"]), r))
        })
        .collect();
    let mut by: BTreeMap<String, [u64; 4]> = BTreeMap::new();
    for (s, a) in rows(sample).iter().zip(rows(review)) {
        assert_eq!(s["id"], a["id"], "the audit answers the sample in order");
        let c = core[s["id"].as_str().expect("id")];
        let feasible = a["feasible"].as_bool().expect("an audited feasibility");
        let reason = a["reason"].as_str().expect("an audited reason");
        assert!(
            REASONS.contains(&reason),
            "a reason is one of the six: {reason}"
        );
        let params = a["params"].as_u64().expect("an audited parameter count");
        for key in [s["corpus"].as_str().expect("corpus"), "all"] {
            let t = by.entry(key.to_string()).or_default();
            t[0] += u64::from(c["feasible"] == feasible);
            t[1] += u64::from(c["reason"] == reason);
            t[2] += u64::from(c["params"] == params);
            t[3] += 1;
        }
    }
    let shown: serde_json::Map<String, Value> = by
        .into_iter()
        .map(|(k, [f, r, p, n])| {
            let read =
                json!({"feasible_agree": f, "reason_agree": r, "params_agree": p, "rows": n});
            (k, read)
        })
        .collect();
    json!({"schema": PRECISION_SCHEMA, "from": [DOC, SAMPLE, REVIEW], "readings": shown})
}

/// The audit, once filed, answers every sampled row; the precision doc,
/// once filed, is what the three docs read.
#[test]
fn the_review_scores_when_filed() {
    let Some(review) = load(REVIEW) else {
        println!("eval_merge_review: {REVIEW} not filed yet — skipped");
        return;
    };
    let (set, sample) = (load(DOC).expect("set"), load(SAMPLE).expect("sample"));
    assert_eq!(
        rows(&review).len(),
        rows(&sample).len(),
        "one audited row per sampled row"
    );
    let read = readings(&set, &sample, &review);
    match load(PRECISION) {
        Some(doc) => assert_eq!(doc, read, "{PRECISION} reads what the docs read"),
        None => println!("eval_merge_review: {PRECISION} not filed yet — skipped"),
    }
}

#[test]
#[ignore]
fn regenerate() {
    assert!(crate::facts::blessing(), "CE_BLESS=1 regenerates");
    let review = load(REVIEW).unwrap_or_else(|| panic!("{REVIEW}: the audit comes first"));
    let read = readings(
        &load(DOC).expect("set"),
        &load(SAMPLE).expect("sample"),
        &review,
    );
    println!("{}", read["readings"]);
    write(PRECISION, &read);
}
