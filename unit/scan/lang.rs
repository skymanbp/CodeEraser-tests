use super::*;

/// The LANGS table is total (row() cannot panic), the wire positions
/// are frozen (RM15), and the boundary splits where the plan says:
/// the seven judged codes 0..6 (Markdown grammar-less but judged),
/// the sentinel 7, the v2.5 size-only arm 8..14, and the plan v2.30
/// reserved codes 15..20 — rows without an extension, never produced
/// by from_path, outside the judged mask and grammar-less until their
/// own step wires them (language-expansion.md §13).
#[test]
fn langs_table_is_total_and_the_boundary_holds() {
    assert_eq!(LANGS.len(), 21, "one row per variant");
    assert_eq!(Lang::LangUnknown as i64, 7, "frozen sentinel");
    assert_eq!(Lang::JavaScript as i64, 8, "arm appends after it");
    assert_eq!(Lang::C as i64, 15, "reserved codes append after the arm");
    assert_eq!(Lang::R as i64, 20, "… through R");
    for &(l, exts, ..) in LANGS {
        assert!(!l.name().is_empty()); // row() is total
        assert_eq!(l.scan_only(), l as i64 > 7, "boundary = the sentinel");
        let reserved = l == Lang::LangUnknown || l as i64 >= 15;
        assert_eq!(
            exts.is_empty(),
            reserved,
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
    // the seven judged languages, bits 0..6 — the echo-pinned
    // mask is a pure summary of the scan_only column
    assert_eq!(Lang::judged_mask(), 0b111_1111);
    let js = Path::new("a.js");
    assert_eq!(Lang::from_path(js), Some(Lang::JavaScript));
    assert_eq!(Lang::judged_path(js), None, "sized, never judged");
    assert_eq!(Lang::judged_path(Path::new("a.md")), Some(Lang::Markdown));
    assert_eq!(
        Lang::from_path(Path::new("a.rb")),
        None,
        "reserved: no extension until its step"
    );
}
