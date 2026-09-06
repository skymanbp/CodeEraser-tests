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

/// O24: the aggregate tail names the family command from the one
/// table (`family_command`), and the plan document carries the same
/// map under `families` (0.3.0); a kind the table does not know keeps
/// the old sentence and stays out of the map rather than getting a
/// made-up command.
#[test]
fn out_of_class_names_the_family_command_on_every_face() {
    use crate::erase::model::{Counts, Plan, T1T2_NO_WHOLE_UNIT};
    assert_eq!(
        out_of_class_line(T1T2_NO_WHOLE_UNIT, 40),
        "advisory t1t2_block_no_whole_unit: 40 finding(s) — no deterministic-safe erase; see `ce dedup`"
    );
    assert_eq!(
        out_of_class_line("someday_kind", 1),
        "advisory someday_kind: 1 finding(s) — no deterministic-safe erase; see the family command"
    );
    let mut out_of_class = std::collections::BTreeMap::new();
    out_of_class.insert(T1T2_NO_WHOLE_UNIT, 40usize);
    out_of_class.insert("someday_kind", 1);
    let plan = Plan {
        rows: vec![],
        counts: Counts {
            candidates: 0,
            eraseable: 0,
            advisory: 0,
            out_of_class,
        },
    };
    let doc = report_json(&plan);
    assert_eq!(doc["schema"], "ce.erase-plan/0.3.0");
    assert_eq!(
        doc["families"],
        serde_json::json!({ T1T2_NO_WHOLE_UNIT: "ce dedup" })
    );
}
