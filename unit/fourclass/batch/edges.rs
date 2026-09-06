use super::*;
use crate::fourclass::model::classify;
use crate::scan::lang::Lang;
use serde_json::json;

/// Two pairs: `helper/1` leaves pair 0 and arrives at pair 1.
fn pairs() -> Vec<Classification> {
    let gone = "fn helper(d: u32) -> u32 {\n    d.wrapping_mul(7)\n}\n";
    let arrived = "pub fn helper(d: u32) -> u32 {\n    d.wrapping_mul(7)\n}\n";
    vec![
        classify(gone, "", Lang::Rust),
        classify("", arrived, Lang::Rust),
    ]
}

fn hash_of(pairs: &[Classification]) -> u64 {
    key_hash(&pairs[0].decls.0[0])
}

/// (case, reply, Ok rows or Err fragment).
#[test]
fn the_reply_is_an_answer_not_an_authority() {
    let ps = pairs();
    let h = hash_of(&ps);
    let good = json!({"unitEdges": [[0, 1, h]]});
    let got = unit_edges(&good, &ps, &[]).expect("accepted");
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].lines, 0, "a declaration edge claims no line");
    assert_eq!(got[0].to_unit.as_deref(), Some("helper/1"));

    // a pre-7.1.0 core and an over-cap batch both mean "no edges",
    // never an error
    assert!(unit_edges(&json!({}), &ps, &[]).expect("absent").is_empty());
    let dropped = json!({"unitEdges": [], "unitEdgesDropped": true});
    assert!(unit_edges(&dropped, &ps, &[]).expect("dropped").is_empty());

    for (why, reply) in [
        ("self edge", json!({"unitEdges": [[0, 0, h]]})),
        ("unsent source", json!({"unitEdges": [[1, 0, h]]})),
        ("out of range", json!({"unitEdges": [[0, 9, h]]})),
        ("unknown hash", json!({"unitEdges": [[0, 1, 12345]]})),
        (
            "not ascending",
            json!({"unitEdges": [[0, 1, h], [0, 1, h]]}),
        ),
        ("row shape", json!({"unitEdges": [[0, 1]]})),
        ("extra cell", json!({"unitEdges": [[0, 1, h, 0]]})),
        ("negative index", json!({"unitEdges": [[-1, 1, h]]})),
        (
            "false cap flag",
            json!({"unitEdges": [], "unitEdgesDropped": false}),
        ),
        (
            "truncated cap reply",
            json!({"unitEdges": [[0, 1, h]], "unitEdgesDropped": true}),
        ),
    ] {
        assert!(
            unit_edges(&reply, &ps, &[]).is_err(),
            "{why} must be refused"
        );
    }
}

#[test]
fn the_line_stage_wins_an_edge_it_already_named() {
    let ps = pairs();
    let h = hash_of(&ps);
    let line = Relocation {
        from_pair: 0,
        from_unit: Some("helper/1".into()),
        to_pair: 1,
        to_unit: Some("helper/1".into()),
        lines: 2,
    };
    let reply = json!({"unitEdges": [[0, 1, h]]});
    assert!(
        unit_edges(&reply, &ps, &[line])
            .expect("accepted")
            .is_empty(),
        "a relocation is reported once"
    );
}
