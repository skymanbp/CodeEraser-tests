use super::*;
use serde_json::json;

const DEN: i64 = 1 << 16;

fn hit(doc: usize, hits: [u32; 6], shape_equal: bool, score_fp: i64) -> Hit {
    Hit {
        doc,
        score: score_fp / DEN,
        score_fp,
        den: DEN,
        hits,
        shape_equal,
    }
}

#[test]
fn rows_are_the_nine_integers_in_rank_order_over_the_answered_denominator() {
    let sent = rows(&[
        hit(7, [1, 0, 1, 0, 2, 0], false, 3 * DEN),
        hit(2, [2, 1, 0, 0, 0, 1], true, 5),
    ]);
    let den = DEN;
    assert_eq!(
        sent,
        [
            [1, 0, 1, 0, 2, 0, 0, 3 * den, den],
            [2, 1, 0, 0, 0, 1, 1, 5, den]
        ]
    );
}

#[test]
fn the_body_carries_the_ranked_bag_and_the_rows() {
    assert_eq!(
        body(&[[3, 512], [9, 1024]], &rows(&[])),
        json!({"query": [[3, 512], [9, 1024]], "rows": []})
    );
}

fn reply(order: Vec<usize>, roles: Vec<bool>) -> serde_json::Value {
    let role = roles.iter().filter(|r| **r).count();
    json!({"order": order, "roles": roles, "degraded": false,
           "counts": {"rows": roles.len(), "queryTerms": 2, "role": role}})
}

#[test]
fn a_well_formed_reply_is_relayed_as_the_core_said() {
    assert_eq!(
        consume(&reply(vec![2, 0, 1], vec![true, false, true]), 3),
        Ok(Judged {
            order: vec![2, 0, 1],
            roles: vec![true, false, true],
        })
    );
    assert_eq!(consume(&reply(vec![], vec![]), 0), Ok(Judged::default()));
}

#[test]
fn a_degraded_reply_and_every_skew_are_named_non_judgments() {
    let degraded =
        json!({"degraded": true, "reason": "similar_too_large", "order": [], "roles": []});
    assert_eq!(consume(&degraded, 1), Err("similar_too_large".into()));
    // (reply, rows sent, the clause the refusal must name)
    let skews = [
        (reply(vec![0, 0], vec![true, false]), 2, "permutation"),
        (reply(vec![0, 2], vec![true, false]), 2, "permutation"),
        (reply(vec![0, 1], vec![true]), 2, "one role bit"),
        (
            json!({"order": [0], "roles": [1], "degraded": false}),
            1,
            "roles:",
        ),
        (
            json!({"order": [0], "roles": [true], "degraded": false, "counts": {"rows": 1, "role": 0}}),
            1,
            "counts disagree",
        ),
    ];
    for (r, sent, want) in skews {
        let err = consume(&r, sent).expect_err(want);
        assert!(err.contains(want), "{want}: {err}");
    }
}
