use super::*;

#[test]
fn relocation_with_indent_change_is_moved_and_attributed() {
    let before = "def f():\n    x = compute()\n    return x\n\ndef g():\n    pass\n";
    let after = "def f():\n    if ok:\n        x = compute()\n    return x\n\ndef g():\n    pass\n";
    let c = classify(before, after, Lang::Python);
    assert_eq!(c.counts.added_moved, 1, "indented re-add of x=compute()");
    assert_eq!(c.counts.removed_moved, 1);
    assert_eq!(c.counts.added_novel, 1); // the new `if ok:` line
    let m = c.moved.iter().find(|m| !m.removed).unwrap();
    assert_eq!(m.unit.as_deref(), Some("f/0"));
}

#[test]
fn blank_lines_never_move() {
    let before = "a = 1\n\nb = 2\n";
    let after = "b = 2\n\nc = 3\n";
    let c = classify(before, after, Lang::Python);
    assert_eq!(c.counts.added_moved + c.counts.removed_moved, 0);
}

#[test]
fn intact_function_relocation_is_summarized() {
    let before = "def a():\n    return 1\n\ndef b():\n    return 2\n";
    let after = "def b():\n    return 2\n\ndef a():\n    return 1\n";
    let c = classify(before, after, Lang::Python);
    assert!(
        c.relocated_units.contains(&"a/0".to_string())
            || c.relocated_units.contains(&"b/0".to_string()),
        "one of the swapped functions must be reported relocated, got {:?}",
        c.relocated_units
    );
    // The swap repositions one BLANK separator line; per the
    // ground-truth convention blanks never move, so it lands in
    // novel/deleted while both code lines are moved.
    assert_eq!(c.counts.added_novel, 1);
    assert_eq!(c.counts.added_moved, 2);
    assert_eq!(c.counts.removed_moved, 2);
}
