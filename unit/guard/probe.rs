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

/// The (file, first line) of each match `novel` keeps, in order.
fn kept(new: Vec<Value>, base: &[Value]) -> Vec<(String, u64)> {
    novel(new, base)
        .iter()
        .map(|v| {
            let f = v["file"].as_str().unwrap_or("?").to_string();
            (f, v["start_line"].as_u64().unwrap_or(0))
        })
        .collect()
}

fn at(file: &str, line: u64) -> Vec<(String, u64)> {
    vec![(file.to_string(), line)]
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
        at(b, 10)
    );
    assert!(kept(vec![m(b, 10, 30, Some(2))], &[m(b, 10, 30, Some(2))]).is_empty());
    let three = kept(vec![m(b, 10, 30, Some(3))], &[m(b, 10, 30, Some(2))]);
    assert_eq!(three, at(b, 10));
}

/// (P1) with one copy everywhere it is the older rule: one base [10-30]
/// carries a copy the probe split in two, [10-20] and [21-30] (the
/// second's lines are still unused); a partial overlap carries; another
/// source file's region is no cover. (P3) a second match on the same
/// lines (a run of another length) finds every line used up: novel.
#[test]
fn single_copies_keep_the_older_overlap_rule() {
    let base = [one("b.py", 10, 30)];
    assert!(kept(vec![one("b.py", 10, 20), one("b.py", 21, 30)], &base).is_empty());
    assert!(kept(vec![one("b.py", 25, 40)], &base).is_empty());
    assert_eq!(kept(vec![one("c.py", 10, 30)], &base), at("c.py", 10));
    let again = kept(vec![one("b.py", 10, 30), one("b.py", 12, 30)], &base);
    assert_eq!(again, at("b.py", 12));
}

/// An unanswered multiplicity: a new match without `copies` (or with 0)
/// is novel; a replaced-side match without it adds no cover.
#[test]
fn a_match_without_a_count_is_never_an_allowance() {
    let base = [one("b.py", 10, 30)];
    assert_eq!(kept(vec![m("b.py", 10, 30, None)], &base), at("b.py", 10));
    assert_eq!(
        kept(vec![m("b.py", 10, 30, Some(0))], &base),
        at("b.py", 10)
    );
    let uncounted = [m("b.py", 10, 30, None)];
    assert_eq!(kept(vec![one("b.py", 10, 30)], &uncounted), at("b.py", 10));
}

/// A Write whose before side cannot be read subtracts nothing: the
/// replaced text is the one `Applied` read (once), and a directory at
/// the path reads as no text at all.
#[test]
fn an_unreadable_before_text_subtracts_nothing() {
    use crate::guard::envelope::{Envelope, ToolInput};
    let dir = crate::testutil::scratch("guard-probe-unreadable");
    let env = Envelope {
        tool_name: "Write".into(),
        tool_input: ToolInput {
            file_path: dir.to_string_lossy().into_owned(),
            content: "x = 1\n".into(),
            ..ToolInput::default()
        },
        ..Envelope::default()
    };
    let a = super::super::applied::apply(&env).expect("a write always applies");
    assert_eq!(a.prior_text(&env), None);
}
