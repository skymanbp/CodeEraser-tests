//! The candidate rows one side offers (codex review 2026-09-04): a
//! compound `[tombstone] terms` entry is matched whole, a message is a
//! surface and not a side, and a name written twice is one name.

use crate::scan::lang::Lang;
use crate::tombstone::tests::{declared, none, pair, plain, rows};
use crate::tombstone::{Kind, Policy, measure_with, names};

/// A compound term is matched whole and canonically (codex review
/// 2026-09-04): `terms = ["DongpoPork"]` keeps `dongpo_pork` out of
/// every name while `braise_dongpo_pork`, whose other word still
/// names, stays one.
#[test]
fn a_compound_term_is_matched_whole_and_canonically() {
    let policy = declared("[tombstone]\nterms = [\"DongpoPork\"]\n");
    let text = "fn dongpo_pork() {}\nfn braise_dongpo_pork() {}\n";
    let got: Vec<String> = names::names_of(text, Lang::Rust, &policy)
        .into_iter()
        .map(|n| n.text)
        .collect();
    assert!(!got.iter().any(|n| n == "dongpo_pork"), "{got:?}");
    assert!(got.iter().any(|n| n == "braise_dongpo_pork"), "{got:?}");
}

/// A message is a SURFACE, not a side (codex review 2026-09-04): its
/// subject line is its one label, its sentences are prose, and the
/// list lead `- dongpo …` — a structural position that keeps a name
/// alive in a FILE — declares nothing, so R stays the pairs' own; no
/// witness reads it.
#[test]
fn a_message_offers_surfaces_but_keeps_no_name_alive() {
    let text = "Sides (no dongpo)\n\n- dongpo is no longer needed.\n";
    let code = || pair("k.rs", "fn dongpo() {}\n", "fn other() {}\n", Lang::Rust);
    let msg = pair("COMMIT_EDITMSG", "", text, Lang::Markdown);
    let f = measure_with(&[code()], &[msg], &none(), &Policy::default());
    assert_eq!(f.erased.len(), 1, "{:?}", f.erased);
    // the subject is a label row; as a paragraph it is also a mark-less
    // prose row the core will not seat; the item's two overlapping
    // marks (`no longer`, `is no longer needed`) both count
    assert_eq!(
        rows(&f),
        [
            (1, Kind::Bracketed, 0, true),
            (1, Kind::Prose, 0, true),
            (3, Kind::Prose, 2, true)
        ]
    );
    assert!(f.exempt.is_empty(), "no witness reads a message");
    let as_file = pair("notes.md", "", text, Lang::Markdown);
    let f = plain(&[code(), as_file], &none());
    assert!(
        f.erased.is_empty() && f.rows.is_empty(),
        "as a file, the list lead keeps the name alive: {:?}",
        f.rows
    );
}

/// A name written twice in one surface is one name (codex review
/// 2026-09-04): `names` counts distinct keys, on labels as on prose.
#[test]
fn a_name_written_twice_in_one_surface_is_one_name() {
    let after =
        "# Sides (no dongpo, no dongpo)\n\nWe no longer braise dongpo, and dongpo is out.\n";
    let f = plain(
        &[pair("r.md", "# dongpo\n", after, Lang::Markdown)],
        &none(),
    );
    let names: Vec<(Kind, usize)> = f.rows.iter().map(|r| (r.kind, r.names)).collect();
    assert_eq!(
        names,
        [(Kind::Bracketed, 1), (Kind::Prose, 1)],
        "{:?}",
        f.rows
    );
}
