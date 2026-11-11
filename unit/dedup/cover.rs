//! The cover/1 leg's two halves this side owns (v2.35 imports lane, R2):
//! the request plan — rows by path rank in the contract's order, split
//! between file pairs and never inside one — and the strict read of the
//! reply, every malformed answer refused by name.

use super::*;
use serde_json::json;

/// The operating point (t, d).
const F: (usize, usize) = (50, 7);

fn run(a: &'static str, a0: usize, b: &'static str, b0: usize, len: usize) -> Run<'static> {
    Run {
        a,
        a0,
        b,
        b0,
        len,
        distinct: 9,
    }
}

#[test]
fn rows_carry_path_ranks_in_the_contracts_order() {
    let runs = [
        run("z.rs", 5, "z.rs", 90, 60),
        run("a.rs", 7, "z.rs", 0, 60),
        run("a.rs", 1, "z.rs", 3, 50),
    ];
    let rows = wire_rows(&runs);
    let order: Vec<usize> = rows.iter().map(|(i, _)| *i).collect();
    assert_eq!(order, [2, 1, 0], "ascending on (fileA, fileB, a0, b0)");
    assert_eq!(
        rows[0].1,
        [0, 1, 1, 3, 50, 9],
        "a.rs is code 0, z.rs code 1"
    );
    assert_eq!(rows[2].1[..2], [1, 1], "a same-file pair shares one code");
}

#[test]
fn requests_split_between_pairs_and_refuse_one_pair_past_the_cap() {
    let runs: Vec<Run> = [("a", "b", 3), ("a", "c", 2), ("b", "c", 2)]
        .iter()
        .flat_map(|&(a, b, n)| (0..n).map(move |k| run(a, k * 100, b, k * 100, 60)))
        .collect();
    let rows = wire_rows(&runs);
    assert_eq!(
        chunks(&rows, 4).unwrap(),
        [0..3, 3..7],
        "whole pairs, greedy"
    );
    let whole = chunks(&rows, 7).unwrap();
    assert_eq!((whole.len(), whole[0].clone()), (1, 0..7), "one request");
    let refused = chunks(&rows, 2).unwrap_err().to_string();
    assert!(
        refused.contains("one file pair has 3 verified runs, over the core's cover_rows cap 2"),
        "{refused}"
    );
}

/// A well-formed reply for three rows: 0 reported, 1 suppressed, 2 dropped.
fn reply() -> Value {
    json!({"reported": [0], "suppressed": [1], "dropped": [2], "minTokens": 50, "minDistinct": 7,
           "counts": {"rows": 3, "reported": 1, "suppressed": 1, "dropped": 1}, "degraded": false})
}

#[test]
fn the_reply_is_read_strictly() {
    use Class::*;
    assert_eq!(
        consume(&reply(), 3, F).unwrap(),
        [Reported, Suppressed, Dropped]
    );
    let bent = |path: &str, v: Value| {
        let mut r = reply();
        let (head, key) = path.split_once('.').unwrap_or(("", path));
        if head.is_empty() {
            r[key] = v
        } else {
            r[head][key] = v
        }
        consume(&r, 3, F).unwrap_err().to_string()
    };
    let cases = [
        ("counts.rows", json!(4), "rows or filters not echoed"),
        ("minTokens", json!(49), "rows or filters not echoed"),
        ("minDistinct", json!(6), "rows or filters not echoed"),
        ("counts.dropped", json!(2), "counts.dropped"),
        ("dropped", json!([2, 2]), "dropped not ascending"),
        ("dropped", json!([1]), "row 1 in two classes"),
        ("dropped", json!([3]), "dropped index 3 out of range"),
    ];
    for (path, v, why) in cases {
        let got = bent(path, v);
        assert!(got.contains(why), "{path}: {got}");
    }
    let mut short = reply();
    short["dropped"] = json!([]);
    short["counts"]["dropped"] = json!(0);
    let got = consume(&short, 3, F).unwrap_err().to_string();
    assert!(got.contains("row 2 in no class"), "{got}");
}
