//! survival.rs's pins: which blame record counts as a line the window
//! added, under which (commit, path) it is joined, and how a pair's
//! blame lines split into survivors and lines beyond the ledger.

use std::collections::{HashMap, HashSet};

const IN: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678";
const OUT: &str = "0000000000000000000000000000000000000abc";

/// One `--line-porcelain` record for a REBASED line (old author date,
/// fresh committer date) whose commit is `sha` at `file`, plus a
/// content line that spells a record header — it arrives TAB-prefixed,
/// so it can never be read as one.
fn record(sha: &str, file: &str) -> String {
    format!(
        "{sha} 1 1 1\nauthor A\nauthor-time 1000\ncommitter-time 900000\n\
         previous {OUT} a.rs\nsummary rebased forward\nfilename {file}\n\t{IN} 2 2\n"
    )
}

/// (record's commit, its filename, the root's prefix) → the key it is
/// counted under, or none: a record counts iff its commit is in the
/// window set — dates play no part — and is keyed by its path in that
/// commit, relative to the root; a path outside the root is keyed "".
/// A C-quoted filename is read back to its bytes. One table: pasted
/// assert stanzas would mint a clone block against the dedup budget.
#[test]
fn records_join_per_window_commit_and_root_path() {
    let window: HashSet<&str> = [IN].into_iter().collect();
    let quoted = r#""q\"\\\t\303\251.rs""#;
    let rows = [
        (IN, "a.rs", "", Some("a.rs")),
        (OUT, "a.rs", "", None),
        (IN, "sub/a.rs", "sub/", Some("a.rs")),
        (IN, "top.rs", "sub/", Some("")),
        (IN, quoted, "", Some("q\"\\\té.rs")),
    ];
    for (sha, file, prefix, want) in rows {
        let mut got = HashMap::new();
        super::blamed_lines(&record(sha, file), &window, prefix, &mut got);
        let want: HashMap<(String, String), usize> = want
            .map(|p| ((IN.to_string(), p.to_string()), 1))
            .into_iter()
            .collect();
        assert_eq!(got, want, "record of {sha} at {file} under {prefix:?}");
    }
}

/// A pair's survivors are its blame lines capped at the ledger's added
/// lines; the rest — and every line of a pair the ledger never read —
/// is beyond the ledger, so survivors never exceed the added lines.
#[test]
fn survivors_cap_at_the_ledger_and_the_rest_is_counted() {
    let key = |p: &str| (IN.to_string(), p.to_string());
    let blamed = HashMap::from([(key("a.rs"), 5), (key("b.rs"), 1), (key("c.rs"), 4)]);
    let added = HashMap::from([(key("a.rs"), 2), (key("b.rs"), 3)]);
    let got = super::settle(&blamed, &added);
    let want = super::Survival {
        surviving: 3,
        beyond_ledger: 7,
    };
    assert_eq!(got, want);
}
