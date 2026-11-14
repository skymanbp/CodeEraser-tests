//! The dead-file gate's boundary (plan v2.35, the dead-file amendment,
//! 5B): the core's `states`, `blocked`, `blockUnanswered`, `fail` and
//! `deadNow` held to the dead rows and to what the request carried, every
//! refusal by name. One literal table: whether entities rode (`e`) or not
//! (`-`), whether the reply degraded (`d`) or not (`-`), the reply's gate
//! keys (`states` is the three dead files' unless the row names it), then
//! ` => ` and the words its refusal must carry (`ok` = accepted).

use super::{Gate, consume};
use serde_json::{Value, json};

/// Dead files 0 (state born, a root), 1 (standing, a root) and 2
/// (orphaned, a dependent); node 3 is referenced.
const CASES: &str = r#"
e - {"blocked": [0], "fail": true, "deadNow": {"files": [1, 2, 3], "dead": [1, 2]}} => ok
e - {"blocked": [0, 2], "fail": true, "deadNow": {"files": [1, 2, 3], "dead": [1, 2]}} => ok
e - {"blocked": [], "fail": false, "deadNow": {"files": [1, 2, 3], "dead": [1, 2]}} => ok
- - {"blocked": [], "fail": false, "blockUnanswered": true} => ok
- d {"blocked": [], "fail": true} => ok
e d {"blocked": [], "fail": true} => ok
- - {"fail": false} => pre-amendment core: graph/1 offers no `blocked`
- - {"blocked": [1, 0], "fail": true} => `blocked` row 1 is not strictly ascending
- - {"blocked": [0, 0], "fail": true} => `blocked` row 1 is not strictly ascending
- - {"blocked": [3], "fail": true} => blocked node 3 is not an unreferenced file
- - {"blocked": [], "fail": false, "states": null} => pre-amendment core: graph/1 offers no `states`
- - {"blocked": [], "fail": false, "states": [[0, 3], [1, 1]]} => `states` is not one row per dead row
- - {"blocked": [], "fail": false, "states": [[1, 1], [0, 3], [2, 2]]} => `states` is not one row per dead row
- - {"blocked": [], "fail": false, "states": [[0, 3], [1, 4], [2, 2]]} => file 1: state 4 outside 0..3
- - {"blocked": [0], "fail": true, "blockUnanswered": true} => `blocked` rows beside `blockUnanswered`
- - {"blocked": [0], "fail": false} => `fail` false is not the gate's answer
- - {"blocked": [], "fail": true} => `fail` true is not the gate's answer
e - {"blocked": [], "fail": false} => offers no `deadNow` for the entities it was sent
- - {"blocked": [], "fail": false, "deadNow": {"files": [], "dead": []}} => answered `deadNow` to a request without entities
e - {"blocked": [], "fail": false, "deadNow": {"files": [3, 1], "dead": []}} => `deadNow.files` is not strictly ascending
e - {"blocked": [], "fail": false, "deadNow": {"files": [1], "dead": "x"}} => `deadNow.dead`: not a list of ids
"#;

#[test]
fn the_gate_is_held_to_the_dead_rows_and_the_request_and_refused_by_name() {
    let dead = vec![vec![0, 1, 2], vec![1, 1, 2], vec![2, 3, 2]];
    let sent = Gate {
        entities: vec![(0, 3), (1, 1), (2, 2), (3, 4)],
        point: None,
        block: 3,
    };
    for (line, says) in crate::testutil::case_table(CASES, 21) {
        let (flags, keys) = line.split_at(4);
        let mut reply: Value = serde_json::from_str(keys).expect(keys);
        if reply.get("states").is_none() {
            reply["states"] = json!([[0, 3], [1, 1], [2, 2]]);
        }
        let degraded = flags.as_bytes()[2] == b'd';
        let fail = reply["fail"].as_bool().expect("a fail bit");
        let entities = (flags.as_bytes()[0] == b'e').then_some(&sent);
        if let Some(g) = crate::testutil::holds(
            line,
            says,
            consume(&reply, &dead, entities, (degraded, fail)),
        ) {
            assert_eq!(g.states, vec![[0, 3], [1, 1], [2, 2]], "{line}");
        }
    }
}

/// The request keys: one entity row per file node, the point verbatim
/// when it rode and absent otherwise, the mask always; a blocked file's
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

/// The face's exit rule (release 1.10.0): a degraded document is
/// unjudged with and without `--check` (exit 2, as flow, arch and merge);
/// an unanswered gate is unjudged only under `--check`; an answered gate
/// is judged either way, whether or not it blocks (its no is exit 1).
#[test]
fn a_degraded_document_is_unjudged_with_and_without_check() {
    let degraded = json!({"degraded": "graph_too_large", "gate": {"answered": true}});
    let unanswered = json!({"degraded": null, "gate": {"answered": false}});
    let answered = json!({"degraded": null, "gate": {"answered": true, "blocked": 1}});
    let read = |doc: &Value| (super::unjudged(doc, false), super::unjudged(doc, true));
    assert_eq!(
        (read(&degraded), read(&unanswered), read(&answered)),
        ((true, true), (false, true), (false, false))
    );
}
