//! The reply's `also` table as this side reads it (proto 9.1.0,
//! graph/resolve/mod.rs `also_rows`): rows group by site in order, and a
//! table a core could not have written is refused by name — absent (a
//! core before 9.1.0: reading it as empty would drop the edges it
//! carries), out of order, naming no site, or repeating the site's own
//! answer.

use super::super::also_rows;
use crate::graph::ladder::{Outcome, Reason};
use serde_json::json;

fn outs() -> Vec<Outcome> {
    vec![
        Outcome::Resolved {
            path: "pkg/__init__.py".into(),
            rung: 2,
        },
        Outcome::Unresolved(Reason::OutOfScope),
    ]
}

#[test]
fn also_rows_group_by_site_in_order() {
    let reply = json!({"also": [[0, "pkg/a.py", 2], [0, "pkg/b.py", 2], [1, "ns/m.py", 2]]});
    let got = also_rows(&reply, &outs()).expect("well formed");
    assert_eq!(
        got,
        vec![
            vec![("pkg/a.py".to_string(), 2), ("pkg/b.py".to_string(), 2)],
            vec![("ns/m.py".to_string(), 2)],
        ]
    );
}

#[test]
fn a_table_no_core_writes_is_refused_by_name() {
    let refused = |reply: serde_json::Value, why: &str| {
        let err = also_rows(&reply, &outs()).expect_err(why);
        assert!(err.contains(why), "{err}");
    };
    refused(json!({}), "a core before proto 9.1.0");
    refused(
        json!({"also": [[1, "a.py", 2], [0, "b.py", 2]]}),
        "also 1: not after the row before it",
    );
    refused(json!({"also": [[2, "a.py", 2]]}), "also names site 2");
    refused(
        json!({"also": [[0, "pkg/__init__.py", 2]]}),
        "repeats site 0's answer",
    );
}
