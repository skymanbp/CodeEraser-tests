//! The grouping's boundary (plan v2.35, the dead-file amendment, 5A):
//! the core's `roots` / `under` held to the `dead` rows, every refusal
//! by name. One literal table: a reply's grouping keys, then ` => `,
//! then the words its refusal must carry (`ok` = accepted).

use crate::testutil::node;
use serde_json::{Value, json};

/// Four files; 0, 1 and 2 are dead (0 the root of 1 and 2), 3 is not.
const CASES: &str = r#"
{"roots": [[0, 0]], "under": [[1, 0], [2, 0]]} => ok
{"roots": [[0, 0]], "under": [], "underDropped": true} => ok
{"under": [[1, 0], [2, 0]]} => pre-amendment core: graph/1 offers no `roots`
{"roots": [[0, 0]]} => pre-amendment core: graph/1 offers no `under`
{"roots": [[0, 0], [3, 0]], "under": [[1, 0], [2, 0]]} => root 3 is not a dead row
{"roots": [[1, 0], [0, 0]], "under": [[2, 0]]} => `roots` row 1 is not strictly ascending
{"roots": [[0, 0], [0, 0]], "under": [[1, 0], [2, 0]]} => `roots` row 1 is not strictly ascending
{"roots": [[0, 0]], "under": [[2, 0], [1, 0]]} => `under` row 1 is not strictly ascending
{"roots": [[0, 0]], "under": [[1, 0], [1, 0], [2, 0]]} => `under` row 1 is not strictly ascending
{"roots": [[0, 4]], "under": [[1, 0], [2, 0]]} => state 4 outside 0..3
{"roots": [[0, 0], [1, 0]], "under": [[1, 0], [2, 0]]} => pair (1, 0): the dependent is a root
{"roots": [[0, 0]], "under": [[1, 0], [2, 0], [3, 0]]} => pair (3, 0): the dependent is not a dead row
{"roots": [[0, 0]], "under": [[1, 0], [2, 1]]} => pair (2, 1): the root is not in `roots`
{"roots": [[0, 0]], "under": [[1, 0]]} => dead row 2 is neither a root nor under one
{"roots": [[0, 0]], "under": [[1, 0], [2, 0]], "underDropped": true} => `under` rows beside `underDropped`
"#;

#[test]
fn the_grouping_is_held_to_the_dead_rows_and_refused_by_name() {
    let nodes: Vec<_> = ["a.rs", "b.rs", "c.rs", "d.rs"]
        .iter()
        .map(|p| node(p, "", super::super::super::wire::GRAN_FILE))
        .collect();
    for (keys, says) in crate::testutil::case_table(CASES, 15) {
        let mut reply: Value = json!({
            "dead": [[0, 1, 2], [1, 3, 2], [2, 3, 2]], "reported": [],
            "fail": false, "blocked": [], "counts": {"kept": 2}
        });
        let extra: Value = serde_json::from_str(keys).expect(keys);
        for (k, v) in extra.as_object().expect("an object") {
            reply[k] = v.clone();
        }
        let got = super::consume(&reply, &nodes, None, None);
        if let Some(j) = crate::testutil::holds(keys, says, got) {
            assert_eq!(j.grouping.roots[0], [0, 0], "{keys}");
        }
    }
}
