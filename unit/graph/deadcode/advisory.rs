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

/// The rows, and the refusals: the two K38 legs by their own messages
/// (a key the wire never offered; an offered key with no names), and a
/// judged reply with no advisory key at all. One row per name with the
/// core's code as it came (the document names it, plan v2.32 step 4);
/// the producer's cut rides through unchanged.
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
        Some(Advised::Dropped)
    ));
    let reply = json!({"exportUnmentioned": [[0, 3, 0, 0], [1, 7, 0, 2]]});
    let Some(Advised::Rows { rows, cut }) = consume(&reply, &nodes, Some(&n)).unwrap() else {
        panic!("rows");
    };
    assert!(!cut);
    let got: Vec<(i64, &str, i64, i64)> = rows
        .iter()
        .map(|r| (r.node, r.symbol.as_str(), r.line, r.code))
        .collect();
    assert_eq!(got, [(0, "a", 1, 0), (0, "b", 2, 0), (1, "c", 1, 2)]);
    assert!(matches!(
        consume(&reply, &nodes, Some(&names(true))).unwrap(),
        Some(Advised::Rows { cut: true, .. })
    ));
    // degraded: no keys at all, an empty face
    assert!(matches!(
        consume(&json!({"degraded": true}), &nodes, Some(&n)).unwrap(),
        Some(Advised::Rows { rows, .. }) if rows.is_empty()
    ));
    let unoffered = json!({"exportUnmentioned": [[1, 3, 0, 0]]});
    assert!(refusal(&unoffered, &n).contains("outside the offered table"));
    let mut empty = names(false);
    empty.names.insert([1, 3, 0], Vec::new());
    assert!(refusal(&unoffered, &empty).contains("names no local candidate"));
    assert!(refusal(&json!({"degraded": false, "dead": []}), &n).contains("pre-6.2.0"));
}
