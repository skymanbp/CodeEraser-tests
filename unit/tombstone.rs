use super::*;
use std::collections::BTreeSet;
use std::path::Path;

/// A changeset pair from its four parts — the one constructor the
/// hub, names and surfaces tests share (three local copies were the
/// clone gate's first catch on this suite).
pub(super) fn pair<'a>(rel: &'a str, before: &'a str, after: &'a str, lang: Lang) -> PairText<'a> {
    PairText {
        rel,
        before,
        after,
        lang,
    }
}

/// A changeset measured with nothing declared in ce.toml.
pub(super) fn plain(pairs: &[PairText], session: &BTreeSet<u64>) -> Findings {
    measure(pairs, session, &Policy::default())
}

pub(super) fn none() -> BTreeSet<u64> {
    BTreeSet::new()
}

/// The exemptions as rows: (file, segment start, witness, tokens).
fn exempt_rows(f: &Findings) -> Vec<(String, Option<usize>, Witness, usize)> {
    f.exempt
        .iter()
        .map(|e| (e.file.clone(), e.line, e.why, e.tokens))
        .collect()
}

/// The candidate rows as the wire will see them, with their line:
/// (line, kind, marks, binds an erased name).
pub(super) fn rows(f: &Findings) -> Vec<(usize, Kind, usize, bool)> {
    f.rows
        .iter()
        .map(|r| (r.line, r.kind, r.marks, r.names > 0))
        .collect()
}

/// A judgment naming every row a site — what the feed renders; the
/// conjunction itself is the core's (contracts/fixtures/tombstone).
/// The policy a `[tombstone]` table declares, as the hooks build it.
pub(super) fn declared(table: &str) -> Policy {
    Policy::of(Path::new("."), &toml::from_str(table).unwrap())
}

fn judged_all(f: &Findings, label: usize, prose: usize) -> Judgment {
    Ok(Judged {
        sites: (0..f.rows.len()).collect(),
        label,
        prose,
        over: false,
    })
}

#[test]
fn a_heading_that_frames_the_erased_name_is_a_label_row() {
    let after = "# Tomato and Egg (no Dongpo Pork)\n\nStir.\n";
    let pairs = [pair(
        "recipes.md",
        "# Dongpo Pork\n\nBraise.\n",
        after,
        Lang::Markdown,
    )];
    let f = plain(&pairs, &none());
    assert_eq!(rows(&f), [(1, Kind::Bracketed, 0, true)]);
    let r = &f.rows[0];
    assert_eq!(
        (
            r.file.as_str(),
            r.name.as_str(),
            r.excerpt.as_str(),
            r.ledger
        ),
        ("recipes.md", "dongpo", "Tomato and Egg (no Dongpo Pork)", 0)
    );
}

#[test]
fn a_ledger_segment_exempts_its_own_surfaces_only() {
    // the third witness (plan v2.27): a banner that is a version
    // ledger by itself is exempt as a segment and counted once; the
    // section below it, with one version in it, is a row as before
    let banner = "> **v1.5.1** 2026-09-02 47efc44 · v1.5.0 2026-09-01 · v1.4.1 65928ac · \
                  dongpo is no longer braised.\n> a second banner line.\n";
    let after =
        format!("# Plan\n\n{banner}\n## Sides (no dongpo)\n\nSince 1.6.0 they are stir-fried.\n");
    let pairs = [pair(
        "plan.md",
        "# Plan\n\n## dongpo\n",
        &after,
        Lang::Markdown,
    )];
    let f = plain(&pairs, &none());
    assert_eq!(
        exempt_rows(&f),
        [("plan.md".to_string(), Some(3), Witness::Segment, 7)]
    );
    assert_eq!(rows(&f), [(6, Kind::Bracketed, 0, true)]);
    assert_eq!(f.rows[0].ledger, 1);
    assert_eq!(
        feed_json(&f, None, &judged_all(&f, 1, 0))["exempt"][0],
        serde_json::json!({"file": "plan.md", "line": 3, "why": "segment"})
    );
    // a code file narrates nothing by job: the same ledger in a
    // comment exempts nothing, and the sentence is a prose row
    let rs = "// ledger: v1.5.1 2026-09-02 47efc44 v1.5.0 2026-09-01 v1.4.1 65928ac\n\
              // braise_dongpo_pork is no longer used.\nfn cook() {}\n";
    let f = plain(
        &[pair("k.rs", "fn braise_dongpo_pork() {}\n", rs, Lang::Rust)],
        &none(),
    );
    assert_eq!(rows(&f), [(1, Kind::Prose, 1, true)]);
    assert_eq!((f.exempt.len(), f.rows[0].ledger), (0, 0));
}

