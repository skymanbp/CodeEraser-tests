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
/// the strings stay here — a node's path, a section's `path#unit`
/// label, an advisory name — and nothing outside the tables resolves.
#[test]
fn the_request_is_the_judgment_and_the_strings_stay_here() {
    let (w, j) = fixture("graph_too_large");
    let (req, names) = request(("deadcode", true), &w, &j).expect("request");
    let body = req.body();
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
    let said = |class: &str, i: i128| names.resolve(class, &[i]);
    assert_eq!(said("path", 2).as_deref(), Some("a.rs"));
    assert_eq!(said("node_name", 2).as_deref(), Some("a.rs#Intro"));
    assert_eq!(said("node_name", 3).as_deref(), Some("pkg"));
    assert_eq!(said("symbol", 0).as_deref(), Some("never_spelled"));
    assert_eq!(said("path", 4), None);
    assert_eq!(said("symbol", 1), None);
    assert_eq!(said("site_spec", 0), None);
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
