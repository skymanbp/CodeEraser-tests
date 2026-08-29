use super::*;

/// The LANGS table is total (row() cannot panic), the wire
/// positions are frozen (RM15), and the v2.5 boundary splits
/// exactly where the plan says: scan-only after the sentinel,
/// Markdown grammar-less but judged.
#[test]
fn langs_table_is_total_and_the_boundary_holds() {
    assert_eq!(LANGS.len(), 15, "one row per variant");
    assert_eq!(Lang::LangUnknown as i64, 7, "frozen sentinel");
    assert_eq!(Lang::JavaScript as i64, 8, "arm appends after it");
    for &(l, ..) in LANGS {
        assert!(!l.name().is_empty()); // row() is total
        assert_eq!(l.scan_only(), l as i64 > 7, "boundary = the sentinel");
    }
    // the seven judged languages, bits 0..6 — the echo-pinned
    // mask is a pure summary of the scan_only column
    assert_eq!(Lang::judged_mask(), 0b111_1111);
    let js = Path::new("a.js");
    assert_eq!(Lang::from_path(js), Some(Lang::JavaScript));
    assert_eq!(Lang::judged_path(js), None, "sized, never judged");
    assert_eq!(Lang::judged_path(Path::new("a.md")), Some(Lang::Markdown));
}
