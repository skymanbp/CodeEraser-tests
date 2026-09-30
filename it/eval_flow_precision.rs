//! The v2.31 flow exams' precision gates (booklet analysis-track.md
//! §5.5 「精度册」 and §13 item 25; registry docs/EVAL-SET-FLOW.md): an
//! exam's precision doc is on disk exactly when its stage is Scored,
//! and then it verifies against its sample and review and refuses the
//! tamper battery; the product's judged mask holds a language's bit
//! exactly when its doc reads `judged`; the gate reads four states,
//! pinned on hand tallies. The battery also runs on synthetic docs of
//! the gate's own (never the product's answers), so it holds before any
//! doc is filed. No git, no corpus clone, no core.

use crate::eval_flow_parts::draw::text;
use crate::eval_flow_parts::review::REVIEWS;
use crate::eval_flow_parts::{EXAMS, FlowExam, Stage, each_filed, exam};
use crate::eval_support::{Forgery, assert_forgeries_refused, load};
use crate::flow_precision::{
    Answers, GATED, PRECISIONS, assemble, gate_of, gates, judged, per_kind, verdict,
    verify_precision,
};
use serde_json::{Value, json};

/// A precision doc of the gate's own over the exam's filed review: row
/// 0 unjudged, kinds 0 and 2 never flagged, kind 1 answered as its
/// truth, kind 3 against it.
fn synthetic(exam: &FlowExam, review: &Value) -> Value {
    let rows = review["rows"].as_array().expect("rows");
    let flagged = rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let positive = ["unreachable", "dead", "unread"].contains(&r["truth"].as_str()?);
            match r["kind"].as_u64()? {
                _ if i == 0 => None,
                0 | 2 => Some(false),
                1 => Some(positive),
                _ => Some(!positive),
            }
        })
        .collect();
    let unit = &rows[0];
    let unjudged = vec![json!({
        "corpus": unit["corpus"], "path": unit["path"], "unit": unit["unit"],
        "reason": "synthetic: the gate's own unjudged row",
    })];
    let stamp = json!({"ce": "synthetic", "commit": "synthetic", "dirty": false});
    assemble(exam, review, &Answers { flagged, unjudged }, stamp)
}

/// A precision doc is on disk exactly when the exam's stage is Scored;
/// a filed one verifies and refuses the tamper battery.
#[test]
fn flow_precision_docs_filed_by_stage() {
    each_filed(&PRECISIONS, Stage::Scored, |exam, path| {
        let (sample, review) = (exam.sample(), REVIEWS.load(exam, exam.lang));
        let doc = load(path);
        verify_precision(exam, &sample, &review, &doc);
        assert_flow_precision_tampering(exam, &sample, &review, &doc);
    });
}

/// `codeeraser::flow::judged_mask()` holds an exam language's bit
/// exactly when its precision doc is filed and reads judged (tsx and
/// typescript each their own bit), and no bit of another language.
#[test]
fn the_judged_mask_is_the_precision_docs() {
    let mask = codeeraser::flow::judged_mask();
    let mut known = 0i64;
    for exam in EXAMS.iter() {
        let bit = 1i64 << exam.language() as i64;
        known |= bit;
        let judged = exam.stage >= Stage::Scored
            && load(&PRECISIONS.file(exam, exam.lang))["judged"] == json!(true);
        assert_eq!(
            mask & bit != 0,
            judged,
            "{}: mask bit vs the precision doc's judged",
            exam.lang
        );
    }
    assert_eq!(mask & !known, 0, "a judged-mask bit of no exam language");
}

/// `fp tp fn state`: a false positive fails; none with a true positive
/// passes; none with no positive to find (no rows at all among them) is
/// vacuous; none with every positive missed is silent — never vacuous,
/// or more evidence would read better than less.
const STATES: &str = "\
0 0 0 vacuous
0 0 3 silent
0 1 0 pass
0 2 5 pass
1 0 0 fail
1 4 2 fail";

#[test]
fn the_flow_gate_reads_four_states() {
    for line in STATES.lines() {
        let w: Vec<&str> = line.split(' ').collect();
        let n = |i: usize| w[i].parse::<u64>().expect("count");
        let tally = json!({"fp": n(0), "tp": n(1), "fn": n(2)});
        assert_eq!(gate_of(&tally), w[3], "{line}");
    }
}

