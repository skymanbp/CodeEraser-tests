use super::*;
use serde_json::json;

fn counts(answers: u64) -> Value {
    json!({"rules": 8, "queries": 1, "asserts": 0, "strata": 3, "facts": 4, "derived": 6,
           "answers": answers, "violations": 0, "proofNodes": 2, "proofTruncated": 0})
}

#[test]
fn the_body_carries_the_texts_the_tables_by_decimal_code_and_the_two_flags() {
    let facts = BTreeMap::from([
        (1u32, vec![vec![3u64], vec![5]]),
        (9, vec![vec![3, 5, 7, 1]]),
    ]);
    assert_eq!(
        body(&json!(["p.", null, "?- p."]), (&facts, None), true, false),
        json!({
            "texts": ["p.", null, "?- p."],
            "facts": {"1": [[3], [5]], "9": [[3, 5, 7, 1]]},
            "why": true, "schema": false,
        })
    );
    let tree = json!({"paths": ["a/x.py", "b"], "packages": [1]});
    assert_eq!(
        body(&json!([]), (&BTreeMap::new(), Some(&tree)), false, false)["tree"],
        tree,
        "the tree form rides as sent"
    );
}

#[test]
fn consume_types_a_healthy_reply_and_keeps_the_cores_order() {
    let reply = json!({
        "goals": [[0, 0, 0], [1, 1, 0, 3]],
        "preds": [[1002, 0], [1006, 0, 3]],
        "answers": [[0, 5], [0, 3], [1, 3, -2]],
        "proof": [[0, 0, 0, -1, -1, -1, 5], [0, 0, 1, 0, 3, 1002, 5], [0, 0, 2, 1, -1, 1, 5]],
        "errors": [],
        "counts": counts(3), "degraded": false,
        "schema": [[0, 2, 0, 4], [1, 1, 0]],
    });
    let j = consume(&reply, 40).unwrap();
    assert_eq!(j.goals, [(0, vec![0]), (1, vec![0, 3])]);
    assert_eq!(
        j.preds,
        BTreeMap::from([(1002, vec![0]), (1006, vec![0, 3])])
    );
    assert_eq!(j.answers, [(0, vec![5]), (0, vec![3]), (1, vec![3, -2])]);
    assert_eq!(j.proof.len(), 3);
    assert_eq!(
        j.proof[1],
        ProofRow {
            goal: 0,
            answer: 0,
            node: 1,
            parent: 0,
            rule: 3,
            pred: 1002,
            args: vec![5]
        }
    );
    assert_eq!(j.proof[2].rule, -1);
    assert_eq!(j.counts["answers"], 3);
    assert_eq!(j.schema, Some(vec![vec![0, 2, 0, 4], vec![1, 1, 0]]));
    assert_eq!(j.degraded, None);
}

#[test]
fn consume_names_the_degraded_reply_and_every_skew() {
    let degraded = json!({"degraded": true, "reason": "query_too_large", "counts": counts(0)});
    assert_eq!(
        consume(&degraded, 40).unwrap().degraded.as_deref(),
        Some("query_too_large")
    );
    let skew = |patch: &dyn Fn(&mut Value)| {
        let mut r = json!({"goals": [[0, 0, 0]], "preds": [], "answers": [[0, 5]], "proof": [],
                           "errors": [], "counts": counts(1), "degraded": false});
        patch(&mut r);
        consume(&r, 40).map(|_| ()).unwrap_err()
    };
    // key | the value that skews it | the words the refusal must carry
    const SKEWS: &str = "\
answers | [[1, 5]] | goal that was not sent
answers | [[0, 5, 6]] | arity
errors | [[41, 1]] | token <= sent
errors | [[3, 10]] | code 1..9
goals | [[1, 0, 0]] | goal in order
preds | [[26, 5, 0]] | code >= 1000
proof | [[0, 0, 1]] | proof row
counts | {} | counts.rules missing
goals | null | goals:";
    for row in SKEWS.lines() {
        let cols: Vec<&str> = row.split(" | ").collect();
        let bad: Value = serde_json::from_str(cols[1]).unwrap();
        let why = skew(&|r| r[cols[0]] = bad.clone());
        assert!(why.contains(cols[2]), "{row}: {why}");
    }
    // an error at the index past the last token is the program's end, allowed
    let mut r = json!({"goals": [], "preds": [], "answers": [], "proof": [], "errors": [[40, 1]],
                       "counts": counts(0), "degraded": false});
    assert_eq!(consume(&r, 40).unwrap().errors, [(40, 1)]);
    r["errors"] = json!([[40, 9]]);
    assert_eq!(consume(&r, 40).unwrap().errors, [(40, 9)]);
}
