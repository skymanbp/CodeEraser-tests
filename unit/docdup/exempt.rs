use super::*;
use crate::docdup::spec::{KIND_DOCSTRING, KIND_HTML_TEXT, KIND_TEXT_PARA};

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
    // the prose kinds pass through untouched: md, HTML text (step 5)
    // and plain text (step 5b-8) alike
    for kind in [KIND_MD_PARA, KIND_HTML_TEXT, KIND_TEXT_PARA] {
        let prose = seg(kind, 1, &["Args:", "---"]);
        assert_eq!(strip_skeleton(&prose, &mut lg).len(), 2, "kind {kind}");
        assert_eq!(lg.skeleton_line, 2, "kind {kind} untouched");
    }
}

#[test]
fn jsdoc_and_sphinx_markers_match_under_decoration() {
    // the v2.30 languages' conventions (DOCDUP_REV 6): Javadoc and
    // Doxygen tags under `*` and `\`, LDoc under `---`, roxygen under `#'`
    for line in [
        " * @param x the input",
        "    :param x: input",
        "# Returns:",
        " * @return the value",
        " * @throws IOException",
        "/// \\brief one line",
        "--- @tparam string s",
        "-- @treturn number",
        "#' @export",
        "#' @importFrom rlang abort",
    ] {
        assert!(skeleton_line(line), "{line}");
    }
    assert!(!skeleton_line("-- a plain lua comment"));
    assert!(!skeleton_line("/// - Note: a bullet is prose"));
    assert!(!skeleton_line("# - Returns: prose again"));
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

/// A title underline is not a fence (DOCDUP_REV 9): each case is a
/// header line `kind kept skeleton fenced @@ why` over the segment's
/// lines (kind 1 = comment, 2 = docstring, 0 = md paragraph); the
/// counts are the lines `strip_skeleton` keeps and its two ledger rows.
const UNDERLINES: &str = r#"2 5 1 0 @@ the title-and-underline docstring: only the underline drops
"""
requests.api
~~~~~~~~~~~~

This module implements the Requests API.
"""
====
2 2 1 0 @@ a title on the opening line, quotes stripped
"""Title here
~~~~~~~~~~
Body text.
====
1 2 1 0 @@ a title in a comment
# Title
# ~~~~~
# body
====
2 1 1 3 @@ a tilde fence under Example: (shorter than the line above) as before
Example:
~~~
x = 1
~~~
after
====
2 1 0 2 @@ nothing above: a fence as before
`
~~~~
x = 1
====
2 1 0 2 @@ an underline shorter than the title is a fence as before
requests.adapters
~~~~~~~~
text
====
2 0 0 3 @@ inside a backtick fence a tilde line toggles as before
```
Title
~~~~~
====
0 2 0 0 @@ an md paragraph is untouched
Title
~~~~~
====
2 2 2 0 @@ the stated limit: a tilde fence at least as long as the line above reads as an underline
Hi
~~~
code
~~~~"#;

#[test]
fn a_title_underline_is_dropped_as_skeleton_and_opens_no_fence() {
    for case in UNDERLINES.split("\n====\n") {
        let (head, body) = case.split_once('\n').expect("a header");
        let (counts, why) = head.split_once(" @@ ").expect("counts @@ why");
        let want: Vec<u64> = counts.split(' ').map(|n| n.parse().expect("n")).collect();
        let lines: Vec<&str> = body
            .lines()
            .map(|l| if l == "`" { "" } else { l })
            .collect();
        let mut lg = Ledger::default();
        let kept = strip_skeleton(&seg(want[0] as i64, 1, &lines), &mut lg).len() as u64;
        assert_eq!(
            vec![kept, lg.skeleton_line, lg.fenced_code_line],
            want[1..].to_vec(),
            "{why}"
        );
    }
}
