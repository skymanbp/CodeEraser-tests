//! The merge document's labelling (plan v2.31 step 7, step-7 rulings 5
//! and 6): a parameter's entry takes every member's text at its first
//! hole, from the hole's first root to its last — a gap's whole forest —
//! and an empty side is "". Since plan v2.32 step 3 the core lays the
//! document out: these legs send the request this side builds to the
//! core this process measures with and read the bound document back.

use super::*;
use crate::dedup::t3::tree::UnitTree;
use crate::merge::groups::{FAMILY_NEAR, Member};
use crate::merge::report::{GroupFace, Report};
use serde::Deserialize;

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

/// One group with its answer, laid out and read.
fn laid_out(g: Group, s: Suggestion, holes: Vec<HoleRow>) -> GroupFace {
    let groups = [g];
    let answers = vec![Judged {
        groups: vec![(s, holes)],
        counts: [1, 2, 6, 1, 0, 1],
    }];
    let names = Names {
        members: groups.iter().flat_map(|g| &g.members).collect(),
        texts: [("a.py".to_string(), TEXT.to_string())].into(),
        why: crate::document::Why::default(),
    };
    let req = request(&groups, answers, Unsendable::default(), 0)
        .range("members", names.members.len())
        .range("why", 0);
    let core = crate::daemon::judge::core_bin().expect("a core");
    let doc = document::assemble(&core, req, &names).expect("laid out");
    let mut r = Report::deserialize(&doc).expect("read");
    r.groups.remove(0)
}

#[test]
fn a_parameter_reads_its_first_hole_on_every_member() {
    let g = || Group {
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
    let holes = vec![
        hole(0, 0, 0, 0, 0),
        hole(0, 0, 1, 1, 1),
        hole(1, 0, 0, 1, 1),
        hole(1, 0, 1, 0, 0),
    ];
    let face = laid_out(g(), s, holes);
    assert_eq!(
        (face.group, face.family.as_str(), face.reason.as_str()),
        (0, "t1t2", "ok")
    );
    let values: Vec<_> = face.holes[0]
        .values
        .iter()
        .map(|v| v.text.as_str())
        .collect();
    assert_eq!(values, ["alpha", "beta"]);
    assert_eq!(face.holes.len(), 1, "one entry per parameter");
    let near = Group {
        family: FAMILY_NEAR,
        ..g()
    };
    let gap = vec![hole(0, 0, 0, 0, 1), hole(0, 0, 1, -1, -1)];
    let far = laid_out(near, s, gap);
    let v = &far.holes[0].values;
    assert_eq!(v[0].text, "alpha, beta", "first root to last");
    assert_eq!(v[1].text, "", "an empty side");
    assert_eq!(far.family, "t3");
    assert_eq!(face.members[0].run, [1, 1]);
    assert_eq!((REASONS[1], REASONS[5]), ("position", "no_savings"));
    assert_eq!(REASONS.len(), 6);
}
