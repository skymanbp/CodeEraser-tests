//! The merge family's groups (plan v2.31 step 7, step-7 ruling 4): the
//! whole-unit match and its 90 % floor, the fragment's synthetic root,
//! the trim to the members' common run of tops, the local isomorphism
//! check, the counters of the groups not sent and the chunking that
//! never splits a group.

use super::*;
use crate::dedup::struct_fp::kind_code;
use crate::dedup::t3::tree::fragment_of;
use crate::merge::wire::{GROUP_CAP, chunks};

fn unit(path: &str, start: i64, end: i64) -> Unit {
    Unit {
        path: path.into(),
        key: format!("f{start}/0"),
        nth: 0,
        lang: "python".into(),
        nodes: 30,
        start_line: start,
        end_line: end,
        sig: Vec::new(),
        hist: Vec::new(),
    }
}

/// A member span keeps the longest unit inside it when that unit spans
/// nine tenths of it; a shorter one, or one outside it, is no match.
#[test]
fn a_whole_unit_fills_nine_tenths_of_its_member() {
    let units = [unit("a.py", 1, 9), unit("a.py", 2, 4), unit("a.py", 12, 20)];
    let ids = [0, 1, 2];
    assert_eq!(filling(1, 10, &ids, &units), Some(0), "9 of 10 lines");
    assert_eq!(filling(1, 11, &ids, &units), None, "9 of 11 lines");
    assert_eq!(filling(2, 5, &ids, &units), None, "3 of 4 lines");
    assert_eq!(filling(12, 20, &ids, &units), Some(2), "exact");
    assert_eq!(filling(30, 40, &ids, &units), None, "none inside");
}

/// A fragment's tree: the selected run under one synthetic root — its
/// lld 0, leaf 0, slot 4, its span and its lines the run's; a span that
/// selects nothing has no tree.
#[test]
fn a_fragment_hangs_its_run_under_a_synthetic_root() {
    let text = "def f(a):\n    x = a + 1\n    y = x * 2\n    return y\n";
    let trees = file_fragments(text, Lang::Python, &[(2, 3), (40, 41)], Extras::Without);
    let (t, run) = fragment_of(trees[0].as_ref().expect("two statements")).expect("a run");
    assert_eq!(run, (2, 3), "the run's first line to its last");
    let root = t.lab.len() - 1;
    assert_eq!(t.lab[root], kind_code("ce:fragment"));
    assert_eq!((t.lld[root], t.leaf[root], t.slot[root]), (0, 0, 4));
    let (s, e) = t.spans[root];
    assert_eq!(&text[s..e], "x = a + 1\n    y = x * 2");
    assert!(trees[1].is_none(), "nothing selected");
}

fn tree(lab: &[u64], lld: &[i64], leaf: &[u64]) -> UnitTree {
    UnitTree {
        lab: lab.to_vec(),
        lld: lld.to_vec(),
        leaf: leaf.to_vec(),
        slot: vec![1; lab.len()],
        spans: vec![(0, 0); lab.len()],
        ..UnitTree::default()
    }
}

/// A three-node tree, and the same shape with its root relabelled.
fn base() -> UnitTree {
    tree(&[1, 2, 9], &[0, 1, 0], &[11, 12, 0])
}

fn relabelled() -> UnitTree {
    tree(&[1, 2, 8], &[0, 1, 0], &[11, 12, 0])
}

/// One shape and every internal node's label and hash alike: leaves
/// may differ; a different shape or internal label may not.
#[test]
fn isomorphism_holds_leaves_open_and_the_rest_fixed() {
    let a = base();
    let renamed = tree(&[1, 3, 9], &[0, 1, 0], &[21, 22, 0]);
    let reshaped = tree(&[1, 9], &[0, 0], &[11, 0]);
    assert!(isomorphic(&[&a, &renamed]));
    assert!(!isomorphic(&[&a, &relabelled()]));
    assert!(!isomorphic(&[&a, &reshaped]));
}

