//! CI gate over the frozen changeset FPR contract (no core, no git,
//! no corpora): the document is self-consistent, its totals are
//! DERIVED from its corpora rows, the ledger doc's table equals it
//! cell for cell, the interval arithmetic reproduces the intervals
//! the sibling ledger already published, and the plan's line is
//! enforced as the implication "if this class is promoted, it is
//! under the line" — with a negative probe proving the implication
//! bites. One more leg pins that nothing was promoted.

use crate::common::repo_root;
use crate::common::stats::{clopper_pearson, permille};
use crate::eval_support::{eval_doc, load};
use crate::l2_fpr_replay_parts::render::{HEADER, table_row};
use crate::l2_fpr_replay_parts::{GATE_MAX_PERCENT, SCHEMA};
use serde_json::{Value, json};
use std::collections::BTreeSet;

mod checks;
use checks::{coherent, u};

const LEDGER: &str = "docs/FPR-L2.md";

fn contract() -> Value {
    load(&eval_doc("fpr-l2"))
}

/// Every corpus row adds up, and the totals row is the fold of the
/// corpora rows rather than a second, hand-typed authority.
#[test]
fn the_frozen_contract_re_derives() {
    let doc = contract();
    assert_eq!(doc["schema"], json!(SCHEMA), "schema id");
    assert_eq!(doc["gate_max_percent"], json!(GATE_MAX_PERCENT), "gate");
    let corpora = doc["corpora"].as_array().expect("corpora");
    let names: Vec<&str> = corpora
        .iter()
        .map(|c| c["corpus"].as_str().expect("corpus"))
        .collect();
    assert_eq!(names, ["self", "requests", "ripgrep"], "every corpus");
    let keys = [
        "commits",
        "events",
        "normal",
        "unreviewed",
        "abnormal",
        "intercepts",
        "false_strict",
        "false_wide",
        "misses",
    ];
    for c in corpora.iter().chain(std::iter::once(&doc["totals"])) {
        coherent(c);
    }
    for key in keys {
        let sum: u64 = corpora.iter().map(|c| u(c, key)).sum();
        assert_eq!(u(&doc["totals"], key), sum, "totals.{key} is derived");
    }
}

/// The rows are the arbitration record: one per intercepted (commit,
/// firing pair), labelled from the frozen ground truth, and the
/// normal-labelled ones number exactly the strict false count.
#[test]
fn every_row_is_well_formed_and_the_labels_add_up() {
    let doc = contract();
    let rows = doc["rows"].as_array().expect("rows");
    for r in rows {
        checks::arbitration_row(r);
    }
    for c in doc["corpora"].as_array().expect("corpora") {
        let name = c["corpus"].as_str().expect("corpus");
        let mine = |label: &str| {
            rows.iter()
                .filter(|r| r["corpus"] == json!(name) && r["label"] == json!(label))
                .map(|r| r["sha"].as_str().expect("sha"))
                .collect::<BTreeSet<_>>()
                .len() as u64
        };
        assert_eq!(mine("normal"), u(c, "false_strict"), "{name}: false events");
        assert_eq!(
            mine("normal") + mine("unreviewed"),
            u(c, "false_wide"),
            "{name}: wide events"
        );
        assert_eq!(
            mine("copy"),
            u(c, "abnormal") - u(c, "misses"),
            "{name}: caught positives"
        );
    }
}

/// The wide denominator carries the plan's floor, and the line is
/// enforced exactly as §4.2 states it: a class that no tier reads may
/// record any number, a promoted one may not exceed the gate.
#[test]
fn the_denominator_clears_the_floor_and_the_line_binds_when_promoted() {
    let doc = contract();
    let t = &doc["totals"];
    assert!(
        u(t, "normal") + u(t, "unreviewed") >= 500,
        "the plan's per-500 denominator: {} non-copy events",
        u(t, "normal") + u(t, "unreviewed")
    );
    assert_eq!(over_the_line(&doc), None, "{:?}", over_the_line(&doc));
}

