use super::*;

/// O24: the plan document carries the family command behind every
/// out-of-class kind under `families` (0.3.0), from the one table
/// (`family_command`); a kind the table does not know stays out of the
/// map rather than getting a made-up command. (The console sentence is
/// the core's, CE.Text.Erase.)
#[test]
fn out_of_class_names_the_family_command_on_every_face() {
    use crate::erase::model::{Counts, Plan, T1T2_NO_WHOLE_UNIT};
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
