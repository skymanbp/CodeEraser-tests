//! The flow exams' precision docs (booklet §5.5 「精度册」 and its
//! four-state gate, §13 item 25): one language's reviewed questions,
//! each with the product's answer (flagged.rs) and its verdict, the
//! per-kind tallies, the gate per non-advisory kind and whether the
//! language enters the judged mask — every number re-derived from the
//! rows by the functions below, for the generator and the verifier
//! alike (G1). The gate reads four states: `fail` (a false positive),
//! `pass` (none, and at least one true positive), `vacuous` (none, and
//! no positive to find: the sample's negatives carry the admission),
//! `silent` (none, yet positives the product missed every one of).
//!   CE_FLOW_LANG=python [CE_FLOW_OUT=<dir>] [CE_FLOW_PRECISION_DRY=1] cargo test --test it -- --ignored flow_precision::flow_precision --nocapture
//! The doc is written on a clean tree only (the verifier refuses a
//! dirty stamp); a dry run prints the tallies and writes nothing.
//! A module of its own beside eval_flow_parts, not inside it: nothing
//! there reads it back, so it joins no import cycle (check axis 6).

pub(crate) mod flagged;

use crate::common::core_bin;
use crate::eval_flow_parts::FlowExam;
use crate::eval_flow_parts::assert_keys;
use crate::eval_flow_parts::batches::CANNOT_TELL;
use crate::eval_flow_parts::draw::text;
use crate::eval_flow_parts::exam;
use crate::eval_flow_parts::generate::filed;
use crate::eval_flow_parts::review::{REVIEWS, corpora, row_list, verify_review};
use crate::eval_lang_parts::Docs;
use crate::eval_lang_parts::generate::env;
use crate::eval_support::generated_from;
use codeeraser::corelink::Link;
use serde_json::{Map, Value, json};

pub use flagged::Answers;

pub const PRECISIONS: Docs = Docs("flow-precision");
pub const PRECISION_SCHEMA: &str = "ce.eval-flow-precision/1.0.0";

/// The review fields a precision row echoes; the row adds `flagged`
/// and `verdict`.
pub const ECHO: &str = "rank audit corpus path unit kind stratum line nth name truth";

/// The kinds the gate reads (kind 3 is advisory: tallied, never gated).
pub const GATED: [u64; 3] = [0, 1, 2];

/// The positive answer of each kind, and the verdicts in tally order.
const POSITIVE: [&str; 3] = ["unreachable", "dead", "unread"];
const VERDICTS: [&str; 6] = ["tp", "fp", "tn", "fn", "unjudged", "cannot_tell"];

const METHOD: &str = "each reviewed question of the language answered by the product: every \
    file a question sits in lowered at the pinned tip (flow::lower) and judged whole by the real \
    core, one request per file (flow::wire::judge); a question maps back through the pool item \
    it was drawn as - kind 0 flagged when an unreachable run covers its statement, kind 1 when \
    the core names its (write, variable), kinds 2 / 3 when the core names its variable. A unit \
    unlowered, refused or dynamic answers nothing (unjudged, listed with its reason); a \
    cannot_tell truth enters no rate. Gate per kind 0 / 1 / 2: fail = a false positive; pass = \
    none and a true positive; vacuous = none and no positive to find; silent = none and every \
    positive missed. The language is judged when all three are pass or vacuous; kind 3 is \
    tallied, never gated.";

/// A row's verdict from the blind truth and the product's answer.
pub fn verdict(truth: &str, flagged: Option<bool>) -> &'static str {
    let Some(flagged) = flagged else {
        return "unjudged";
    };
    if truth == CANNOT_TELL {
        return "cannot_tell";
    }
    match (flagged, POSITIVE.contains(&truth)) {
        (true, true) => "tp",
        (true, false) => "fp",
        (false, true) => "fn",
        (false, false) => "tn",
    }
}

/// `[a, b]` when b is not zero, else null.
fn share(a: u64, b: u64) -> Value {
    if b == 0 { Value::Null } else { json!([a, b]) }
}