/// None = this document licenses what it claims. Some(why) = a
/// PROMOTED class whose recorded rate is over its own gate.
fn over_the_line(doc: &Value) -> Option<String> {
    if doc["promoted"] != json!(true) {
        return None;
    }
    let t = &doc["totals"];
    let (n, x) = (u(t, "normal") + u(t, "unreviewed"), u(t, "false_wide"));
    let gate = u(doc, "gate_max_percent");
    (x * 100 > n * gate).then(|| format!("promoted with {x} false of {n} — over {gate}%"))
}

/// The implication is not decoration: a promoted document at 2 % is
/// refused by name, and the same numbers unpromoted are not.
#[test]
fn a_promoted_document_over_the_line_is_refused_by_name() {
    let doc = |promoted: bool| {
        json!({"promoted": promoted, "gate_max_percent": 1,
               "totals": {"normal": 500, "unreviewed": 500, "false_wide": 20}})
    };
    assert_eq!(
        over_the_line(&doc(false)),
        None,
        "unpromoted records freely"
    );
    let why = over_the_line(&doc(true)).expect("a promoted overrun is refused");
    assert!(
        why.contains("20 false of 1000") && why.contains("over 1%"),
        "{why}"
    );
}

/// The ledger doc and the contract are one number, not two. A gate
/// that only ever read the generator would let a doc keep its
/// placeholders (or a stale figure) and ship green.
#[test]
fn the_ledger_doc_quotes_the_contract() {
    let doc = contract();
    let text = crate::facts::read(&repo_root(), LEDGER);
    let table: Vec<&str> = text
        .lines()
        .skip_while(|l| l.trim() != HEADER)
        .skip(2)
        .take_while(|l| l.starts_with('|'))
        .collect();
    let corpora = doc["corpora"].as_array().expect("corpora");
    assert_eq!(table.len(), 4, "{LEDGER}: the 折算与门 table");
    for (line, row) in table
        .iter()
        .zip(corpora.iter().chain(std::iter::once(&doc["totals"])))
    {
        assert_eq!(
            *line,
            table_row(row),
            "{LEDGER}: every printed count, rate and interval"
        );
    }
}

/// The interval arithmetic is the sibling ledger's, pinned against
/// six reference intervals docs/FPR-TOMBSTONE.md already published. A second
/// statistic in the same repository is a second authority.
#[test]
fn the_interval_reproduces_the_published_tombstone_ledger() {
    const PUBLISHED: [(usize, usize, f64, f64); 6] = [
        (400, 0, 0.0, 0.918),
        (530, 3, 0.117, 1.645),
        (530, 7, 0.533, 2.702),
        (537, 1, 0.005, 1.033),
        (537, 6, 0.411, 2.416),
        (936, 4, 0.117, 1.091),
    ];
    checks::published_intervals(&PUBLISHED);
    assert_eq!(permille(3, 530), 5, "exact integer per-mille");
    assert_eq!(
        permille(0, 0),
        0,
        "an empty denominator is 0, never a panic"
    );
    let (lo, hi) = clopper_pearson(1000, 1000);
    assert!(
        (lo - 100.0 * 0.025f64.powf(0.001)).abs() < 1e-9,
        "all-success lower tail"
    );
    assert_eq!(hi, 100.0, "all-success upper endpoint");
}

/// This batch is EVIDENCE, not a promotion: the Stop audit resolves
/// its class with the route default `observe`, and an unset
/// `[guard] mode` must still land there.
#[test]
fn the_audit_class_is_still_unpromoted() {
    let doc = contract();
    assert_eq!(doc["promoted"], json!(false), "the contract claims no tier");
    assert_eq!(
        codeeraser::config::Guard::default().tier("observe"),
        "observe",
        "an unset [guard] mode still leaves this class at observe"
    );
}
