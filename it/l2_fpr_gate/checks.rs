//! Arithmetic checks for the frozen changeset ledger.

use crate::common::stats::{clopper_pearson, permille, round3};
use serde_json::{Value, json};

pub fn u(v: &Value, key: &str) -> u64 {
    v[key].as_u64().unwrap_or_else(|| panic!("{key}: a count"))
}

/// Measured intervals must agree with the published rounded bounds.
pub fn published_intervals(cases: &[(usize, usize, f64, f64)]) {
    for &(n, x, lo, hi) in cases {
        let (got_lo, got_hi) = clopper_pearson(n, x);
        assert!(
            (got_lo - lo).abs() < 5e-4 && (got_hi - hi).abs() < 5e-4,
            "CP({n}, {x}) = ({got_lo:.3}, {got_hi:.3}), published ({lo}, {hi})"
        );
    }
}

fn skipped(c: &Value) -> u64 {
    const REASONS: [&str; 6] = [
        "no_parent",
        "no_pairs",
        "no_texts",
        "no_judged_files",
        "partial",
        "degraded",
    ];
    c["skipped"]
        .as_object()
        .expect("skips")
        .iter()
        .map(|(why, n)| {
            assert!(REASONS.contains(&why.as_str()), "unknown skip: {why}");
            n.as_u64().expect("skip count")
        })
        .sum()
}

/// Counts, named skips, both rate intervals and recall close together.
pub fn coherent(c: &Value) {
    let normal = u(c, "normal");
    let wide = normal + u(c, "unreviewed");
    let abnormal = u(c, "abnormal");
    let strict = u(c, "false_strict");
    let false_wide = u(c, "false_wide");
    let missed = u(c, "misses");
    assert_eq!(
        u(c, "commits"),
        u(c, "events") + skipped(c),
        "every commit accounted"
    );
    assert_eq!(u(c, "events"), wide + abnormal, "events split three ways");
    assert!(strict <= normal && strict <= false_wide && false_wide <= wide);
    assert!(missed <= abnormal, "misses <= abnormal");
    assert_eq!(
        u(c, "intercepts"),
        false_wide + abnormal - missed,
        "fired events"
    );
    let recall = (abnormal > 0).then(|| permille((abnormal - missed) as usize, abnormal as usize));
    assert_eq!(c["recall_permille"], json!(recall), "recall");
    for (n, k, rate, interval) in [
        (normal, strict, "strict_permille", "cp95_strict"),
        (wide, false_wide, "wide_permille", "cp95_wide"),
    ] {
        assert_eq!(u(c, rate), permille(k as usize, n as usize), "{rate}");
        let (lo, hi) = clopper_pearson(n as usize, k as usize);
        assert_eq!(c[interval], json!([round3(lo), round3(hi)]), "{interval}");
    }
}

/// An arbitration row has a full commit id and a concrete firing file.
pub fn arbitration_row(r: &Value) {
    let sha = r["sha"].as_str().expect("sha");
    assert!(
        sha.len() == 40 && sha.chars().all(|c| c.is_ascii_hexdigit()),
        "row sha {sha}"
    );
    assert!(
        ["normal", "copy", "unreviewed"].contains(&r["label"].as_str().expect("label")),
        "row label {r}"
    );
    assert_eq!(r["verdict"], json!("stacking"), "the only M4 rule: {r}");
    assert!(u(r, "pairs") > 0 && !r["file"].as_str().expect("file").is_empty());
}