type Frags = BTreeMap<Span, Option<Tops>>;

/// The counter a two-member fragment family in `path` lands in (all
/// zero when it is sent).
fn counted(path: &str, frags: &Frags) -> Unsendable {
    let plan = Plan {
        family: FAMILY_EXACT,
        spans: vec![(path.into(), 1, 2), (path.into(), 3, 4)],
        units: None,
    };
    let mut u = Unsendable::default();
    if let Err(counter) = settle(plan, &[], &BTreeMap::new(), frags) {
        *counter(&mut u) += 1;
    }
    u
}

fn only(counter: Counter) -> Unsendable {
    let mut u = Unsendable::default();
    *counter(&mut u) += 1;
    u
}

fn frags_of(a: &UnitTree, b: &UnitTree) -> Frags {
    let at = |s, e, t: &UnitTree| {
        let top = Top {
            tree: t.clone(),
            lines: (s, e),
            stream: Vec::new(),
        };
        (("m.py".to_string(), s, e), Some(vec![top]))
    };
    [at(1, 2, a), at(3, 4, b)].into()
}

/// A two-member fragment family over `text`'s spans: its group, or
/// the counter it lands in.
fn fragment_family(text: &str, a: (usize, usize), b: (usize, usize)) -> Result<Group, Unsendable> {
    let tops = file_fragments(text, Lang::Python, &[a, b], Extras::Without);
    let frags: Frags = [a, b]
        .into_iter()
        .zip(tops)
        .map(|((s, e), t)| (("m.py".to_string(), s, e), t))
        .collect();
    let plan = Plan {
        family: FAMILY_EXACT,
        spans: vec![("m.py".into(), a.0, a.1), ("m.py".into(), b.0, b.1)],
        units: None,
    };
    settle(plan, &[], &BTreeMap::new(), &frags).map_err(only)
}

/// A boundary line holding half a statement one member has and the
/// other does not: the trim keeps the run both hold, the family is one
/// shape and is sent, and its text is the kept run's.
#[test]
fn a_boundary_statement_on_one_side_is_trimmed_off() {
    let text = "def f(a):\n    x = a + 1\n    y = x * 2\n    print(y)\n    return y\n\n\
                def g(b):\n    z = 0; x = b + 1\n    y = x * 2\n    print(y)\n    return y\n";
    let g = fragment_family(text, (2, 4), (8, 10)).expect("sent");
    let t = &g.members[1].tree;
    let (s, e) = t.spans[t.spans.len() - 1];
    assert_eq!(&text[s..e], "x = b + 1\n    y = x * 2\n    print(y)");
    assert_eq!(g.members[1].lines, (8, 10), "the member keeps its span");
}

/// A boundary statement on a line of its own trimmed off one member:
/// the member keeps its clone-family span as its identity, its run is
/// the kept lines, and the request prices the run.
#[test]
fn a_trimmed_member_is_priced_by_its_run() {
    let text = "def f(a):\n    x = a + 1\n    y = x * 2\n    print(y)\n    return y\n\n\
                def g(b):\n    z = 0\n    x = b + 1\n    y = x * 2\n    print(y)\n    return y\n";
    let g = fragment_family(text, (2, 4), (8, 11)).expect("sent");
    let spans: Vec<_> = g.members.iter().map(|m| (m.lines, m.run)).collect();
    assert_eq!(spans, [((2, 4), (2, 4)), ((8, 11), (9, 11))]);
    let body = crate::merge::wire::body(&[&g], |_| 0);
    assert_eq!(
        body["members"],
        serde_json::json!([[0, 0, 0, 3, 0], [0, 1, 1, 3, 0]])
    );
}

