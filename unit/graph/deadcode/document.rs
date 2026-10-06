use super::super::advisory::{Advised, Named};
use super::super::{GraphWire, Judged};
use super::*;
use serde_json::json;

/// Two files, a section of a.rs and a package; b.rs judged dead, the
/// section reported, one advisory name on a.rs, the graph degraded.
fn fixture(reason: &str) -> (GraphWire, Judged) {
    let w = crate::testutil::four_node_wire(&[]);
    let j = Judged {
        dead: vec![vec![1, 1, 2]],
        reported: vec![[2, 3]],
        kept: Some(5),
        degraded: Some(reason.into()),
        fail: true,
        advisory: Some(Advised::Rows {
            rows: vec![Named {
                node: 0,
                symbol: "never_spelled".into(),
                line: 3,
                code: 1,
            }],
            cut: true,
        }),
    };
    (w, j)
}

/// The request carries the judgment's rows as they came, one advisory
/// row per name numbered in order, the three advisory bits, the kept
/// count and the degraded reason by its code in the package's list;
/// the file-tier count and the console's `--check` as facts (plan
/// v2.32 step 5: only the lines read them);
/// the strings ride along — each node's path and unit (the core
/// spells a section's `path#unit` label), each advisory name (plan
/// v2.33 W7).
#[test]
fn the_request_is_the_judgment_and_the_strings_ride_along() {
    let (w, j) = fixture("graph_too_large");
    let body = request(("deadcode", true), &w, &j).expect("request").body();
    let reasons = crate::tables::get().document.deadcode.reasons;
    let code = reasons.iter().position(|r| *r == "graph_too_large");
    assert_eq!(body["rows"]["reason"], json!([[code.expect("listed")]]));
    assert_eq!(body["rows"]["dead"], json!([[1, 1, 2]]));
    assert_eq!(body["rows"]["reported"], json!([[2, 3]]));
    assert_eq!(body["rows"]["kept"], json!([[5]]));
    assert_eq!(body["rows"]["unmentioned"], json!([[0, 0, 3, 1]]));
    assert_eq!(
        (&body["ranges"]["nodes"], &body["ranges"]["advisory"]),
        (&json!(4), &json!(1))
    );
    assert_eq!(
        body["facts"],
        json!({
            "unresolvedSites": 7, "asked": 1, "dropped": 0, "cut": 1,
            "files": 2, "check": 1
        })
    );
    let strings = &body["strings"];
    assert_eq!(strings["path"][2], "a.rs");
    assert_eq!(strings["node_unit"][2], "Intro");
    assert_eq!(strings["path"][3], "pkg");
    assert_eq!(strings["node_unit"][3], "");
    assert_eq!(strings["symbol"], json!(["never_spelled"]));
    assert_eq!(strings["why"], json!([]));
}

/// A degraded reply whose reason the package does not list is named,
/// never sent as some other code.
#[test]
fn an_unlisted_reason_is_refused_by_name() {
    let (w, j) = fixture("no_such_reason");
    let err = request(("deadcode", false), &w, &j)
        .err()
        .expect("an unlisted reason")
        .to_string();
    assert!(err.contains("no_such_reason"), "{err}");
}
