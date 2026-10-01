//! The merge/1 leg's body and strict consume (plan v2.31 step 7): the
//! request carries what the core's golden reads, a healthy reply maps
//! onto the groups sent, a degraded one is a cap-mirror drift (this side
//! priced the chunk within both caps), and counts or rows that disagree
//! with what was sent are wire skew.

use super::*;
use crate::merge::groups::{FAMILY_EXACT, Member};

fn group(members: usize, nodes: usize) -> Group {
    Group {
        family: FAMILY_EXACT,
        fragment: false,
        members: (0..members)
            .map(|m| Member {
                path: format!("m{m}.py"),
                unit: None,
                lines: (1, 6),
                run: (1, 6),
                tree: UnitTree {
                    lab: vec![7; nodes],
                    lld: vec![0; nodes],
                    leaf: vec![0; nodes],
                    slot: vec![1; nodes],
                    spans: vec![(0, 0); nodes],
                    own: vec![0; nodes],
                    text: vec![3; nodes],
                },
            })
            .collect(),
    }
}

/// The golden's second pair: three members, one parameter over two
/// holes, every member at nodes 0 and 1 (each a leaf: post = postEnd).
fn healthy() -> Value {
    json!({"counts":{"feasible":1,"groups":1,"holes":6,"members":3,"nodes":12,"suggestions":1},
        "degraded":false,
        "holes":[[0,0,0,0,0,0],[0,0,0,1,0,0],[0,0,0,2,0,0],[0,1,0,0,1,1],[0,1,0,1,1,1],[0,1,0,2,1,1]],
        "suggestions":[[0,1,1,9,1,0]]})
}

/// The body: groups with their family and helper lines, members with
/// their lines and in-degree, one tree per member with dense labels and
/// the four added columns (merge generation 2, ruling R1).
#[test]
fn the_body_carries_every_table() {
    let g = group(2, 3);
    let body = body(&[&g], |p| if p == "m1.py" { 4 } else { 0 });
    assert_eq!(body["groups"], json!([[0, 0, 0]]));
    assert_eq!(body["members"], json!([[0, 0, 0, 6, 0], [0, 1, 1, 6, 4]]));
    let tree = json!({"lab": [0, 0, 0], "lld": [0, 0, 0], "leaf": [0, 0, 0], "slot": [1, 1, 1],
        "own": [0, 0, 0], "text": [3, 3, 3]});
    assert_eq!(body["trees"], json!([tree, tree]));
}

/// Ruling R6: a fragment group carries its language's helper lines
/// (Python 1, a brace language 2), a whole-unit group none.
#[test]
fn a_fragment_group_carries_its_helper_lines() {
    let mut py = group(2, 3);
    py.fragment = true;
    let mut rs = group(2, 3);
    rs.fragment = true;
    for m in &mut rs.members {
        m.path = m.path.replace(".py", ".rs");
    }
    let whole = group(2, 3);
    let body = body(&[&py, &rs, &whole], |_| 0);
    assert_eq!(body["groups"], json!([[0, 0, 1], [1, 0, 2], [2, 0, 0]]));
}

#[test]
fn a_healthy_reply_maps_onto_the_groups_sent() {
    let g = group(3, 4);
    let j = consume(&healthy(), &[&g]).expect("healthy");
    let (s, holes) = &j.groups[0];
    let want = Suggestion {
        params: 1,
        kept: 1,
        savings: 9,
        feasible: true,
        reason: 0,
    };
    assert_eq!(*s, want);
    assert_eq!(holes.len(), 6);
    assert_eq!((holes[3].post, holes[3].post_end), (1, 1));
    assert_eq!(j.counts, [1, 3, 12, 1, 6, 1]);
}

/// A chunk this side priced within both caps and the core degraded:
/// the mirrors drifted — an error naming both owners, never a document.
#[test]
fn a_degraded_reply_is_cap_mirror_drift() {
    let g = group(3, 4);
    let mut reply = healthy();
    reply["degraded"] = json!(true);
    reply["reason"] = json!("merge_too_large");
    let err = consume(&reply, &[&g]).expect_err("drift");
    assert!(err.contains("cap mirror drift"), "{err}");
    assert!(err.contains("merge/wire.rs vs Merge/Cost.hs"), "{err}");
}

/// Counts, rows and bounds that disagree with what was sent refuse —
/// a feasible suggestion that saves no line among them.
#[test]
fn skew_is_refused() {
    let g = group(3, 4);
    let at = [
        "/counts/nodes",
        "/counts/holes",
        "/counts/feasible",
        "/suggestions/0/1",
        "/suggestions/0/3",
        "/suggestions/0/5",
        "/holes/0/4",
        "/holes/1/3",
    ];
    let forged = [11, 5, 0, 2, 0, 1, 4, 3];
    for (at, v) in at.into_iter().zip(forged) {
        let mut reply = healthy();
        *reply.pointer_mut(at).expect(at) = json!(v);
        let err = consume(&reply, &[&g]).expect_err(at);
        assert!(err.starts_with("wire skew"), "{at}: {err}");
    }
    let mut swapped = healthy();
    swapped["holes"].as_array_mut().unwrap().swap(0, 1);
    assert!(consume(&swapped, &[&g]).is_err(), "rows out of order");
}
