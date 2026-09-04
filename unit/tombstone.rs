use super::*;
use std::collections::BTreeSet;

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

fn none() -> BTreeSet<u64> {
    BTreeSet::new()
}

#[test]
fn a_heading_that_frames_the_erased_name_is_a_label_site() {
    let pairs = [pair(
        "recipes.md",
        "# Dongpo Pork\n\nBraise.\n",
        "# Tomato and Egg (no Dongpo Pork)\n\nStir.\n",
        Lang::Markdown,
    )];
    let f = measure(&pairs, &none());
    assert_eq!((f.label, f.prose), (1, 0));
    assert_eq!(
        f.sites,
        [Site {
            file: "recipes.md".into(),
            line: 1,
            kind: Kind::Bracketed,
            name: "dongpo".into(),
            excerpt: "Tomato and Egg (no Dongpo Pork)".into(),
        }]
    );
}

#[test]
fn an_identifier_and_a_docstring_are_the_bare_and_prose_sites() {
    let before = "fn braise_dongpo_pork() {}\n";
    let after = "/// This recipe no longer uses braise_dongpo_pork.\nfn cook_without_dongpo() {}\n";
    let f = measure(&[pair("kitchen.rs", before, after, Lang::Rust)], &none());
    assert_eq!((f.label, f.prose), (1, 1), "{:?}", f.sites);
    let sites: Vec<(usize, Kind)> = f.sites.iter().map(|s| (s.line, s.kind)).collect();
    assert_eq!(sites, [(2, Kind::Bare), (1, Kind::Prose)]);
}

#[test]
fn a_mark_without_a_name_or_a_name_without_a_mark_is_nothing() {
    let before = "fn braise_dongpo_pork() {}\n";
    let mark_only = "/// This recipe no longer needs a wok.\nfn cook() {}\n";
    let name_only = "/// See braise_dongpo_pork in the old cookbook.\nfn cook() {}\n";
    // the conjunction is read per sentence: a mark in one and the
    // name in the next is two sentences about two things
    let split = "/// We no longer simmer. See braise_dongpo_pork for the old way.\nfn cook() {}\n";
    for after in [mark_only, name_only, split] {
        let f = measure(&[pair("kitchen.rs", before, after, Lang::Rust)], &none());
        assert_eq!((f.label, f.prose), (0, 0), "{after}");
        assert!(!f.erased.is_empty(), "the name was erased all the same");
    }
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
    let f = measure(&pairs, &none());
    assert_eq!((f.label, f.prose), (0, 0));
    assert_eq!(f.exempt, [("CHANGELOG.md".to_string(), Witness::Path)]);
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
    let f = measure(&pairs, &session);
    assert_eq!(f.label, 1);
    assert!(f.erased.is_empty(), "this edit erased nothing itself");
    assert!(
        measure(&pairs, &none()).sites.is_empty(),
        "without the session there is nothing to bind"
    );
}

#[test]
fn the_feed_object_carries_counts_keys_and_sites_but_no_name() {
    let pairs = [pair(
        "recipes.md",
        "# Dongpo Pork\n",
        "# Tomato (no Dongpo Pork)\n",
        Lang::Markdown,
    )];
    let f = measure(&pairs, &none());
    let j = feed_json(&f, Some(3));
    assert_eq!(j["rev"], TOMBSTONE_REV);
    assert_eq!(
        (j["label"].as_u64(), j["prose"].as_u64()),
        (Some(1), Some(0))
    );
    assert_eq!(j["session_erased"], 3);
    assert_eq!(j["erased"], f.erased.len());
    assert_eq!(
        j["erased_hashes"].as_array().map(Vec::len),
        Some(f.erased.len())
    );
    // spelled from parts: a literal `file:line kind` would read as a
    // citation of that page to the source-citation gate
    assert_eq!(j["sites"][0], format!("{}:{} bracketed", "recipes.md", 1));
    let text = j.to_string();
    assert!(!text.to_lowercase().contains("dongpo"), "{text}");
}

#[test]
fn sites_are_capped_while_the_counts_stay_exact() {
    let after: String = (0..12)
        .map(|i| format!("# Sides {i} (no dongpo)\n"))
        .collect();
    let pairs = [pair("r.md", "# dongpo\n", &after, Lang::Markdown)];
    let f = measure(&pairs, &none());
    assert_eq!(f.label, 12);
    assert_eq!(
        feed_json(&f, None)["sites"].as_array().map(Vec::len),
        Some(SITE_CAP)
    );
}
