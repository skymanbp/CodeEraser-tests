use super::*;
use serde_json::{Value, json};

/// A probe reply match over `file` lines `start..=end` standing for
/// `copies` units; `copies` None leaves the key out.
fn m(file: &str, start: u64, end: u64, copies: Option<u64>) -> Value {
    let mut v = json!({"file": file, "start_line": start, "end_line": end, "tokens": 60});
    if let Some(c) = copies {
        v["copies"] = c.into();
    }
    v
}

fn one(file: &str, start: u64, end: u64) -> Value {
    m(file, start, end, Some(1))
}

/// The replaced side of most legs: one copy of b.py lines 10-30.
fn base() -> [Value; 1] {
    [one("b.py", 10, 30)]
}

/// Each match `novel` keeps as `file:first line`, in order.
fn kept(new: Vec<Value>, base: &[Value]) -> Vec<String> {
    let site = |v: &Value| format!("{}:{}", v["file"].as_str().unwrap_or("?"), v["start_line"]);
    novel(new, base).iter().map(site).collect()
}

/// Units: a match stands for `copies` units on its region and the
/// replaced text's units are the cover. One base copy, one new match of
/// two copies: the first unit is carried, the second finds every line
/// used up — novel, and the match is kept once. Base 2 / new 2: both
/// carried (P2, n <= k). Base 2 / new 3: the third unit is novel.
#[test]
fn copies_are_subtracted_in_units() {
    let b = "b.py";
    assert_eq!(
        kept(vec![m(b, 10, 30, Some(2))], &[one(b, 10, 30)]),
        ["b.py:10"]
    );
    assert!(kept(vec![m(b, 10, 30, Some(2))], &[m(b, 10, 30, Some(2))]).is_empty());
    let two = [m(b, 10, 30, Some(2))];
    assert_eq!(kept(vec![m(b, 10, 30, Some(3))], &two), ["b.py:10"]);
}

/// (P1) with one copy everywhere it is the older rule: one base [10-30]
/// carries a copy the probe split in two, [10-20] and [21-30] (the
/// second's lines are still unused); a partial overlap carries; another
/// source file's region is no cover. (P3) a second match on the same
/// lines (a run of another length) finds every line used up: novel.
#[test]
fn single_copies_keep_the_older_overlap_rule() {
    let base = base();
    assert!(kept(vec![one("b.py", 10, 20), one("b.py", 21, 30)], &base).is_empty());
    assert!(kept(vec![one("b.py", 25, 40)], &base).is_empty());
    assert_eq!(kept(vec![one("c.py", 10, 30)], &base), ["c.py:10"]);
    let twice = vec![one("b.py", 10, 30), one("b.py", 12, 30)];
    assert_eq!(kept(twice, &base), ["b.py:12"]);
}

/// An unanswered multiplicity: a new match without `copies` (or with 0)
/// is novel; a replaced-side match without it adds no cover.
#[test]
fn a_match_without_a_count_is_never_an_allowance() {
    assert_eq!(kept(vec![m("b.py", 10, 30, None)], &base()), ["b.py:10"]);
    assert_eq!(kept(vec![m("b.py", 10, 30, Some(0))], &base()), ["b.py:10"]);
    let uncounted = [m("b.py", 10, 30, None)];
    assert_eq!(kept(vec![one("b.py", 10, 30)], &uncounted), ["b.py:10"]);
}
