use super::*;
use crate::docdup::spec::KIND_DOCSTRING;

fn seg(kind: i64, start: i64, lines: &[&str]) -> RawSeg {
    RawSeg {
        kind,
        start_line: start,
        end_line: start + lines.len() as i64 - 1,
        lines: lines
            .iter()
            .map(|t| SegLine {
                text: (*t).to_string(),
                mask: None,
            })
            .collect(),
    }
}

#[test]
fn license_needs_first_block_and_head_window() {
    let lic = seg(1, 1, &["// Licensed under the Apache License"]);
    let mut lg = Ledger::default();
    assert_eq!(classify(&lic, true, &mut lg), EXEMPT_LICENSE);
    assert_eq!(classify(&lic, false, &mut lg), EXEMPT_LIVE);
    let late = seg(1, 40, &["// Licensed under the Apache License"]);
    assert_eq!(classify(&late, true, &mut lg), EXEMPT_LIVE);
    assert_eq!(lg.license_header, 1);
}

#[test]
fn allow_without_why_is_a_ledgered_violation_not_an_exemption() {
    let mut lg = Ledger::default();
    let ok = seg(1, 10, &["# ce:allow(docdup) -- generated table"]);
    assert_eq!(classify(&ok, false, &mut lg), EXEMPT_ALLOW);
    let bare = seg(1, 10, &["# ce:allow(docdup)"]);
    assert_eq!(classify(&bare, false, &mut lg), EXEMPT_LIVE);
    let empty_why = seg(1, 10, &["# ce:allow(docdup) -- "]);
    assert_eq!(classify(&empty_why, false, &mut lg), EXEMPT_LIVE);
    assert_eq!((lg.inline_allow, lg.allow_missing_why), (1, 2));
}

#[test]
fn skeleton_lines_strip_from_docstrings_but_not_md() {
    let mut lg = Ledger::default();
    let ds = seg(
        KIND_DOCSTRING,
        1,
        &[
            "\"\"\"Fetch.",
            "Args:",
            "    x: input",
            "Returns:",
            "\"\"\"",
        ],
    );
    let kept = strip_skeleton(&ds, &mut lg);
    assert_eq!(kept.len(), 3);
    assert_eq!(lg.skeleton_line, 2);
    let md = seg(KIND_MD_PARA, 1, &["Args:", "---"]);
    assert_eq!(strip_skeleton(&md, &mut lg).len(), 2);
    assert_eq!(lg.skeleton_line, 2, "md untouched");
}

#[test]
fn jsdoc_and_sphinx_markers_match_under_decoration() {
    for line in [" * @param x the input", "    :param x: input", "# Returns:"] {
        assert!(skeleton_line(line), "{line}");
    }
    assert!(!skeleton_line("returns the cached value"));
    assert!(skeleton_line(" * ----"));
}

/// Seeded counterfactual for the 2026-08-14 amendment's comment
/// half: a rustdoc fenced doctest (the audited ripgrep FP shape)
/// and an overlong regex line (the audited zod FP shape) strip
/// with their ledger counts; prose around them survives; an
/// UNCLOSED fence strips to segment end.
#[test]
fn fenced_and_overlong_comment_lines_strip_with_ledger() {
    let mut lg = Ledger::default();
    let long = format!("// const rx = /{}/;", "a|".repeat(150));
    let c = seg(
        1,
        10,
        &[
            "/// This crate provides printers.",
            "/// ```rust",
            "/// let x = search();",
            "/// ```",
            "/// More prose after the example.",
            &long,
        ],
    );
    let kept = strip_skeleton(&c, &mut lg);
    assert_eq!(kept.len(), 2, "prose survives, code and regex do not");
    assert_eq!(lg.fenced_code_line, 3);
    assert_eq!(lg.overlong_line, 1);
    let unclosed = seg(1, 1, &["// prose", "// ```", "// code one", "// code two"]);
    assert_eq!(strip_skeleton(&unclosed, &mut lg).len(), 1);
    assert_eq!(lg.fenced_code_line, 6, "unclosed fence strips to end");
    let md = seg(KIND_MD_PARA, 1, &[&long]);
    assert_eq!(strip_skeleton(&md, &mut lg).len(), 1, "md untouched");
}