/// Per kind (as a string, all four present): the six verdict counts,
/// the positives (tp + fn) and negatives (tn + fp) the sample holds,
/// precision and recall as integer pairs.
pub fn per_kind(rows: &[Value]) -> Value {
    let mut out = Map::new();
    for kind in 0..4u64 {
        let of = rows.iter().filter(|r| r["kind"].as_u64() == Some(kind));
        let mut n: Map<String, Value> =
            VERDICTS.iter().map(|v| (v.to_string(), json!(0))).collect();
        for r in of {
            let v = text(r, "verdict");
            n[v] = json!(n[v].as_u64().expect("count") + 1);
        }
        let c = |v: &str| n[v].as_u64().expect("count");
        let (tp, fp, tn, fl) = (c("tp"), c("fp"), c("tn"), c("fn"));
        n.insert("positives".into(), json!(tp + fl));
        n.insert("negatives".into(), json!(tn + fp));
        n.insert("precision".into(), share(tp, tp + fp));
        n.insert("recall".into(), share(tp, tp + fl));
        out.insert(kind.to_string(), Value::Object(n));
    }
    Value::Object(out)
}

/// One kind's gate from its tallies — the four states.
pub fn gate_of(line: &Value) -> &'static str {
    let c = |v: &str| line[v].as_u64().expect("count");
    match (c("fp"), c("tp"), c("fn")) {
        (1.., _, _) => "fail",
        (0, 1.., _) => "pass",
        (0, 0, 0) => "vacuous",
        (0, 0, 1..) => "silent",
    }
}

/// The gate of each gated kind.
pub fn gates(per_kind: &Value) -> Value {
    let gate: Map<String, Value> = GATED
        .iter()
        .map(|k| (k.to_string(), json!(gate_of(&per_kind[k.to_string()]))))
        .collect();
    Value::Object(gate)
}

/// Whether the language is judged: every gated kind pass or vacuous.
pub fn judged(gate: &Value) -> bool {
    GATED
        .iter()
        .all(|k| matches!(gate[k.to_string()].as_str(), Some("pass" | "vacuous")))
}

/// A review row's echo with the product's answer and the verdict.
fn row_of(review: &Value, flagged: Option<bool>) -> Value {
    let mut row: Map<String, Value> = ECHO
        .split(' ')
        .map(|f| (f.to_string(), review[f].clone()))
        .collect();
    row.insert("flagged".into(), json!(flagged));
    row.insert(
        "verdict".into(),
        json!(verdict(text(review, "truth"), flagged)),
    );
    Value::Object(row)
}

/// The precision doc of a language from its review and the product's
/// answers; `stamp` is the provenance stamp.
pub fn assemble(exam: &FlowExam, review: &Value, answers: &Answers, stamp: Value) -> Value {
    let reviewed = row_list(review);
    assert_eq!(reviewed.len(), answers.flagged.len(), "one answer per row");
    let rows: Vec<Value> = reviewed
        .iter()
        .zip(&answers.flagged)
        .map(|(r, f)| row_of(r, *f))
        .collect();
    let per_kind = per_kind(&rows);
    let gate = gates(&per_kind);
    json!({
        "schema": PRECISION_SCHEMA, "lang": exam.lang, "generation": exam.generation,
        "corpora": corpora(exam), "rows": rows, "per_kind": per_kind,
        "judged": judged(&gate), "gate": gate, "unjudged_reasons": answers.unjudged,
        "generated_from": stamp, "method": METHOD,
    })
}

/// A unit's address in an unjudged row or reason: corpus, path, nth.
fn unit_of(row: &Value) -> Value {
    json!({"corpus": row["corpus"], "path": row["path"], "unit": row["unit"]})
}

/// The unjudged rows' units, in row order, each once.
fn unjudged_units(rows: &[Value]) -> Vec<Value> {
    let mut out: Vec<Value> = Vec::new();
    for unit in rows.iter().filter(|r| r["flagged"].is_null()).map(unit_of) {
        if !out.contains(&unit) {
            out.push(unit);
        }
    }
    out
}

