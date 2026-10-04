use super::*;

/// Sorted-rank locals: sequences are deduplicated across pairs and
/// rows reference them by rank; no run rides a row (the core measures
/// it and answers it beside the score). The generic pin battery lives
/// with corelink; this family owns the D13 shingle-width pin, exercised
/// through its parse_result.
#[test]
fn chunk_request_dedups_seqs_and_shingle_width_pins() {
    let seqs: Vec<Vec<u64>> = vec![vec![1, 2], vec![3, 4, 3], vec![5, 6]];
    let (order, body) = chunk_request(&[(0, 2), (1, 2)], |g| &seqs[g]);
    assert_eq!(order, vec![0, 1, 2]);
    assert_eq!(body["seqs"], json!([[1, 2], [3, 4, 3], [5, 6]]));
    assert_eq!(body["pairs"], json!([[0, 2], [1, 2]]));
    assert!(body.get("sets").is_none(), "one shape per request");
    let ok = json!({"degraded": false, "runs": [7], "verdicts": [false],
            "scores": [[0, 1, 2, 4]], "counts": {"judged": 1, "jaccardDups": 0},
            "knobs": {"jaccardNum": 80, "jaccardDen": 100, "shingleK": 5,
                "verbatimFloor": 50, "minDocTokens": 50, "docLineCap": 200,
                "licHeadLines": 5}});
    let (rows, _) = parse_result(&ok).expect("well-formed");
    assert_eq!(rows, vec![(0, 1, (2, 4, 7, false))]);
    // each breakage refused by its own name: the D13 alphabet-geometry
    // pin, then a run column one row short
    let broken = |edit: &dyn Fn(&mut Value)| {
        let mut reply = ok.clone();
        edit(&mut reply);
        parse_result(&reply).expect_err("refused").to_string()
    };
    let drift = broken(&|r| r["knobs"]["shingleK"] = json!(4));
    assert!(drift.contains("shingleK"), "{drift}");
    let short = broken(&|r| r["runs"] = json!([]));
    assert!(short.contains("0 runs for 1 score rows"), "{short}");
}
