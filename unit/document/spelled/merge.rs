//! The W7 differential's merge leg (plan v2.33 W7; mounted inside the
//! frozen merge face): each member's path and unit, a member's text at
//! a hole and that text clipped to a cap, and the reasons spelled by the
//! core and by the frozen `Names` / `clip`. The face sends one text per
//! hole row, cut here by the frozen `span_text`; a text reference names
//! a hole row's member and nodes (the only triples the core's document
//! and lines name), or a member the face does not hold.

use super::super::kit::{Leg, leg};
use super::{Names, Texts, span_text};
use crate::dedup::t3::tree::UnitTree;
use crate::merge::groups::Member;
use serde_json::{Value, json};

/// A member over one of `paths` (whose texts are `texts`), its tree's
/// spans drawn over the text's bytes (a char boundary or not).
fn member(leg: &mut Leg, paths: &[String], texts: &Texts) -> Member {
    let path = paths[leg.rng.below(paths.len())].clone();
    let len = texts[&path].len();
    // mostly a span inside the text; now and then one past its end
    let span = |leg: &mut Leg| {
        let s = leg.rng.below(len + 1);
        let e = s + leg.rng.below(len - s + 1);
        if leg.rng.chance(10) {
            (s, len + 1)
        } else {
            (s, e)
        }
    };
    let spans = (0..1 + leg.rng.below(4)).map(|_| span(leg)).collect();
    let unit = leg.rng.chance(70).then(|| leg.word());
    let tree = UnitTree {
        spans,
        ..UnitTree::default()
    };
    Member {
        path,
        unit,
        lines: (1, 1),
        run: (1, 1),
        tree,
    }
}

/// One face: its groups' members (id = place across groups), the
/// member and hole rows, each hole row's member id and nodes.
struct Face {
    members: Vec<Member>,
    rows: Value,
    holes: Vec<[i64; 3]>,
}

fn face(leg: &mut Leg, paths: &[String], texts: &Texts) -> Face {
    let (mut members, mut member_rows, mut hole_rows, mut holes) = (vec![], vec![], vec![], vec![]);
    for g in 0..1 + leg.rng.below(3) {
        let first = members.len();
        for m in 0..2 + leg.rng.below(2) {
            member_rows.push(json!([members.len(), g, m, 1, 1, 1, 1, 1]));
            members.push(member(leg, paths, texts));
        }
        for hole in 0..leg.rng.below(4) {
            let m = leg.rng.below(members.len() - first);
            let (post, end) = (leg.rng.below(5) as i64 - 1, leg.rng.below(5) as i64 - 1);
            hole_rows.push(json!([g, hole, 0, m, post, end]));
            holes.push([(first + m) as i64, post, end]);
        }
    }
    Face {
        members,
        rows: json!({"members": member_rows, "holes": hole_rows}),
        holes,
    }
}

/// A text or clipped reference: a hole row's triple, or a member past
/// the face's.
fn text_ref(leg: &mut Leg, f: &Face) -> Value {
    let [k, post, end] = match f.holes.len() {
        0 => [f.members.len() as i64, 0, 0],
        n => f.holes[leg.rng.below(n)],
    };
    let k = if leg.rng.chance(5) {
        f.members.len() as i64
    } else {
        k
    };
    match leg.rng.below(2) {
        0 => json!(["text", k, post, end]),
        _ => json!(["clipped", k, post, end, leg.rng.below(30) as i64 - 2]),
    }
}

leg!(merge_spells_as_the_frozen_face, "merge", 0x771b, |leg| {
    let paths = leg.paths(1, 3);
    let texts: Texts = paths
        .iter()
        .map(|p| (p.clone(), leg.words(12).join(" \t")))
        .collect();
    let f = face(leg, &paths, &texts);
    let cut = |&[k, post, end]: &[i64; 3]| {
        let m = &f.members[k as usize];
        span_text(&texts[&m.path], &m.tree, post, end)
    };
    let hole_text: Vec<String> = f.holes.iter().map(cut).collect();
    let (why, reasons) = leg.reasons();
    let (p, u): (Vec<&String>, Vec<&Option<String>>) =
        f.members.iter().map(|m| (&m.path, &m.unit)).unzip();
    let sent = json!({"path": p, "unit": u, "holeText": hole_text, "why": reasons});
    let n = f.members.len();
    let drawn = [
        ("path", vec![n]),
        ("unit", vec![n]),
        ("why", vec![reasons.len()]),
    ];
    let mut refs = leg.references(&drawn, 100);
    refs.extend((0..100).map(|_| text_ref(leg, &f)));
    let frozen = Names {
        members: f.members.iter().collect(),
        texts: texts.clone(),
        why,
    };
    leg.spell("merge", (&sent, &f.rows), &frozen, refs);
});
