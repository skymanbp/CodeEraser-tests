use super::*;
use serde_json::json;

/// A run's occurrences: every start position, overlapping ones
/// included; nothing for an empty run or one longer than the stream.
#[test]
fn a_run_is_counted_at_every_start() {
    let s = [1, 2, 1, 2, 1, 3];
    assert_eq!(occurrences(&s, &[1, 2]), 2);
    assert_eq!(occurrences(&s, &[1, 2, 1]), 2, "overlapping starts count");
    assert_eq!(occurrences(&s, &[3]), 1);
    assert_eq!(occurrences(&s, &[4]), 0);
    assert_eq!(occurrences(&s, &[]), 0);
    assert_eq!(occurrences(&[1], &[1, 1]), 0);
}

/// The reply read strictly: an ascending in-range table and an `all`
/// that agrees with it, or wire skew by name; a degraded reply is a
/// named non-judgment.
#[test]
fn the_reply_is_read_strictly() {
    let ok = consume(&json!({"relocated": [0, 2], "all": false}), 3).expect("ok");
    assert_eq!(
        ok,
        Verdict {
            moved: vec![0, 2],
            all: false
        }
    );
    let all = consume(&json!({"relocated": [0, 1], "all": true}), 2).expect("all");
    assert!(all.all);
    let bad = [
        (json!({"relocated": [1, 0], "all": false}), "ascend"),
        (json!({"relocated": [3], "all": false}), "ascend"),
        (json!({"relocated": [0], "all": true}), "disagrees"),
        (json!({"relocated": [0]}), "boolean"),
        (
            json!({"degraded": true, "reason": "relocate_too_large"}),
            "too_large",
        ),
    ];
    for (reply, needle) in bad {
        let err = consume(&reply, 2).expect_err("refused");
        assert!(err.contains(needle), "{reply}: {err}");
    }
    assert_eq!(body(&[[2, 1]]), json!({"rows": [[2, 1]]}));
}