/// The exam's filed review, or the review gate's synthetic one while
/// its generation has none filed (rust, the one two-corpus exam, is
/// at its second generation from commit F on).
fn review_of(exam: &FlowExam, sample: &Value) -> Value {
    match exam.stage >= Stage::Audited {
        true => REVIEWS.load(exam, exam.lang),
        false => crate::eval_flow_review::synthetic_review(exam, sample),
    }
}

/// The battery on synthetic docs over the python review (kind 0
/// vacuous, 1 pass, 2 silent) and the two-corpus rust review.
#[test]
fn a_tampered_flow_precision_is_refused() {
    for lang in ["python", "rust"] {
        let exam = exam(lang);
        let sample = exam.sample();
        let review = review_of(exam, &sample);
        let doc = synthetic(exam, &review);
        verify_precision(exam, &sample, &review, &doc);
        assert_flow_precision_tampering(exam, &sample, &review, &doc);
    }
    let python = synthetic(exam("python"), &REVIEWS.load(exam("python"), "python"));
    let want = json!({"0": "vacuous", "1": "pass", "2": "silent"});
    assert_eq!(python["gate"], want, "the synthetic python gates");
    assert_eq!(python["judged"], json!(false), "silent is not admitted");
}

/// The precision doc's tamper battery, six forms through the forgery
/// frame: a flipped verdict, a flipped answer (its verdict left as it
/// was), a forged gate, a forged judged bit, a dropped row, and a
/// silent kind read as vacuous — staged when the doc holds no silent
/// kind by muting a kind's true answers with every tally re-derived,
/// so only the gate lies (unstageable on a doc whose gated kinds hold
/// no positive: there is nothing a mute could hide).
fn assert_flow_precision_tampering(
    exam: &FlowExam,
    sample: &Value,
    review: &Value,
    pristine: &Value,
) {
    let check = |doc: &Value| verify_precision(exam, sample, review, doc);
    let mut forgeries: Vec<Forgery> = vec![
        (
            &|d| flip(&mut d["rows"][0]["verdict"], "tp", "fp"),
            "a flipped verdict",
        ),
        (&flip_answer, "a flipped answer"),
        (
            &|d| flip(&mut d["gate"]["0"], "fail", "pass"),
            "a forged gate",
        ),
        (
            &|d| d["judged"] = json!(d["judged"] != json!(true)),
            "a forged judged bit",
        ),
        (&|d| drop(rows(d).remove(0)), "a dropped row"),
    ];
    if stageable(pristine).is_some() {
        forgeries.push((&silent_as_vacuous, "a silent kind read as vacuous"));
    }
    assert_forgeries_refused(pristine, &forgeries, &check);
}

fn rows(doc: &mut Value) -> &mut Vec<Value> {
    doc["rows"].as_array_mut().expect("rows")
}

/// `a` when the value is not `a`, else `b`.
fn flip(value: &mut Value, a: &str, b: &str) {
    *value = json!(if *value == json!(a) { b } else { a });
}

/// Row 0's answer turned over (none read as true), its verdict kept.
fn flip_answer(doc: &mut Value) {
    let row = &mut doc["rows"][0];
    row["flagged"] = json!(row["flagged"] != json!(true));
}

/// A gated kind that is silent, or holds a positive a mute would hide.
fn stageable(doc: &Value) -> Option<String> {
    GATED.iter().map(|k| k.to_string()).find(|k| {
        doc["gate"][k] == json!("silent") || doc["per_kind"][k]["positives"].as_u64() > Some(0)
    })
}

/// The stageable kind muted (every true answer false, verdicts and
/// tallies re-derived), then its gate written as vacuous and the
/// judged bit read off the forged gates.
fn silent_as_vacuous(doc: &mut Value) {
    let kind = stageable(doc).expect("a stageable kind");
    for row in rows(doc).iter_mut() {
        if row["kind"].as_u64().map(|k| k.to_string()) == Some(kind.clone())
            && row["flagged"] == json!(true)
        {
            row["flagged"] = json!(false);
            row["verdict"] = json!(verdict(text(row, "truth"), Some(false)));
        }
    }
    doc["per_kind"] = per_kind(rows(doc));
    doc["gate"] = gates(&doc["per_kind"]);
    doc["gate"][&kind] = json!("vacuous");
    doc["judged"] = json!(judged(&doc["gate"]));
}
