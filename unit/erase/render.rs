use super::*;
use crate::erase::model::Row;

fn row(reason: &'static str, sites: i64) -> Row {
    Row {
        class: "t1_twin",
        eraseable: false,
        reason,
        path: "a.py".into(),
        span: None,
        provenance: String::new(),
        sites,
        hash: 0,
    }
}

/// FIELD-TEST (plan v2.25): only the reason that IS about the
/// language's unresolved sites carries their count; every other
/// advisory reason renders exactly as before, and a zero count is
/// still spelled out — "0 sites" is a fact, not an absence.
#[test]
fn only_language_unresolved_carries_its_site_count() {
    assert_eq!(
        reason_detail(&row("language_unresolved", 312)),
        " — 312 unresolved reference sites in this language"
    );
    assert_eq!(
        reason_detail(&row("language_unresolved", 0)),
        " — 0 unresolved reference sites in this language"
    );
    for other in ["bytes_differ", "not_full_segment", "public_surface"] {
        assert_eq!(reason_detail(&row(other, 312)), "", "{other}");
    }
}
