//! The merge document's labelling (plan v2.31 step 7, step-7 rulings 5
//! and 6): a parameter's entry takes every member's text at its first
//! hole, from the hole's first root to its last — a gap's whole forest —
//! and an empty side is "".

use super::*;
use crate::dedup::t3::tree::UnitTree;
use crate::merge::groups::{FAMILY_NEAR, Member};

const TEXT: &str = "f(alpha, beta)";

fn member() -> Member {
    Member {
        path: "a.py".into(),
        unit: Some("a.py:f/2#0".into()),
        lines: (1, 1),
        run: (1, 1),
        tree: UnitTree {
            lab: vec![1, 1, 2],
            lld: vec![0, 1, 0],
            leaf: vec![5, 6, 0],
            slot: vec![1, 1, 4],
            spans: vec![(2, 7), (9, 13), (0, 14)],
            ..UnitTree::default()
        },
    }
}

fn hole(hole: usize, param: usize, m: usize, post: i64, post_end: i64) -> HoleRow {
    HoleRow {
        hole,
        param,
        m,
        post,
        post_end,
    }
}

#[test]
fn a_parameter_reads_its_first_hole_on_every_member() {
    let g = Group {
        family: FAMILY_EXACT,
        fragment: false,
        members: vec![member(), member()],
    };
    let s = Suggestion {
        params: 1,
        kept: 0,
        savings: -1,
        feasible: true,
        reason: 0,
    };
    let holes = [
        hole(0, 0, 0, 0, 0),
        hole(0, 0, 1, 1, 1),
        hole(1, 0, 0, 1, 1),
        hole(1, 0, 1, 0, 0),
    ];
    let texts: Texts = [("a.py".to_string(), TEXT.to_string())].into();
    let face = label(3, &g, &s, &holes, &texts);
    assert_eq!((face.group, face.family, face.reason), (3, "t1t2", "ok"));
    let values: Vec<_> = face.holes[0]
        .values
        .iter()
        .map(|v| v.text.as_str())
        .collect();
    assert_eq!(values, ["alpha", "beta"]);
    assert_eq!(face.holes.len(), 1, "one entry per parameter");
    let near = Group {
        family: FAMILY_NEAR,
        ..g
    };
    let gap = [hole(0, 0, 0, 0, 1), hole(0, 0, 1, -1, -1)];
    let far = label(0, &near, &s, &gap, &texts);
    let v = &far.holes[0].values;
    assert_eq!(v[0].text, "alpha, beta", "first root to last");
    assert_eq!(v[1].text, "", "an empty side");
    assert_eq!(face.members[0].run, [1, 1]);
    assert_eq!((REASONS[1], REASONS[5]), ("position", "no_savings"));
    assert_eq!(REASONS.len(), 6);
}