/// Every row the review's row re-derived with its own answer: the echo
/// field for field and the verdict, in the review's order.
fn verify_rows(lang: &str, review: &Value, doc: &Value) {
    let rows = row_list(doc);
    let want = row_list(review);
    assert_eq!(rows.len(), want.len(), "{lang}: precision row count");
    for (i, r) in want.iter().enumerate() {
        let at = format!("{lang}/{}", text(r, "audit"));
        let flagged = &rows[i]["flagged"];
        let answer = flagged.is_boolean() || flagged.is_null();
        assert!(answer, "{at}: flagged is no answer");
        assert_eq!(rows[i], row_of(r, flagged.as_bool()), "{at}: row drifted");
    }
}

/// Everything but the stamp's cleanliness: the envelope, the rows, the
/// tallies, the gates and the judged bit re-derived, the unjudged units
/// named in row order with a reason each.
pub fn verify_body(exam: &FlowExam, sample: &Value, review: &Value, doc: &Value) {
    let lang = exam.lang;
    verify_review(exam, sample, review);
    let want = json!({
        "schema": PRECISION_SCHEMA, "lang": lang, "generation": exam.generation,
        "corpora": corpora(exam), "method": METHOD,
    });
    assert_keys(lang, doc, &want);
    verify_rows(lang, review, doc);
    let per_kind = per_kind(row_list(doc));
    assert_eq!(doc["per_kind"], per_kind, "{lang}: per_kind drifted");
    let gate = gates(&per_kind);
    assert_eq!(doc["gate"], gate, "{lang}: gate drifted");
    assert_eq!(
        doc["judged"],
        json!(judged(&gate)),
        "{lang}: judged drifted"
    );
    let listed = doc["unjudged_reasons"]
        .as_array()
        .expect("unjudged_reasons");
    let units: Vec<Value> = listed.iter().map(unit_of).collect();
    assert_eq!(
        units,
        unjudged_units(row_list(doc)),
        "{lang}: unjudged units"
    );
    let reasoned = |u: &Value| u["reason"].as_str().is_some_and(|r| !r.is_empty());
    assert!(
        listed.iter().all(reasoned),
        "{lang}: an unjudged unit without its reason"
    );
}

/// One precision doc against its exam, sample and review: the body,
/// and a stamp of a clean tree at a commit.
pub fn verify_precision(exam: &FlowExam, sample: &Value, review: &Value, doc: &Value) {
    verify_body(exam, sample, review, doc);
    let from = &doc["generated_from"];
    assert!(from["commit"].is_string(), "{}: no commit", exam.lang);
    assert_eq!(
        from["dirty"],
        json!(false),
        "{}: generated on a dirty tree",
        exam.lang
    );
}

#[test]
#[ignore = "reads the pinned corpus clones and asks the real core"]
fn flow_precision() {
    let exam = exam(&env("CE_FLOW_LANG"));
    let (sample, review) = (exam.sample(), REVIEWS.load(exam, exam.lang));
    let (mut link, _) = Link::open(&core_bin()).expect("open core");
    let answers = flagged::answers(exam, &review, &mut link);
    let doc = assemble(exam, &review, &answers, generated_from());
    verify_body(exam, &sample, &review, &doc);
    println!(
        "{}: per_kind {}\ngate {} judged {}\nunjudged {}",
        exam.lang, doc["per_kind"], doc["gate"], doc["judged"], doc["unjudged_reasons"]
    );
    if std::env::var_os("CE_FLOW_PRECISION_DRY").is_some() {
        let missed = row_list(&doc)
            .iter()
            .filter(|r| matches!(text(r, "verdict"), "fp" | "fn"));
        missed.for_each(|r| println!("{} {r}", text(r, "verdict")));
        println!("dry run: nothing written");
        return;
    }
    verify_precision(exam, &sample, &review, &doc);
    filed(&PRECISIONS.file(exam, exam.lang), &doc);
}
