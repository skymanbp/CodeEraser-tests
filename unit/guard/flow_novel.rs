use super::*;

fn p(unit: &str, kind: u8, line: u32, var: Option<&str>) -> Placed {
    Placed {
        unit: unit.into(),
        nth: 0,
        kind,
        line,
        line_end: line,
        var: var.map(String::from),
    }
}

/// A finding that only moved (lines added above it) is the same
/// finding: the key has no line, so nothing is novel.
#[test]
fn a_moved_finding_is_not_novel() {
    let before = [p("f", 1, 4, Some("x"))];
    let after = [p("f", 1, 9, Some("x"))];
    assert!(novel(&before, &after).is_empty());
}

/// A renamed variable, or the same finding in a renamed unit, is a new
/// key and so a new finding; each before finding cancels one after.
#[test]
fn a_rename_is_novel_and_the_difference_is_a_multiset() {
    let before = [p("f", 1, 4, Some("x")), p("f", 0, 7, None)];
    let after = [
        p("f", 1, 4, Some("y")),
        p("g", 0, 7, None),
        p("f", 0, 8, None),
        p("f", 0, 9, None),
    ];
    let got: Vec<(String, u32)> = novel(&before, &after)
        .into_iter()
        .map(|q| (q.unit.clone(), q.line))
        .collect();
    assert_eq!(got, [("f".into(), 4), ("g".into(), 7), ("f".into(), 9)]);
}

/// The difference keeps every kind, the advisory one included: the
/// guard leg, not this subtraction, decides which kinds make the
/// condition.
#[test]
fn the_subtraction_is_kind_blind() {
    let after = [p("f", 3, 1, Some("a"))];
    assert_eq!(novel(&[], &after).len(), 1);
    assert_eq!(key(&after[0]), ("f".to_string(), 3, "a".to_string()));
}
