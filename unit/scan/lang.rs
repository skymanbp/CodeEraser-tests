use super::*;

/// The LANGS table is total (row() cannot panic), the wire positions
/// are frozen (RM15), and the boundary splits where the plan says:
/// the seven launch codes 0..6 (Markdown grammar-less but judged),
/// the sentinel 7, the v2.5 size-only arm 8..14, and the plan v2.30
/// codes 15..20 — C and C++ judged since step 2, Java since step 3, Lua
/// and R since step 4 — with HTML (10) lifted out of the arm in step 5
/// (a document language: judged, never fingerprinted), and Ruby still
/// reserved (out of v2.30, booklet §14 item 13): a row without an
/// extension, never produced by from_path, outside the judged mask and
/// grammar-less (language-expansion.md §13); then the prose-only arm 21
/// (Text, step 5b-8): indexed for docdup alone, outside the judged mask
/// and every size surface.
#[test]
fn langs_table_is_total_and_the_boundary_holds() {
    assert_eq!(LANGS.len(), 22, "one row per variant");
    assert_eq!(Lang::LangUnknown as i64, 7, "frozen sentinel");
    assert_eq!(Lang::JavaScript as i64, 8, "arm appends after it");
    assert_eq!(Lang::C as i64, 15, "plan v2.30 codes append after the arm");
    assert_eq!(Lang::R as i64, 20, "… through R");
    assert_eq!(Lang::Text as i64, 21, "the prose-only arm after R");
    for &(l, exts, ..) in LANGS {
        assert!(!l.name().is_empty()); // row() is total
        let code = l as i64;
        let reserved = l == Lang::Ruby;
        assert_eq!(
            l.scan_only(),
            ((8..=14).contains(&code) && l != Lang::Html) || reserved,
            "boundary = the arm less HTML, plus the rows still reserved: {l:?}"
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
        assert_eq!(l.prose_only(), l == Lang::Text, "{l:?}");
    }
    // the thirteen judged languages: bits 0..6 plus HTML (10), C (15),
    // C++ (16), Lua (17), Java (18) and R (20) — the echo-pinned mask
    // is a pure summary of the scan_only column; Text (21) sits outside
    // it (prose_only)
    assert_eq!(
        Lang::judged_mask(),
        0b111_1111 | (1 << 10) | (1 << 15) | (1 << 16) | (1 << 17) | (1 << 18) | (1 << 20)
    );
}

/// The grammar face is the GRAMMARS table read out: exactly the
/// languages that parse, twelve of them, each once — the docs' grammar
/// count reads this face since step 7 (before it the registry scraped
/// lang.rs's text).
#[test]
fn with_grammar_reads_the_grammar_table_out() {
    let parsing: Vec<Lang> = Lang::with_grammar().collect();
    let expected: Vec<Lang> = LANGS
        .iter()
        .map(|&(l, ..)| l)
        .filter(|l| l.grammar().is_some())
        .collect();
    assert_eq!(
        parsing.len(),
        12,
        "twelve grammars (step 7 promoted the count off a scrape)"
    );
    for l in &expected {
        assert_eq!(parsing.iter().filter(|p| *p == l).count(), 1, "{l:?} once");
    }
    assert_eq!(
        parsing.len(),
        expected.len(),
        "no grammar row outside the table"
    );
}

/// The boundary as a path meets it: a size-only extension is sized and
/// never judged, every judged extension reaches its language (R under
/// either case, HTML under either extension), a header reads as C++,
/// a reserved language has no extension yet, and plain text is
/// indexed for docdup alone (step 5b-8).
#[test]
fn paths_reach_their_languages() {
    let js = Path::new("a.js");
    assert_eq!(Lang::from_path(js), Some(Lang::JavaScript));
    assert_eq!(Lang::judged_path(js), None, "sized, never judged");
    assert_eq!(Lang::judged_path(Path::new("a.md")), Some(Lang::Markdown));
    assert_eq!(Lang::judged_path(Path::new("a.c")), Some(Lang::C));
    assert_eq!(Lang::judged_path(Path::new("A.java")), Some(Lang::Java));
    assert_eq!(Lang::judged_path(Path::new("a.lua")), Some(Lang::Lua));
    for h in ["a.html", "a.htm"] {
        assert_eq!(Lang::judged_path(Path::new(h)), Some(Lang::Html), "{h}");
    }
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
    // the prose-only arm (step 5b-8): a `.txt` is indexed for docdup
    // alone — no size row, no judgment — while a `.txt` name a
    // specification reserves for a machine format is no language at
    // all; a sized-only extension is never indexed
    let txt = Path::new("notes.txt");
    assert_eq!(Lang::from_path(txt), Some(Lang::Text));
    assert_eq!(Lang::indexed_path(txt), Some(Lang::Text));
    assert_eq!(
        (Lang::sized_path(txt), Lang::judged_path(txt)),
        (None, None)
    );
    assert!(Lang::prose_path(txt));
    assert_eq!(Lang::sized_path(js), Some(Lang::JavaScript));
    assert_eq!(Lang::indexed_path(js), None, "sized, never indexed");
    for m in ["x/CMakeLists.txt", "b/compile_flags.txt", "site/robots.txt"] {
        assert_eq!(Lang::from_path(Path::new(m)), None, "{m}: a machine format");
    }
}
