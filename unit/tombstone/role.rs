use super::*;

#[test]
fn the_path_convention_is_a_witness() {
    for rel in [
        "CHANGELOG.md",
        "docs/History.md",
        "docs/adr/0007-drop-pork.md",
        "notes/ADR-3.md",
        "release-notes/1.5.md",
    ] {
        assert_eq!(
            changelog_role(rel, "", Lang::Markdown),
            Some(Witness::Path),
            "{rel}"
        );
    }
    assert_eq!(changelog_role("README.md", "", Lang::Markdown), None);
    assert_eq!(
        changelog_role("CHANGELOG.rs", "", Lang::Rust),
        None,
        "only a document holds the role"
    );
}

#[test]
fn a_version_indexed_ledger_is_a_witness() {
    let ledger =
        "# Notes\n\n## 1.5.1\n\n- fix\n\n## 2026-09-01\n\n- more\n\n## Unreleased\n\ntext\n";
    assert_eq!(
        changelog_role("docs/notes.md", ledger, Lang::Markdown),
        Some(Witness::Ledger)
    );
    let guide = "# Guide\n\n## Install\n\n## Use\n\n## 1.0 compatibility\n";
    assert_eq!(
        changelog_role("docs/guide.md", guide, Lang::Markdown),
        None,
        "one heading of three"
    );
    let short = "# Notes\n\n## 1.5.1\n";
    assert_eq!(
        changelog_role("docs/short.md", short, Lang::Markdown),
        None,
        "fewer than three headings"
    );
}

/// A heading indexes a version only through the reader the segment
/// witness uses (v2.35 V4): three numeric parts, or two behind `v`. A
/// section number, a `§` reference or a language version is not one,
/// so a design booklet numbered `### 4.1 …` is no ledger.
#[test]
fn version_marks_are_read_by_shape() {
    for yes in [
        "1.6.0",
        "[v1.9.2] — fix",
        "v2.3 notes",
        "2026-09-04",
        "未发布",
        "Unreleased changes",
    ] {
        assert!(versioned(yes), "{yes}");
    }
    for no in [
        "4.1 语言",
        "§4.2 the gate",
        "Python 3.12",
        "v1",
        "a.b",
        "2026-9-4",
    ] {
        assert!(!versioned(no), "{no}");
    }
    let booklet = "# Track\n\n## 4.1 Rows\n\n### 4.2 Cols\n\n## 4.3 Caps\n\n## Notes\n";
    assert_eq!(
        changelog_role("docs/track.md", booklet, Lang::Markdown),
        None
    );
    assert_eq!(
        (
            Witness::Path.name(),
            Witness::Ledger.name(),
            Witness::Segment.name(),
            Witness::Declared.name()
        ),
        ("path", "ledger", "segment", "declared")
    );
}

#[test]
fn ledger_tokens_are_versions_dates_and_commits_read_once() {
    // v1.5.1 and 1.5.1 are one version; the run number, the section
    // sign, a percentage, a line span and hex-looking words are none
    let text = "v1.5.1 and 1.5.1 again, 6.6.0, v2.26, 2026-09-04, 47efc44, 70DFEBB; \
                §4.2 0.57 % 1.4.x Cost.hs:102-103 33911794800 deadbeef defaced 2026-9-4";
    assert_eq!(ledger_tokens(text), 6);
    assert_eq!(
        ledger_tokens("§4.2 sets 0.57 % over 1.4.x; see Cost.hs:102-103 and run 33911794800"),
        0
    );
    assert_eq!(
        ledger_tokens("自 v1.5.1（2026-09-02，47efc44）起"),
        3,
        "CJK punctuation splits"
    );
    assert_eq!(
        ledger_tokens("bind 10.0.0.1 or 1.2.3.4"),
        0,
        "four parts are an address, not a release"
    );
}

#[test]
fn the_segment_is_the_quote_run_or_the_section_body_around_a_line() {
    let doc = "# Title\n\n> v1.0.0 2026-01-01 aaaaaa1\n> v1.1.0\n\nintro 1.2.3\n\n## A\n\nv2.0.0 text\n\n\
               ### A.1\n\n2026-02-02 deeper\n\n## B\n\nv3.0.0\n";
    // (line asked, first line of its segment, distinct tokens, why)
    let cases = [
        (3, 3, 4, "the quote run, lines 3-4"),
        (4, 3, 4, "the same run from its second line"),
        (6, 1, 1, "the preamble body, its quote run excluded"),
        (8, 8, 1, "a heading line is in its own body"),
        (10, 8, 1, "a body ends at the next heading of any level"),
        (14, 12, 1, "a level-3 body"),
        (18, 16, 1, "the last body runs to EOF"),
    ];
    for (line, start, tokens, why) in cases {
        assert_eq!(segment(doc, line), (start, tokens), "{why}");
    }
    assert_eq!(segment("plain 1.0.0\n", 1), (1, 1), "no heading at all");
    assert_eq!(
        segment("# T\n\n```\nx 1.0.0\n```\n", 4),
        (4, 0),
        "a fenced line is its own empty segment"
    );
}
