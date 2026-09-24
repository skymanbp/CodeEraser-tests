//! scan/wire.rs battery, #[path]-mounted at the 300-line file gate
//! (the candidates_tests.rs precedent) when the 3.2.0 rulepack channel
//! grew the codec: the default grade table, the chunk plan, and the
//! class override rows.

use super::*;

/// The default grade table mirrors Thresholds::default — the
/// same numbers CE.Scan.Cost.gradeTable declares; the fixture
/// book pins the core half, this pins the assembly half.
#[test]
fn default_grades_carry_every_code_in_order() {
    let rows = grade_rows(&Thresholds::default()).expect("coherent defaults");
    assert_eq!(rows.len(), 7);
    assert!(rows.iter().enumerate().all(|(i, r)| r[0] == i as u64));
    assert_eq!(rows[0], [0, 300, 750]);
    assert_eq!(rows[1], [1, 50, 75]);
    assert_eq!(rows[6], [6, 0, 0]);
    // the C6 ladder guard names the ce.toml keys, and fail 0
    // stays a legal "no hard line"
    let raised = Thresholds {
        file_lines_warn: 800,
        ..Thresholds::default()
    };
    let err = grade_rows(&raised).expect_err("fail below warn refuses");
    assert!(err.to_string().contains("file_lines_warn"), "{err}");
    let unlined = Thresholds {
        file_lines_warn: 800,
        file_lines_fail: 0,
        ..Thresholds::default()
    };
    grade_rows(&unlined).expect("fail 0 = no hard line is coherent");
}

/// The override rows carry each class's EFFECTIVE pair for the
/// codes it declared a line for — a warn beside an inherited fail
/// sends both, an undeclared code sends nothing — (class, code)
/// ascending; no class, no rows.
#[test]
fn class_grade_rows_carry_effective_pairs_for_declared_codes() {
    use crate::config::{ClassCfg, ClassKnobs, RulesCfg};
    let global = Thresholds::default();
    let rules = RulesCfg {
        class: vec![
            ClassCfg {
                name: "tests".into(),
                globs: vec!["cli/tests/**".into()],
                knobs: ClassKnobs {
                    file_lines_warn: Some(400),
                    fn_lines_fail: Some(90),
                    ..Default::default()
                },
            },
            ClassCfg {
                name: "gen".into(),
                globs: vec!["gen/**".into()],
                knobs: ClassKnobs {
                    cognitive_warn: Some(25),
                    ..Default::default()
                },
            },
        ],
    };
    assert_eq!(
        class_grade_rows(&rules, &global),
        vec![[1, 0, 400, 750], [1, 1, 50, 90], [2, 4, 25, 0]]
    );
    assert!(class_grade_rows(&RulesCfg::default(), &global).is_empty());
}

/// The judged-language echo (7.2.0) is pinned like the grade table:
/// the sent mask is accepted, a foreign value is drift refused by
/// name, and no echo at all is a pre-7.2.0 core refused by name.
#[test]
fn the_judged_mask_echo_is_required_and_pinned() {
    let grades = grade_rows(&Thresholds::default()).expect("coherent defaults");
    let r = ScanRequest {
        rows: &[],
        grades: &grades,
        naming: &[],
        row_classes: None,
        overrides: &[],
        fence: Value::Null,
        blocks: &[],
        calls: &[],
    };
    let mask = crate::scan::lang::Lang::judged_mask();
    let mut reply = json!({ "grades": grades, "judgedMask": mask });
    assert_echo(&reply, &r).expect("the sent mask echoes");
    reply["judgedMask"] = json!(mask | 1 << 40);
    let drift = assert_echo(&reply, &r).expect_err("a foreign mask is drift");
    assert!(drift.to_string().contains("judgedMask"), "{drift}");
    reply.as_object_mut().expect("object").remove("judgedMask");
    let silent = assert_echo(&reply, &r).expect_err("no echo = a pre-7.2.0 core");
    assert!(silent.to_string().contains("7.2.0"), "{silent}");
}
