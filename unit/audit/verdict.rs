use super::*;

fn block(a: &str, b: &str) -> crate::dedup::pairs::Block {
    crate::dedup::pairs::Block {
        a_file: a.into(),
        a_start: 1,
        a_end: 5,
        b_file: b.into(),
        b_start: 9,
        b_end: 13,
        tokens: 50,
        distinct: 20,
    }
}

#[test]
fn convictions_relabel_and_the_fail_bit_relays() {
    let blocks = vec![block("x.rs", "y.rs"), block("y.rs", "z.rs")];
    let reply = json!({"dups": [1], "fail": true, "degraded": false});
    let v = consume(&reply, &blocks).expect("judged");
    assert!(v.fail);
    assert_eq!(v.dups, 1);
    assert_eq!(v.shown, vec!["y.rs:1-5 <-> z.rs:9-13 (50 tokens)"]);
}

#[test]
fn a_degraded_reply_and_a_skewed_index_both_answer_none() {
    let blocks = vec![block("x.rs", "y.rs")];
    let degraded = json!({"dups": [], "fail": true, "degraded": true});
    assert!(consume(&degraded, &blocks).is_none());
    // index 7 names no block: wire skew, not a rendering guess
    let skewed = json!({"dups": [7], "fail": true, "degraded": false});
    assert!(consume(&skewed, &blocks).is_none());
}

#[test]
fn display_caps_at_ten_but_the_count_stays_true() {
    let blocks: Vec<_> = (0..12)
        .map(|i| block(&format!("a{i}.rs"), "b.rs"))
        .collect();
    let reply = json!({"dups": (0..12).collect::<Vec<usize>>(), "fail": true, "degraded": false});
    let v = consume(&reply, &blocks).expect("judged");
    assert_eq!(v.dups, 12);
    assert_eq!(v.shown.len(), 10);
}
