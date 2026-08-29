use super::*;

/// `.ce` is ce's own state at any depth — the prefix-only test let
/// a vendored `vendor/pkg/.ce/index.db` through on both legs.
#[test]
fn ce_owned_matches_any_component_not_just_the_prefix() {
    assert!(ce_owned(".ce/observe.ndjson"));
    assert!(ce_owned("vendor/pkg/.ce/index.db"));
    assert!(!ce_owned("src/.certs/key.rs"), "prefix of a name is not it");
    assert!(!ce_owned("a.rs"));
}

/// The `-z` record grammar, pinned where a filesystem cannot go:
/// Windows refuses to create a tab-bearing path, but git happily
/// emits one from a tree object, and `core.quotePath=false` does
/// NOT unquote it — only -z does. A `splitn(3)` keeps that tab
/// inside the path instead of reading it as a third separator.
#[test]
fn a_numstat_record_keeps_tabs_inside_the_path() {
    let rec = "1\t0\tweird\tname.rs";
    let mut cols = rec.splitn(3, '\t');
    assert_eq!(cols.next(), Some("1"));
    assert_eq!(cols.next(), Some("0"));
    assert_eq!(
        cols.next(),
        Some("weird\tname.rs"),
        "the path is whatever follows the two counts, tabs and all"
    );
    assert_eq!(
        numstat_row(rec),
        Some((1, String::from("weird\tname.rs"))),
        "and the row parser keeps it too"
    );
}

/// The two legs answer in ONE universe. Every row here is one git
/// would print for a real file: without the judged gate on this
/// leg, `app.js` and `ce.toml` counted their full numstat once
/// committed and exactly 0 while untracked.
#[test]
fn numstat_rows_share_the_untracked_legs_judged_universe() {
    assert_eq!(numstat_row("11\t0\ta.rs"), Some((11, "a.rs".into())));
    assert_eq!(
        numstat_row("-\t-\ta.rs"),
        Some((0, "a.rs".into())),
        "binary"
    );
    assert_eq!(numstat_row("3\t2\tapp.js"), None, "scan-only arm");
    assert_eq!(numstat_row("3\t2\tce.toml"), None, "no language at all");
    assert_eq!(numstat_row("1\t0\t.ce/observe.ndjson"), None, "ce's own");
    assert_eq!(numstat_row("1\t0"), None, "malformed rows still skip");
}
