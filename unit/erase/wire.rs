use super::*;

fn cand(path: &str, span: Option<(i64, i64)>) -> Candidate {
    Candidate {
        class: 3,
        facts: [1, 2, 0, 0],
        path: path.to_string(),
        span,
        provenance: String::new(),
        sites: 0,
    }
}

/// The target table the request carries: path ids dense in order of
/// first appearance, 0/0 for a whole file, the span's own lines
/// otherwise — three candidates on two paths, hand-checked.
#[test]
fn targets_are_dense_path_ids_with_the_span_or_zeros() {
    let cands = [
        cand("a.md", None),
        cand("a.md", Some((4, 9))),
        cand("b.py", None),
    ];
    assert_eq!(targets_of(&cands), vec![[0, 0, 0], [0, 4, 9], [1, 0, 0]]);
}

/// The reply decode: both tables length-locked and read in step; a
/// reply without `kept` is skew named by its key (corelink::skew),
/// never read as "every row stands".
#[test]
fn decode_locks_both_tables_and_names_a_closure_less_core() {
    let ok = serde_json::json!({ "rows": [[1, 0], [0, 6]], "kept": [1, 0] });
    let got: Vec<(bool, i64, bool)> = decode(&ok, 2)
        .expect("decoded")
        .iter()
        .map(|v| (v.eraseable, v.reason, v.kept))
        .collect();
    assert_eq!(got, vec![(true, 0, true), (false, 6, false)]);
    let old = serde_json::json!({ "rows": [[1, 0], [0, 6]] });
    let why = format!("{:#}", decode(&old, 2).expect_err("no kept"));
    assert!(why.contains("core offers no erase/1 `kept`"), "{why}");
    let short = serde_json::json!({ "rows": [[1, 0], [0, 6]], "kept": [1] });
    assert!(
        decode(&short, 2).is_err(),
        "a short kept table cannot default"
    );
}

/// The request split (plan v2.35 T5): greedy runs of whole paths, each
/// at most `cap` rows — a path's rows never straddle a request, a run
/// that would pass the cap opens the next request, and one path over
/// the cap alone is refused by name before anything is sent.
#[test]
fn requests_split_between_paths_and_never_inside_one() {
    let cands = [
        cand("a.md", None),
        cand("a.md", Some((4, 9))),
        cand("b.py", None),
        cand("c.py", None),
        cand("c.py", Some((1, 2))),
        cand("d.py", None),
    ];
    assert_eq!(chunks(&cands, 6).expect("one"), vec![0..6]);
    assert_eq!(chunks(&cands, 3).expect("two"), vec![0..3, 3..6]);
    assert_eq!(
        chunks(&cands, 2).expect("four"),
        vec![0..2, 2..3, 3..5, 5..6]
    );
    let why = chunks(&cands, 1).expect_err("a.md alone is over");
    assert_eq!(
        why.to_string(),
        "erase: a.md alone has 2 candidates over the core's erase_rows cap 1; a path's rows \
         are judged in one request — narrow the plan with an `exclude` in ce.toml"
    );
    assert_eq!(targets_of(&cands[3..5]), vec![[0, 0, 0], [0, 1, 2]]);
}
