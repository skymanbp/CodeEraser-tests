use super::*;

/// Sorted-rank locals: sets are deduplicated across pairs and
/// rows reference them by rank; the run rides each row verbatim.
/// The generic pin battery lives with corelink; this family owns
/// the D13 shingle-width pin, exercised through its parse_result.
#[test]
fn chunk_request_dedups_sets_and_shingle_width_pins() {
    let sets: Vec<Vec<u64>> = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
    let (order, body) = chunk_request(&[(2, 0, 7), (2, 1, 0)], |g| &sets[g]);
    assert_eq!(order, vec![0, 1, 2]);
    assert_eq!(body["sets"], json!([[1, 2], [3, 4], [5, 6]]));
    assert_eq!(body["pairs"], json!([[2, 0, 7], [2, 1, 0]]));
    let ok = json!({"scores": [[0, 1, 2, 4]], "verdicts": [false],
            "counts": {"judged": 1, "jaccardDups": 0},
            "knobs": {"jaccardNum": 80, "jaccardDen": 100, "shingleK": 5,
                "verbatimFloor": 50, "minDocTokens": 50, "docLineCap": 200,
                "licHeadLines": 5},
            "degraded": false});
    assert_eq!(
        parse_result(&ok).expect("well-formed").0,
        vec![(0, 1, (2, 4, false))]
    );
    let mut drifted = ok;
    drifted["knobs"]["shingleK"] = json!(4);
    assert!(
        parse_result(&drifted).is_err(),
        "alphabet-geometry drift refuses (D13)"
    );
}
