use super::*;
use crate::flow::lower::lower_file;

/// Three units: an unreachable run, a dead store, an unused parameter.
const PY: &str = "def gone():\n    return 1\n    print(\"never\")\n\n\ndef overwritten():\n    x = 1\n    x = 2\n    return x\n\n\ndef ignores(a):\n    return 3\n";

fn finding(kind: u8, seq: i64, v: i64) -> Finding {
    Finding {
        u: 0,
        kind,
        seq,
        v,
        seq_end: seq,
    }
}

/// The names are one table, in the core's code order, and an unknown
/// code is never passed off as a kind.
#[test]
fn the_kind_names_are_the_core_codes_in_order() {
    let names: Vec<&str> = (0..5).map(kind_name).collect();
    assert_eq!(
        names,
        [
            "unreachable",
            "dead_store",
            "unused_local",
            "unused_param",
            "?"
        ]
    );
}

/// An unused parameter is advisory in every language; the others are
/// judged exactly when the precision gate admitted the language.
#[test]
fn the_advisory_kind_is_never_judged_and_the_mask_decides_the_rest() {
    let py = Lang::Python;
    assert!(!judged(py, ADVISORY));
    for kind in 0..ADVISORY {
        assert_eq!(judged(py, kind), lang_judged(py), "kind {kind}");
    }
    let admitted = crate::flow::judged_mask() & (1 << py as i64) != 0;
    assert_eq!(lang_judged(py), admitted);
}

/// A finding reads its lines and variable through the unit's legend: a
/// statement's line, or a parameter's own line for kind 3.
#[test]
fn a_finding_is_placed_through_its_units_legend() {
    let file = lower_file(PY, Lang::Python).expect("python lowers");
    let names: Vec<(usize, &str)> = file
        .units
        .iter()
        .map(|u| (u.nth, u.name.as_str()))
        .collect();
    assert_eq!(names, [(0, "gone"), (1, "overwritten"), (2, "ignores")]);
    let run = place(&file, 0, &finding(0, 1, -1)).expect("a statement of gone");
    assert_eq!(
        (run.unit.as_str(), run.line, run.line_end, run.var),
        ("gone", 3, 3, None)
    );
    let param = place(&file, 2, &finding(ADVISORY, -1, 0)).expect("a parameter of ignores");
    assert_eq!((param.line, param.var.as_deref()), (12, Some("a")));
    assert!(place(&file, 7, &finding(0, 0, -1)).is_none(), "no unit 7");
}

/// The feeds count findings by kind under the kind names, zeros kept.
#[test]
fn kinds_are_counted_under_their_names() {
    assert_eq!(
        kinds_json([0, 3, 3]),
        json!({"unreachable": 1, "dead_store": 0, "unused_local": 0, "unused_param": 2})
    );
}