#[test]
fn an_identifier_and_a_docstring_are_the_bare_and_prose_rows() {
    let before = "fn braise_dongpo_pork() {}\n";
    let after = "/// This recipe no longer uses braise_dongpo_pork.\nfn cook_without_dongpo() {}\n";
    let f = plain(&[pair("kitchen.rs", before, after, Lang::Rust)], &none());
    assert_eq!(
        rows(&f),
        [(2, Kind::Bare, 0, true), (1, Kind::Prose, 1, true)]
    );
}

#[test]
fn a_mark_alone_or_a_name_alone_is_a_row_the_core_will_not_seat() {
    // the conjunction is the core's: rows [2,1,0] and [2,0,1] are sent
    // and judged no site (contracts/fixtures/tombstone/golden.ndjson,
    // pair 1); this side only refuses to send a sentence with neither
    let before = "fn braise_dongpo_pork() {}\n";
    let mark_only = "/// This recipe no longer needs a wok.\nfn cook() {}\n";
    let name_only = "/// See braise_dongpo_pork in the old cookbook.\nfn cook() {}\n";
    // read per sentence: a mark in one and the name in the next is
    // two sentences about two things — two rows, each one-sided
    let split = "/// We no longer simmer. See braise_dongpo_pork for the old way.\nfn cook() {}\n";
    let shapes = [
        (mark_only, vec![(1, Kind::Prose, 1, false)]),
        (name_only, vec![(1, Kind::Prose, 0, true)]),
        (
            split,
            vec![(1, Kind::Prose, 1, false), (1, Kind::Prose, 0, true)],
        ),
    ];
    for (after, expect) in shapes {
        let f = plain(&[pair("kitchen.rs", before, after, Lang::Rust)], &none());
        assert_eq!(rows(&f), expect, "{after}");
        assert!(!f.erased.is_empty(), "the name was erased all the same");
    }
    let stir = "/// Stir.\nfn cook() {}\n";
    let f = plain(&[pair("kitchen.rs", before, stir, Lang::Rust)], &none());
    assert!(f.rows.is_empty(), "neither a mark nor a name: not a row");
}

#[test]
fn a_changelog_is_exempt_whole_and_counted() {
    let pairs = [
        pair("src/k.rs", "fn braise_dongpo_pork() {}\n", "", Lang::Rust),
        pair(
            "CHANGELOG.md",
            "# Changelog\n",
            "# Changelog\n\n## Unreleased\n\n- Removed braise_dongpo_pork; pork is no longer braised.\n",
            Lang::Markdown,
        ),
    ];
    let f = plain(&pairs, &none());
    assert!(f.rows.is_empty(), "{:?}", f.rows);
    assert_eq!(
        exempt_rows(&f),
        [("CHANGELOG.md".to_string(), None, Witness::Path, 0)]
    );
}

