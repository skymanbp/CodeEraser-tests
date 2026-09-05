use super::*;

fn units(with_impl_a: bool, twice_under_b: bool) -> Vec<(String, i64, i64)> {
    let mut v = vec![
        ("impl B".to_string(), 10, 30),
        ("add/1".to_string(), 12, 14),
    ];
    if twice_under_b {
        v.push(("add/1".into(), 20, 22));
    }
    if with_impl_a {
        v.insert(0, ("impl A".into(), 1, 8));
        v.insert(1, ("add/1".into(), 3, 5));
    }
    v.push(("lonely/0".into(), 40, 41));
    v
}

fn anchor_at(table: &[Anchored], start: i64) -> String {
    table
        .iter()
        .find(|u| u.start == start)
        .map(|u| u.anchor.clone())
        .unwrap_or_else(|| panic!("no unit starts at {start}"))
}

/// The §7.2 counterfactuals. Two same-key methods under different
/// impls are told apart by their impl; deleting impl A and its `add`
/// leaves impl B's `add` at the anchor it had — the shift `nth`
/// suffered is gone — and leaves the top-level `lonely` alone too.
/// Two same-key units under ONE container are the residual case and
/// differ only by their order under it; and the order the units
/// arrive in changes nothing.
#[test]
fn anchors_follow_containers_not_positions() {
    let both = anchored(units(true, false));
    let b_only = anchored(units(false, false));
    assert_ne!(
        anchor_at(&both, 3),
        anchor_at(&both, 12),
        "impl A vs impl B"
    );
    assert_eq!(
        anchor_at(&both, 12),
        anchor_at(&b_only, 12),
        "deleting the sibling moves nothing"
    );
    assert_eq!(
        anchor_at(&both, 40),
        anchor_at(&b_only, 40),
        "top level untouched"
    );
    assert!(
        anchor_at(&both, 40).ends_with("#0"),
        "one unit under its chain"
    );

    let redefined = anchored(units(false, true));
    assert_ne!(anchor_at(&redefined, 12), anchor_at(&redefined, 20));
    assert!(
        anchor_at(&redefined, 20).ends_with("#1"),
        "order under one chain"
    );

    let mut shuffled = units(true, true);
    shuffled.reverse();
    let reversed = anchored(shuffled);
    for u in anchored(units(true, true)) {
        assert_eq!(
            anchor_at(&reversed, u.start),
            u.anchor,
            "order-free at {}",
            u.start
        );
    }
}

/// `owner` is the innermost unit holding the WHOLE span (the join's
/// UnitMap stance): a block inside `add` belongs to `add`, a block
/// crossing two units belongs to the file's top level, and an unknown
/// path answers the top level too rather than guessing.
#[test]
fn owner_is_innermost_whole_span_or_top_level() {
    let table = Anchors {
        by_file: [("a.rs".to_string(), anchored(units(true, false)))]
            .into_iter()
            .collect(),
    };
    let (path, key, anchor) = table.owner("a.rs", 3, 4);
    assert_eq!((path.as_str(), key.as_str()), ("a.rs", "add/1"));
    assert_eq!(anchor, anchor_at(&anchored(units(true, false)), 3));
    assert_eq!(
        table.owner("a.rs", 2, 6).1,
        "impl A",
        "whole impl holds a wider span"
    );
    assert_eq!(
        table.owner("a.rs", 5, 12),
        ("a.rs".into(), String::new(), String::new())
    );
    assert_eq!(table.owner("zz.rs", 1, 1).1, "", "unknown file: top level");
    assert_eq!(
        table.of("a.rs", "add/1", 12, 14, 0),
        Some(anchor_at(&anchored(units(true, false)), 12).as_str())
    );
    assert_eq!(
        table.of("a.rs", "add/1", 12, 15, 0),
        None,
        "exact span only"
    );
    // two closures on one line: same key, same span, two units — the
    // k-th on the scanner's side is the k-th here, and a third is none
    let twins = Anchors {
        by_file: [(
            "b.rs".to_string(),
            anchored(vec![
                ("outer/0".into(), 1, 9),
                ("(anonymous)/1".into(), 5, 5),
                ("(anonymous)/1".into(), 5, 5),
            ]),
        )]
        .into_iter()
        .collect(),
    };
    let (first, second) = (
        twins.of("b.rs", "(anonymous)/1", 5, 5, 0).expect("first"),
        twins.of("b.rs", "(anonymous)/1", 5, 5, 1).expect("second"),
    );
    assert_ne!(first, second, "same line, two identities");
    assert!(first.ends_with("#0") && second.ends_with("#1"));
    assert_eq!(twins.of("b.rs", "(anonymous)/1", 5, 5, 2), None);
}
