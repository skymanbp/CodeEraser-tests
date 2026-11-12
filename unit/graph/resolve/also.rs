//! The reply's `also` table as this side reads it (proto 9.1.0,
//! graph/resolve/mod.rs `also_rows`): rows group by site in order, and a
//! table a core could not have written is refused by name — absent (a
//! core before 9.1.0: reading it as empty would drop the edges it
//! carries), out of order, naming no site, or repeating the site's own
//! answer. The `opaque` table beside it (lane v235_graphtrust) marks its
//! sites, and is refused by name when absent, out of order or naming no
//! site.

use super::super::also_rows;
use crate::graph::ladder::{Also, Outcome, Reason};
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
    let reply =
        json!({"also": [[0, "pkg/a.py", 2], [0, "pkg/b.py", 2], [1, "ns/m.py", 2]], "opaque": [1]});
    let got = also_rows(&reply, &outs()).expect("well formed");
    let also = |rows: &[(&str, u8)], opaque| Also {
        rows: rows.iter().map(|(t, r)| (t.to_string(), *r)).collect(),
        opaque,
    };
    assert_eq!(
        got,
        vec![
            also(&[("pkg/a.py", 2), ("pkg/b.py", 2)], false),
            also(&[("ns/m.py", 2)], true),
        ]
    );
}

/// Part 4C: a section answer's holder row names the answer's own file —
/// no repeat of the answer (the section is the answer), so it reads.
#[test]
fn a_section_answers_holder_row_is_no_repeat_of_its_answer() {
    let section = Outcome::ResolvedSection {
        path: "c.md".into(),
        slug: Some("sec".into()),
        rung: 2,
    };
    let reply = json!({"also": [[0, "c.md", 2]], "opaque": []});
    let got = also_rows(&reply, &[section]).expect("a holder row reads");
    assert_eq!(got[0].rows, [("c.md".to_string(), 2)]);
}

#[test]
fn a_table_no_core_writes_is_refused_by_name() {
    let refused = |reply: serde_json::Value, why: &str| {
        let err = also_rows(&reply, &outs()).expect_err(why);
        assert!(err.contains(why), "{err}");
    };
    refused(json!({}), "a core before proto 9.1.0");
    refused(
        json!({"also": [[1, "a.py", 2], [0, "b.py", 2]], "opaque": []}),
        "also 1: not after the row before it",
    );
    refused(
        json!({"also": [[2, "a.py", 2]], "opaque": []}),
        "also names site 2",
    );
    refused(
        json!({"also": [[0, "pkg/__init__.py", 2]], "opaque": []}),
        "repeats site 0's answer",
    );
    refused(json!({"also": []}), "no `opaque` table");
    refused(
        json!({"also": [], "opaque": [1, 0]}),
        "opaque rows not ascending",
    );
    refused(json!({"also": [], "opaque": [2]}), "opaque names site 2");
}
