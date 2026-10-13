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

/// The plan's rows against erase/1's ceiling (plan v2.33 step 11b item
/// 4): the judgment is one request, so a plan over the cap is refused
/// by name before it is sent, naming the narrowing a user has — never
/// the "cap mirror drift" a degraded reply used to be read as.
#[test]
fn a_plan_over_the_row_cap_is_refused_before_sending() {
    within_cap(4096, 4096).expect("at the cap");
    let why = within_cap(4097, 4096).expect_err("over the cap");
    assert_eq!(
        why.to_string(),
        "erase: 4097 candidates over the core's erase_rows cap 4096; narrow the plan with \
         `ce erase <subdirectory>` or an `exclude` in ce.toml"
    );
}
