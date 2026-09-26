//! The 3d exit criterion, mechanized: text inside fences, inline
//! code and HTML comments yields ZERO segment words, and the
//! extractor demonstrably rides the detector's one mask.

use super::*;
use crate::docdup::shingle;
use crate::docdup::spec::KIND_HTML_TEXT;

fn md_segs(text: &str) -> Vec<RawSeg> {
    extract(text, Lang::Markdown).0
}

fn seg_words(seg: &RawSeg) -> Vec<u64> {
    let mut out = Vec::new();
    for l in &seg.lines {
        shingle::line_words(&l.text, l.mask.as_deref(), &mut out);
    }
    out
}

/// (kind, start, end) rows of one source — the assertion currency of
/// every geometry test below.
fn rows(src: &str, lang: Lang) -> Vec<(i64, i64, i64)> {
    extract(src, lang)
        .0
        .iter()
        .map(|s| (s.kind, s.start_line, s.end_line))
        .collect()
}

/// The (start, end) spans alone — the geometry legs that assert
/// adjacency, not kind.
fn spans(src: &str, lang: Lang) -> Vec<(i64, i64)> {
    rows(src, lang)
        .into_iter()
        .map(|(_, a, b)| (a, b))
        .collect()
}

#[test]
fn fenced_inline_and_html_comment_text_is_invisible() {
    let text = "para alpha beta\n\n```\nfence secret\n```\n\ngamma `inline secret` delta\n\n<!-- comment secret -->\n";
    let segs = md_segs(text);
    let all: Vec<u64> = segs.iter().flat_map(seg_words).collect();
    let mut secret = Vec::new();
    shingle::line_words("secret", None, &mut secret);
    assert!(!all.contains(&secret[0]), "masked text leaked into words");
    // the fence body and the pure-comment line form no segment at all
    assert_eq!(segs.len(), 2, "alpha/beta and gamma/delta paragraphs");
}

#[test]
fn headings_tables_and_bare_markers_break_paragraphs() {
    let text = "one two\n# Head\nthree four\n| a | b |\nfive six\n-\nseven eight\n- item text\n";
    // each excluded line severs adjacency; the list item WITH text is
    // content and joins the preceding run
    assert_eq!(
        spans(text, Lang::Markdown),
        [(1, 1), (3, 3), (5, 5), (7, 8)]
    );
}

#[test]
fn comment_runs_merge_by_column_and_docstrings_extract() {
    let py = "\"\"\"module doc\nsecond line\n\"\"\"\n# one\n# two\nx = 1\n# lone\n\ndef f():\n    \"\"\"fn doc\"\"\"\n    return 1  # trailing\n";
    assert_eq!(
        rows(py, Lang::Python),
        [
            (KIND_DOCSTRING, 1, 3),   // module docstring
            (KIND_COMMENT, 4, 5),     // merged # run
            (KIND_COMMENT, 7, 7),     // lone comment
            (KIND_DOCSTRING, 10, 10), // fn docstring
            (KIND_COMMENT, 11, 11),   // trailing comment
        ]
    );
}

#[test]
fn rust_block_and_line_comments_extract() {
    let rs = "/* block\n   comment */\nfn main() {\n    // a\n    // b\n}\n";
    assert_eq!(
        rows(rs, Lang::Rust),
        [(KIND_COMMENT, 1, 2), (KIND_COMMENT, 4, 5)]
    );
}

/// DOCDUP_REV 5 (v2.28 amendment): tree-sitter-rust's doc-comment node
/// ends at column 0 of the NEXT row, so before the fix every `///`
/// line was its own segment ending one row late — (1, 2), (2, 3),
/// (3, 4) for the three-line `///` block. A run is one paragraph
/// whatever its marker, a `///` run continued by `//` at the same
/// column is still one run, and a column change breaks it.
#[test]
fn rust_doc_comment_runs_merge_like_plain_runs() {
    let cases: [(&str, &[(i64, i64)]); 3] = [
        ("//! crate doc\n//! second\n\nfn f() {}\n", &[(1, 2)]),
        ("/// a\n/// b.\n/// c\nfn f() {}\n", &[(1, 3)]),
        ("/// d\n// e\n    // f\nfn g() {}\n", &[(1, 2), (3, 3)]),
    ];
    for (src, want) in cases {
        assert!(rows(src, Lang::Rust).iter().all(|r| r.0 == KIND_COMMENT));
        assert_eq!(spans(src, Lang::Rust), want, "{src:?}");
    }
}

