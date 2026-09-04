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

#[test]
fn version_marks_are_read_by_shape() {
    assert!(has_semver("v1.6.0") && has_semver("2.3"));
    assert!(!has_semver("v1") && !has_semver("a.b"));
    assert!(has_iso_date("released 2026-09-04") && !has_iso_date("2026-9-4"));
    assert!(versioned("未发布") && versioned("Unreleased changes"));
    assert_eq!(
        (Witness::Path.name(), Witness::Ledger.name()),
        ("path", "ledger")
    );
}
