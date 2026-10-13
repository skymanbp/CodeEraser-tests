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

/// The one line planner of every lockstep family (plan v2.33 step 11b
/// item 3), over a stand-in: a chunk whose body fits is sent whole, one
/// over the line is halved until each body fits, and one pair whose body
/// alone passes it is refused by name with nothing sent.
#[test]
fn a_chunk_over_the_line_is_halved_until_each_body_fits() {
    let build = |c: &[u64]| (Vec::new(), json!(c.iter().sum::<u64>()));
    let fits = |b: &Value| match b.as_u64() <= Some(10) {
        true => Ok(()),
        false => Err(format!("{b} over 10")),
    };
    let plan = |c: &[u64]| {
        let mut sent = Vec::new();
        within_line(c, &build, &fits, &mut |_, body| {
            sent.push(body);
            Ok(())
        })
        .map(|()| sent)
    };
    assert_eq!(plan(&[1, 2]).expect("fits"), [json!(3)], "whole");
    assert_eq!(
        plan(&[4, 3, 2, 6, 5]).expect("halved"),
        [json!(7), json!(2), json!(6), json!(5)]
    );
    let alone = plan(&[11]).expect_err("one pair over the line");
    assert_eq!(alone.to_string(), "11 over 10 (one pair alone)");
}