/// A statement one member has inside the one top both share: no run of
/// tops in common, the family is not one shape.
#[test]
fn a_statement_inside_one_side_is_still_not_isomorphic() {
    let text = "def f(a):\n    for x in a:\n        print(x)\n        a.pop()\n    return a\n\n\
                def g(b):\n    for x in b:\n        print(x)\n        log(x)\n        b.pop()\n    return b\n";
    let got = fragment_family(text, (2, 4), (8, 11)).map(|_| ());
    assert_eq!(got, Err(only(|u| &mut u.not_isomorphic)));
}

/// The counters: a language with no slot table, a tree not built, the
/// node cap, a shape that is not one — each group lands in its own.
#[test]
fn every_group_not_sent_is_counted_by_why() {
    let none = Frags::new();
    assert_eq!(counted("m.md", &none), only(|u| &mut u.no_slot_table));
    assert_eq!(counted("m.py", &none), only(|u| &mut u.unbuilt));
    let big = TREE_NODE_CAP / 2 + 1;
    let lld: Vec<i64> = (0..big as i64).collect();
    let wide = tree(&vec![1; big], &lld, &vec![0; big]);
    assert_eq!(
        counted("m.py", &frags_of(&wide, &wide)),
        only(|u| &mut u.over_cap)
    );
    let (a, b) = (base(), relabelled());
    assert_eq!(
        counted("m.py", &frags_of(&a, &b)),
        only(|u| &mut u.not_isomorphic)
    );
    assert_eq!(counted("m.py", &frags_of(&a, &a)), Unsendable::default());
}

/// Two whole units: a group over them from `family`, member order as
/// given.
fn pair(family: u8, units: [&str; 2]) -> Group {
    Group {
        family,
        fragment: false,
        members: units
            .iter()
            .map(|u| Member {
                path: "m.py".into(),
                unit: Some((*u).into()),
                lines: (1, 2),
                run: (1, 2),
                tree: base(),
            })
            .collect(),
    }
}

/// One member set, one suggestion: the same two units entering once as
/// a T3 pair and once as a T1/T2 family leave the T1/T2 group alone,
/// counted once; another set is kept, in order.
#[test]
fn one_member_set_is_one_suggestion_and_t1t2_covers_t3() {
    let mut groups = vec![
        pair(FAMILY_NEAR, ["m.py:f#0", "m.py:g#0"]),
        pair(FAMILY_EXACT, ["m.py:g#0", "m.py:f#0"]),
        pair(FAMILY_NEAR, ["m.py:f#0", "m.py:h#0"]),
    ];
    assert_eq!(one_per_member_set(&mut groups), 1, "one dropped");
    let kept: Vec<(u8, Vec<String>)> = groups.iter().map(|g| (g.family, g.member_set())).collect();
    let set = |a: &str, b: &str| vec![a.to_string(), b.to_string()];
    assert_eq!(
        kept,
        [
            (FAMILY_EXACT, set("m.py:f#0", "m.py:g#0")),
            (FAMILY_NEAR, set("m.py:f#0", "m.py:h#0")),
        ]
    );
}

/// Chunks keep each group whole and within both caps, in order.
#[test]
fn a_chunk_never_splits_a_group() {
    let group = |nodes: usize| Group {
        family: FAMILY_EXACT,
        fragment: true,
        members: (0..2)
            .map(|_| Member {
                path: "m.py".into(),
                unit: None,
                lines: (1, 1),
                run: (1, 1),
                tree: tree(&vec![1; nodes], &vec![0; nodes], &vec![0; nodes]),
            })
            .collect(),
    };
    let quarter = TREE_NODE_CAP / 4;
    let groups = [group(quarter), group(quarter), group(quarter), group(1)];
    assert_eq!(
        chunks(&groups),
        vec![0..2, 2..4],
        "two groups fill the node cap"
    );
    let many: Vec<Group> = (0..=GROUP_CAP).map(|_| group(1)).collect();
    assert_eq!(chunks(&many), vec![0..GROUP_CAP, GROUP_CAP..GROUP_CAP + 1]);
    assert!(chunks(&[]).is_empty());
}