/// Seeded counterfactual for the 2026-08-14 amendment's md half: a
/// sponsor-table / badge-strip block (every line HTML markup — the
/// audited zod FP shape) yields ZERO segments and a nonzero
/// html_line count, while a single LONG line of genuine md prose
/// (past DOC_LINE_CAP) survives untouched — the false-kill guard.
#[test]
fn html_markup_lines_shed_but_long_md_prose_survives() {
    let table = "<table align=\"center\">\n  <tr>\n    <td align=\"center\">\n      \
                 <img src=\"logo.svg\" alt=\"x\" />\n    </td>\n  </tr>\n</table>\n";
    let (segs, shed) = extract(table, Lang::Markdown);
    assert!(segs.is_empty(), "HTML block must form no md_para");
    assert_eq!(shed.html, 7, "every markup line ledgered");
    let prose = format!("{}\n", "word ".repeat(60).trim());
    assert!(prose.len() > crate::docdup::spec::DOC_LINE_CAP);
    let (segs, shed) = extract(&prose, Lang::Markdown);
    assert_eq!(segs.len(), 1, "unwrapped md prose is content");
    assert_eq!(shed.html, 0);
}

/// Plan v2.30 step 5 (register D20): an HTML block element is one
/// html_text segment over its SOURCE rows — a paragraph wrapping onto
/// a second line spans both — masked to its own text leaves: markup,
/// attributes, entities and comments yield no words, an inline element
/// hands its text up, a nested block is a segment of its own whose
/// text is not the parent's, `pre` / `code` and script content are
/// shed and counted, and two cells on one line are two segments over
/// the same row.
#[test]
fn html_blocks_segment_over_their_rows_under_a_text_mask() {
    let html = "<body>\n<p class=\"lead\">alpha &amp; <b>beta</b>\n gamma</p>\n<ul><li>one <p>inner secret</p></li></ul>\n<pre>fence secret</pre>\n<script>script secret</script>\n<table><tr><td>cell one</td><td>cell two</td></tr></table>\n<p>delta <code>code secret</code> <!-- comment secret --></p>\n</body>\n";
    let (segs, shed) = extract(html, Lang::Html);
    assert!(segs.iter().all(|s| s.kind == KIND_HTML_TEXT));
    assert_eq!(
        spans(html, Lang::Html),
        [(2, 3), (4, 4), (4, 4), (7, 7), (7, 7), (8, 8)]
    );
    let words = |text: &str| {
        let mut out = Vec::new();
        shingle::line_words(text, None, &mut out);
        out
    };
    let got: Vec<Vec<u64>> = segs.iter().map(seg_words).collect();
    let want = [
        "alpha beta gamma",
        "one",
        "inner secret",
        "cell one",
        "cell two",
        "delta",
    ];
    assert_eq!(got, want.map(words));
    assert_eq!(
        (shed.code, shed.script),
        (2, 1),
        "pre + code shed, script shed"
    );
}

/// Booklet §9: a `ce:allow(docdup) -- why` marker written in an HTML
/// comment inside the element rides the segment's raw row — the one
/// claim grammar (crate::allow) reads it unchanged, exactly as it reads
/// the marker at the end of a README line — while the comment's words
/// stay masked out of the prose. A marker on a row the element does not
/// span exempts nothing.
#[test]
fn an_html_comment_carries_the_allow_marker() {
    use crate::docdup::exempt::{EXEMPT_ALLOW, EXEMPT_LIVE, Ledger, classify};
    let html = "<!-- ce:allow(docdup) -- a marker above the element -->
<p>one set of links <!-- ce:allow(docdup) -- listed in both languages --></p>
<p>two sets</p>
";
    let (segs, _) = extract(html, Lang::Html);
    let mut ledger = Ledger::default();
    let verdicts: Vec<i64> = segs
        .iter()
        .map(|s| classify(s, false, &mut ledger))
        .collect();
    assert_eq!(verdicts, [EXEMPT_ALLOW, EXEMPT_LIVE]);
    let mut prose = Vec::new();
    shingle::line_words("one set of links", None, &mut prose);
    assert_eq!(seg_words(&segs[0]), prose, "the comment is no prose");
}
