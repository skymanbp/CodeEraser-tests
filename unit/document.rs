// The binder's legs (cli/src/document.rs): a reference is an object
// whose one key is `$`, resolved by class and integers; anything else
// passes through; an unknown class, a range miss or a malformed
// reference is an error naming it.
use super::*;

struct Two;

impl Resolve for Two {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        let names = ["a.rs".to_string(), "b.rs".to_string()];
        match class {
            "path" => at(&names, ints),
            "big" => Some(
                ints.iter()
                    .map(i128::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            _ => None,
        }
    }
}

#[test]
fn references_bind_and_everything_else_passes_through() {
    let skeleton = json!({
        "schema": "x/0.1.0", "n": 3, "flag": true, "none": null,
        "rows": [{"path": {"$": ["path", 1]}, "keep": {"$": "not a reference", "other": 1}}],
        "big": {"$": ["big", 18446744073709551615u64, -1]},
    });
    let bound = bind(skeleton, &Two).expect("bound");
    assert_eq!(
        bound,
        json!({
            "schema": "x/0.1.0", "n": 3, "flag": true, "none": null,
            "rows": [{"path": "b.rs", "keep": {"$": "not a reference", "other": 1}}],
            "big": "18446744073709551615,-1",
        })
    );
}

#[test]
fn an_unresolvable_reference_is_named() {
    let cases = [
        (json!({"$": ["path", 2]}), "no string for"),
        (json!({"$": ["dir", 0]}), "no string for"),
        (json!({"$": []}), "not [class, integers"),
        (json!({"$": [7]}), "not [class, integers"),
        (json!({"$": ["path", "1"]}), "non-integer"),
    ];
    for (skeleton, said) in cases {
        let err = bind(skeleton.clone(), &Two)
            .expect_err("refused")
            .to_string();
        assert!(err.contains(said), "{skeleton}: {err}");
    }
}

#[test]
fn ranks_are_the_joint_string_order() {
    assert_eq!(ranks(["b", "a/", "a", "c"]), vec![2, 1, 0, 3]);
    assert_eq!(ranks(["x", "x"]), vec![0, 0]);
}

#[test]
fn a_request_pads_its_absent_tables_and_facts() {
    let body = Request::new("arch")
        .rows("files", [[0, 0, 1]])
        .empty(&["files", "dirs"])
        .fact("units", 2)
        .zero(&["units", "vars"])
        .range("why", 1)
        .degraded(0)
        .body();
    assert_eq!(
        body,
        json!({
            "family": "arch", "ranges": {"why": 1},
            "rows": {"files": [[0, 0, 1]], "dirs": []},
            "facts": {"units": 2, "vars": 0}, "degraded": 0,
            "lang": lines::lang(),
        })
    );
}
