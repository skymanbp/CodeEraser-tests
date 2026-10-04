use super::*;
use serde_json::json;

/// The three reply pins fire generically ONCE here; family wire
/// tests keep only their own knob lists and bespoke cases.
#[test]
fn reply_pins_refuse_drift_and_degradation() {
    let ok = json!({"knobs": {"x": 7}, "degraded": false,
            "scores": [[1, 2]], "counts": {"n": 3}});
    assert!(pin_knobs(&ok, &[("x", json!(7))], "pair").is_ok());
    assert!(pin_knobs(&ok, &[("x", json!(8))], "pair").is_err());
    assert!(refuse_degraded(&ok, "pair").is_ok());
    let mut bad = ok.clone();
    bad["degraded"] = json!(true);
    assert!(refuse_degraded(&bad, "pair").is_err());
    let (rows, counts) = scores_and_counts::<[u64; 2]>(&ok, &["n"]).expect("decode");
    assert_eq!((rows, counts), (vec![[1, 2]], vec![3]));
}

/// The per-row column throat (the verdict bits of ADR-008 P1 and every
/// family column beside them) fires generically ONCE here: a column
/// decodes in row order, a truncated or missing one refuses — an
/// unqualified score row must never default.
#[test]
fn row_columns_decode_and_length_lock() {
    let ok = json!({"verdicts": [true, false]});
    let bits: Vec<bool> = rows_for(&ok, "verdicts", 2).expect("bits");
    assert_eq!(bits, vec![true, false]);
    let short = rows_for::<bool>(&ok, "verdicts", 3).expect_err("truncated bits refuse");
    assert!(
        short
            .to_string()
            .contains("core sent 2 verdicts for 3 score rows"),
        "{short}"
    );
    assert!(
        rows_for::<bool>(&json!({}), "verdicts", 0).is_err(),
        "missing verdicts refuses even for zero rows"
    );
}
