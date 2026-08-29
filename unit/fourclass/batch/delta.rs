use super::super::PairInput;
use super::*;
use crate::fourclass::model::{ChangedLines, FourClass};
use crate::scan::lang::Lang;
use serde_json::json;

fn one_leftover() -> (
    Vec<PairInput<'static>>,
    Vec<(Side, Side)>,
    Vec<Classification>,
) {
    let inputs = vec![PairInput {
        before: "let x = compute();\n",
        after: "",
        lang: Lang::Rust,
    }];
    let sent = vec![(vec![vec![(1usize, 7u64, 10usize)]], Vec::new())];
    let pairs = vec![Classification {
        counts: FourClass {
            removed_deleted: 1,
            ..FourClass::default()
        },
        moved: Vec::new(),
        relocated_units: Vec::new(),
        changed: ChangedLines {
            removed: vec![1],
            added: Vec::new(),
        },
        degraded: false,
    }];
    (inputs, sent, pairs)
}

/// The same leftover line listed twice in the delta is a named
/// error — under the old any()-check it decremented the class
/// count twice and UNDERFLOWED the usize (panic in debug, ~18e18
/// "deleted" lines in release). AND the caller's pairs stay the
/// untouched pure L1: the first (valid) application happened on
/// merge's copy, so the error path leaks none of it (#5).
#[test]
fn a_double_listed_delta_line_is_refused_not_underflowed() {
    let (inputs, sent, pairs) = one_leftover();
    let reply = json!({"moved": [[0, [1, 1], []]], "blocks": []});
    let err = merge(&reply, &inputs, &sent, &pairs).expect_err("double apply");
    assert!(err.contains("unconsumed"), "{err}");
    assert_eq!(pairs[0].counts.removed_deleted, 1, "pure L1 intact");
    assert!(pairs[0].moved.is_empty(), "no half-merged moved lines");
}

/// The single-application baseline that keeps the refusal above
/// from passing vacuously: one line, applied once, reclassifies —
/// on the RETURNED copy, never the input.
#[test]
fn a_single_delta_line_reclassifies_once() {
    let (inputs, sent, pairs) = one_leftover();
    let reply = json!({"moved": [[0, [1], []]], "blocks": []});
    let (merged, _) = merge(&reply, &inputs, &sent, &pairs).expect("single apply");
    assert_eq!(merged[0].counts.removed_deleted, 0);
    assert_eq!(merged[0].counts.removed_moved, 1);
    assert_eq!(pairs[0].counts.removed_deleted, 1, "input untouched");
}
