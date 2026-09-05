//! The metric arithmetic the oracle gate re-derives (split from
//! eval_similar_precision.rs at the file budget): the block the merge
//! script wrote for every generation — p@1 per arm over all answered
//! queries, over the role-bit-1 / role-bit-0 / non-clone top-1s, hit@5
//! per arm, the role bit's confusion over every arbitrated candidate.
//! The gate asserts each frozen summary equals this over its rows.

use serde_json::Value;

/// The candidate at an arm's rank 1, if the arm answered.
fn top1<'r>(r: &'r Value, arm: &str) -> Option<&'r Value> {
    r["candidates"]
        .as_array()
        .expect("candidates")
        .iter()
        .find(|c| c[arm]["rank"] == 1)
}

fn frac(rows: &[&Value], hit: impl Fn(&Value) -> bool) -> Value {
    serde_json::json!([rows.iter().filter(|c| hit(c)).count(), rows.len()])
}

/// The metric block the merge script wrote, re-derived here: p@1 per
/// arm over all answered queries, over the role-bit-1 and role-bit-0
/// top-1s, over non-clone top-1s; hit@5 per arm; the role bit's
/// confusion over every arbitrated candidate.
pub fn metrics(rows: &[Value]) -> Value {
    let same = |c: &Value| c["truth"] == "same_role";
    let mut m = serde_json::Map::new();
    m.insert("queries".into(), rows.len().into());
    let all: Vec<&Value> = rows
        .iter()
        .flat_map(|r| r["candidates"].as_array().expect("candidates"))
        .collect();
    m.insert("candidates".into(), all.len().into());
    for arm in ["bare", "widened"] {
        arm_metrics(rows, arm, &mut m);
    }
    let cell = |role: bool, truth: bool| {
        all.iter()
            .filter(|c| (c["role"] == role) && (same(c) == truth))
            .count()
    };
    m.insert(
        "role_bit_over_candidates".into(),
        serde_json::json!({"tp": cell(true, true), "fp": cell(true, false), "fn": cell(false, true), "tn": cell(false, false)}),
    );
    Value::Object(m)
}

/// One arm's block: p@1 over all answered queries and over the
/// role-bit-1, role-bit-0 and non-clone top-1s; hit@5.
fn arm_metrics(rows: &[Value], arm: &str, m: &mut serde_json::Map<String, Value>) {
    let same = |c: &Value| c["truth"] == "same_role";
    let answered: Vec<&Value> = rows.iter().filter_map(|r| top1(r, arm)).collect();
    let subset = |field: &str, want: bool| -> Vec<&Value> {
        answered
            .iter()
            .copied()
            .filter(|c| c[field] == want)
            .collect()
    };
    m.insert(format!("p_at_1_{arm}"), frac(&answered, same));
    m.insert(
        format!("p_at_1_{arm}_nonclone"),
        frac(&subset("clone", false), same),
    );
    m.insert(
        format!("p_at_1_{arm}_role1"),
        frac(&subset("role", true), same),
    );
    m.insert(
        format!("p_at_1_{arm}_role0"),
        frac(&subset("role", false), same),
    );
    let hit5 = rows
        .iter()
        .filter(|r| {
            r["candidates"]
                .as_array()
                .expect("candidates")
                .iter()
                .any(|c| !c[arm].is_null() && same(c))
        })
        .count();
    m.insert(
        format!("hit_at_5_{arm}"),
        serde_json::json!([hit5, rows.len()]),
    );
}