/// `[tombstone] ledger` exempts a file whole by the repository's own
/// word — a code file too — and counts it `declared`; a word in
/// `terms` spells no name, whole or inside a compound, while the
/// compound's other word still does.
#[test]
fn the_table_declares_ledgers_and_vocabulary() {
    let policy = declared("[tombstone]\nledger = [\"notes/\"]\nterms = [\"Pork\"]\n");
    let pairs = [
        pair(
            "kitchen.rs",
            "fn braise_pork() {}\nfn stew_beef() {}\n",
            "fn stew_beef() {}\n",
            Lang::Rust,
        ),
        pair(
            "notes/log.rs",
            "",
            "/// This recipe no longer uses braise_pork.\n",
            Lang::Rust,
        ),
        pair(
            "notes/a.md",
            "",
            "# Removed\n\nSee the log.\n",
            Lang::Markdown,
        ),
    ];
    let f = measure(&pairs, &none(), &policy);
    let erased: Vec<&str> = f.erased.iter().map(|n| n.text.as_str()).collect();
    assert_eq!(
        erased,
        ["braise"],
        "pork is vocabulary, whole or in a compound"
    );
    assert!(f.rows.is_empty(), "{:?}", f.rows);
    assert_eq!(
        exempt_rows(&f),
        [
            ("notes/log.rs".into(), None, Witness::Declared, 0),
            ("notes/a.md".into(), None, Witness::Declared, 0),
        ]
    );
    assert_eq!(
        feed_json(&f, None, &judged_all(&f, 0, 0))["exempt"][0]["why"],
        "declared"
    );
    let bare = plain(&pairs, &none());
    assert!(
        !bare.rows.is_empty() && bare.exempt.is_empty(),
        "undeclared, the notes are rows"
    );
}

#[test]
fn a_name_erased_earlier_in_the_session_still_binds() {
    let session = BTreeSet::from([names::key("dongpo")]);
    let pairs = [pair(
        "r.md",
        "# Menu\n",
        "# Menu\n\n## Sides (no dongpo)\n",
        Lang::Markdown,
    )];
    let f = plain(&pairs, &session);
    assert_eq!(rows(&f), [(3, Kind::Bracketed, 0, true)]);
    assert!(f.erased.is_empty(), "this edit erased nothing itself");
    assert!(
        plain(&pairs, &none()).rows.is_empty(),
        "without the session there is nothing to bind"
    );
}

#[test]
fn the_feed_object_carries_counts_keys_and_the_judgment_but_no_name() {
    let pairs = [pair(
        "recipes.md",
        "# Dongpo Pork\n",
        "# Tomato (no Dongpo Pork)\n",
        Lang::Markdown,
    )];
    let f = plain(&pairs, &none());
    let j = feed_json(&f, Some(3), &judged_all(&f, 1, 0));
    // the site is spelled from parts: a literal `file:line kind` would
    // read as a citation of that page to the source-citation gate
    let place = format!("{}:{} bracketed", "recipes.md", 1);
    assert_eq!(
        j["judged"],
        serde_json::json!({"sites": [place], "label": 1, "prose": 0, "over": false})
    );
    let counts = (
        j["rev"] == TOMBSTONE_REV,
        j["rows"] == 1,
        j["session_erased"] == 3,
        j["erased"] == f.erased.len(),
        j["erased_hashes"].as_array().map(Vec::len) == Some(f.erased.len()),
    );
    assert_eq!(counts, (true, true, true, true, true), "{j}");
    let text = j.to_string();
    assert!(!text.to_lowercase().contains("dongpo"), "{text}");
    // no verdict is a named non-judgment beside the measurement
    let degraded = feed_json(&f, None, &Err("core unavailable".into()));
    assert_eq!(
        degraded["judged"],
        serde_json::json!({"degraded": "core unavailable"})
    );
    assert_eq!(degraded["rows"], 1);
}

#[test]
fn sites_are_capped_while_the_counts_stay_exact() {
    let after: String = (0..12)
        .map(|i| format!("# Sides {i} (no dongpo)\n"))
        .collect();
    let pairs = [pair("r.md", "# dongpo\n", &after, Lang::Markdown)];
    let f = plain(&pairs, &none());
    assert_eq!(f.rows.len(), 12);
    let j = feed_json(&f, None, &judged_all(&f, 12, 0));
    assert_eq!(
        j["judged"]["sites"].as_array().map(Vec::len),
        Some(SITE_CAP)
    );
    assert_eq!(j["judged"]["label"], 12);
}
