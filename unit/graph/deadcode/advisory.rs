use super::*;
use crate::mention::{AdvisoryName, Names};
use crate::testutil::node;
use serde_json::json;

fn names(cut: bool) -> Unmentioned {
    let mut m = Names::new();
    for (key, syms) in [([0, 3, 0], vec!["a", "b"]), ([1, 7, 0], vec!["c"])] {
        m.insert(
            key,
            syms.into_iter()
                .enumerate()
                .map(|(i, s)| AdvisoryName {
                    symbol: s.into(),
                    line: i as i64 + 1,
                })
                .collect(),
        );
    }
    Unmentioned { names: m, cut }
}

fn refusal(reply: &Value, n: &Unmentioned) -> String {
    let nodes = [node("a.rs", "", 0), node("b.rs", "", 0)];
    consume(reply, &nodes, Some(n))
        .expect_err("a refusal")
        .to_string()
}

/// The faces, and the refusals: the two K38 legs by their own
/// messages (a key the wire never offered; an offered key with no
/// names), and a judged reply with no advisory key at all. The
/// producer's cut rides through to the face unchanged.
#[test]
fn rows_name_back_and_the_mirror_legs_refuse_skew() {
    let nodes = [node("a.rs", "", 0), node("b.rs", "", 0)];
    let n = names(false);
    assert!(consume(&json!({}), &nodes, None).unwrap().is_none());
    assert!(matches!(
        consume(
            &json!({"unmentionedDropped": true, "exportUnmentioned": []}),
            &nodes,
            Some(&n)
        )
        .unwrap(),
        Some(UnmentionedFace::Dropped)
    ));
    let reply = json!({"exportUnmentioned": [[0, 3, 0, 0], [1, 7, 0, 2]]});
    let Some(UnmentionedFace::Rows { rows, cut }) = consume(&reply, &nodes, Some(&n)).unwrap()
    else {
        panic!("rows");
    };
    assert!(!cut);
    let got: Vec<(&str, &str, i64, &str)> = rows
        .iter()
        .map(|r| (r.name.as_str(), r.symbol.as_str(), r.line, r.code))
        .collect();
    assert_eq!(
        got,
        [
            ("a.rs", "a", 1, "public_unmentioned"),
            ("a.rs", "b", 2, "public_unmentioned"),
            ("b.rs", "c", 1, "restricted_unmentioned"),
        ]
    );
    assert!(matches!(
        consume(&reply, &nodes, Some(&names(true))).unwrap(),
        Some(UnmentionedFace::Rows { cut: true, .. })
    ));
    // degraded: no keys at all, an empty face
    assert!(matches!(
        consume(&json!({"degraded": true}), &nodes, Some(&n)).unwrap(),
        Some(UnmentionedFace::Rows { rows, .. }) if rows.is_empty()
    ));
    let unoffered = json!({"exportUnmentioned": [[1, 3, 0, 0]]});
    assert!(refusal(&unoffered, &n).contains("outside the offered table"));
    let mut empty = names(false);
    empty.names.insert([1, 3, 0], Vec::new());
    assert!(refusal(&unoffered, &empty).contains("names no local candidate"));
    assert!(refusal(&json!({"degraded": false, "dead": []}), &n).contains("pre-6.2.0"));
}
