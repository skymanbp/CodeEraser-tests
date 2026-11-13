//! The dead-file gate's boundary (plan v2.35, the dead-file amendment,
//! 5B): the core's `blocked`, `blockUnanswered`, `fail` and `deadNow`
//! held to the grouping and to what the request carried, every refusal
//! by name. One literal table: whether entities rode (`e`) or not (`-`),
//! whether the reply degraded (`d`) or not (`-`), the reply's gate keys,
//! then ` => ` and the words its refusal must carry (`ok` = accepted).

use super::{Gate, consume};
use crate::graph::deadcode::Grouping;
use serde_json::{Value, json};

/// Roots 0 (state born) and 1 (standing); 2 is a dependent.
const CASES: &str = r#"
e - {"blocked": [0], "fail": true, "deadNow": {"files": [1, 2, 3], "dead": [1, 2]}} => ok
e - {"blocked": [], "fail": false, "deadNow": {"files": [1, 2, 3], "dead": [1, 2]}} => ok
- - {"blocked": [], "fail": false, "blockUnanswered": true} => ok
- d {"blocked": [], "fail": true} => ok
e d {"blocked": [], "fail": true} => ok
- - {"fail": false} => pre-amendment core: graph/1 offers no `blocked`
- - {"blocked": [1, 0], "fail": true} => `blocked` row 1 is not strictly ascending
- - {"blocked": [0, 0], "fail": true} => `blocked` row 1 is not strictly ascending
- - {"blocked": [2], "fail": true} => blocked node 2 is not a root
- - {"blocked": [0], "fail": true, "blockUnanswered": true} => `blocked` rows beside `blockUnanswered`
- - {"blocked": [0], "fail": false} => `fail` false is not the gate's answer
- - {"blocked": [], "fail": true} => `fail` true is not the gate's answer
e - {"blocked": [], "fail": false} => offers no `deadNow` for the entities it was sent
- - {"blocked": [], "fail": false, "deadNow": {"files": [], "dead": []}} => answered `deadNow` to a request without entities
e - {"blocked": [], "fail": false, "deadNow": {"files": [3, 1], "dead": []}} => `deadNow.files` is not strictly ascending
e - {"blocked": [], "fail": false, "deadNow": {"files": [1], "dead": "x"}} => `deadNow.dead`: not a list of ids
"#;

#[test]
fn the_gate_is_held_to_the_grouping_and_the_request_and_refused_by_name() {
    let g = Grouping {
        roots: vec![[0, 3], [1, 1]],
        under: vec![[2, 0]],
        dropped: false,
    };
    let sent = Gate {
        entities: vec![(0, 3), (1, 1), (2, 2)],
        point: None,
        block: 2,
    };
    for (line, says) in crate::testutil::case_table(CASES, 16) {
        let (flags, keys) = line.split_at(4);
        let reply: Value = serde_json::from_str(keys).expect(keys);
        let degraded = flags.as_bytes()[2] == b'd';
        let fail = reply["fail"].as_bool().expect("a fail bit");
        let entities = (flags.as_bytes()[0] == b'e').then_some(&sent);
        crate::testutil::holds(line, says, consume(&reply, &g, entities, (degraded, fail)));
    }
}

/// The request keys: one entity row per file node, the point verbatim
/// when it rode and absent otherwise, the mask always; a blocked root's
/// id is read off the table this side sent.
#[test]
fn the_gate_rides_the_request_as_sent() {
    let mut gate = Gate {
        entities: vec![(0, 7), (2, u64::MAX)],
        point: None,
        block: 3,
    };
    let mut obj = serde_json::Map::new();
    gate.insert(&mut obj);
    assert_eq!(
        Value::Object(obj),
        json!({"entities": [[0, 7], [2, u64::MAX]], "deadBlock": 3})
    );
    gate.point = Some(json!({"files": [7], "dead": []}));
    let mut obj = serde_json::Map::new();
    gate.insert(&mut obj);
    assert_eq!(obj["deadPoint"], json!({"files": [7], "dead": []}));
    assert_eq!((gate.id_of(2), gate.id_of(1)), (Some(u64::MAX), None));
}
