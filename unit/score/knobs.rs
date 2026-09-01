use super::*;

/// The name→code weld is load-bearing wire vocabulary (review
/// C3): the axis order IS the core's Score.penalties order, so
/// this literal pin turns a reorder into a named red instead of
/// a silently re-aimed weight — with the 2.8.0 echo assert as
/// the runtime half of the same weld.
#[test]
fn axes_order_is_frozen_and_weight_rows_resolve_names() {
    assert_eq!(
        AXES,
        [
            "size",
            "complexity",
            "clone",
            "docdup",
            "deadcode",
            "churn",
            "cycle"
        ]
    );
    let mut cfg = ScoreCfg::default();
    cfg.weights.insert("clone".into(), 5);
    assert_eq!(weight_rows(&cfg).expect("known name"), vec![[2, 5]]);
    cfg.weights.insert("bogus".into(), 1);
    assert!(weight_rows(&cfg).is_err(), "unknown axis name refuses");
}

/// The class rows are the ceilings codes under a class index,
/// (class, code)-ascending: an absent knob sends nothing, a hard
/// line of 0 sends nothing (no hard line), a warn line of 0 still
/// rides — the core's refusal is the loud reading.
#[test]
fn class_knob_rows_shadow_the_ceiling_codes_in_order() {
    use crate::config::{ClassCfg, ClassKnobs};
    let class = |w: Option<usize>, h: Option<usize>, c: Option<usize>| ClassCfg {
        name: "x".into(),
        globs: vec!["**".into()],
        knobs: ClassKnobs {
            file_lines_warn: w,
            file_lines_fail: h,
            cognitive_warn: c,
            // the scan-only lines (P3) never reach the score rows —
            // declared here to prove exactly that. `cognitive_fail`
            // (v2.24) rides this list for the same reason and carries
            // the stronger claim: arming a complexity WALL must leave
            // the score's ceilings table untouched, which is what
            // keeps scores comparable across the knob's arrival.
            fn_lines_warn: Some(80),
            fn_lines_fail: Some(90),
            cognitive_fail: Some(40),
            ratchet_tolerance: None,
            cognitive_ratchet_tolerance: None,
        },
    };
    let rules = RulesCfg {
        class: vec![
            class(Some(400), Some(0), None),
            class(None, Some(900), Some(20)),
            class(Some(0), None, None),
        ],
    };
    assert_eq!(
        class_knob_rows(&rules),
        vec![[1, 0, 400], [2, 1, 20], [2, 2, 900], [3, 0, 0]]
    );
    assert!(class_knob_rows(&RulesCfg::default()).is_empty());
}
