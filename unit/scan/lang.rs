use super::*;

/// The LANGS table is total (row() cannot panic), the wire positions
/// are frozen (RM15), and the boundary splits where the plan says:
/// the seven launch codes 0..6 (Markdown grammar-less but judged),
/// the sentinel 7, the v2.5 size-only arm 8..14, and the plan v2.30
/// codes 15..20 — C and C++ judged since step 2, Java since step 3, Lua
/// and R since step 4, and Ruby still reserved (out of v2.30, booklet
/// §14 item 13): a row without an extension, never produced by
/// from_path, outside the judged mask and grammar-less
/// (language-expansion.md §13).
#[test]
fn langs_table_is_total_and_the_boundary_holds() {
    assert_eq!(LANGS.len(), 21, "one row per variant");
    assert_eq!(Lang::LangUnknown as i64, 7, "frozen sentinel");
    assert_eq!(Lang::JavaScript as i64, 8, "arm appends after it");
    assert_eq!(Lang::C as i64, 15, "plan v2.30 codes append after the arm");
    assert_eq!(Lang::R as i64, 20, "… through R");
    for &(l, exts, ..) in LANGS {
        assert!(!l.name().is_empty()); // row() is total
        let code = l as i64;
        let reserved = l == Lang::Ruby;
        assert_eq!(
            l.scan_only(),
            (8..=14).contains(&code) || reserved,
            "boundary = the arm plus the rows still reserved: {l:?}"
        );
        assert_eq!(
            exts.is_empty(),
            l == Lang::LangUnknown || reserved,
            "extension-less = sentinel or reserved: {l:?}"
        );
        assert!(
            !reserved || l.grammar().is_none(),
            "a reserved row has no grammar yet: {l:?}"
        );
        assert_eq!(
            l.fingerprints(),
            l.grammar().is_some() && l != Lang::Html,
            "{l:?}"
        );
    }
    // the twelve judged languages: bits 0..6 plus C (15), C++ (16), Lua
    // (17), Java (18) and R (20) — the echo-pinned mask is a pure
    // summary of the scan_only column
    assert_eq!(
        Lang::judged_mask(),
        0b111_1111 | (1 << 15) | (1 << 16) | (1 << 17) | (1 << 18) | (1 << 20)
    );
}

/// The boundary as a path meets it: a size-only extension is sized and
/// never judged, every judged extension reaches its language (R under
/// either case), a header reads as C++, and a reserved language has no
/// extension yet.
#[test]
fn paths_reach_their_languages() {
    let js = Path::new("a.js");
    assert_eq!(Lang::from_path(js), Some(Lang::JavaScript));
    assert_eq!(Lang::judged_path(js), None, "sized, never judged");
    assert_eq!(Lang::judged_path(Path::new("a.md")), Some(Lang::Markdown));
    assert_eq!(Lang::judged_path(Path::new("a.c")), Some(Lang::C));
    assert_eq!(Lang::judged_path(Path::new("A.java")), Some(Lang::Java));
    assert_eq!(Lang::judged_path(Path::new("a.lua")), Some(Lang::Lua));
    for r in ["a.R", "a.r"] {
        assert_eq!(Lang::judged_path(Path::new(r)), Some(Lang::R), "{r}");
    }
    assert_eq!(
        Lang::judged_path(Path::new("a.h")),
        Some(Lang::Cpp),
        "a header is C++ by the 2026-09-24 ruling: parsed as C it loses every class body"
    );
    assert_eq!(
        Lang::from_path(Path::new("a.rb")),
        None,
        "reserved: no extension until its step"
    );
}
