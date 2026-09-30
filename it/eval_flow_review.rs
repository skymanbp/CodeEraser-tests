//! The v2.31 flow exams' blind-review gates (booklet §5.5 「盲判」;
//! registry docs/EVAL-SET-FLOW.md): the batches are a pure function of
//! the frozen sample, one corpus and at most BATCH_MAX questions each,
//! carrying no rank; the assembler accepts a complete answer set for
//! every exam and names every defect of a broken one; an exam's review
//! doc is on disk exactly when its stage is past Sampled, and then it
//! verifies and refuses every forgery of the battery. The answer sets
//! here are synthetic — the gate's own, never an auditor's. No git, no
//! corpus clone.

use crate::eval_flow_parts::batches::{self, BATCH_MAX, CANNOT_TELL, answer_file, plan, render};
use crate::eval_flow_parts::review::{Filed, REVIEWS, assemble, verify_review};
use crate::eval_flow_parts::{EXAMS, FlowExam, Stage, each_filed, exam, tamper};
use crate::eval_support::load;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The clone base and batch directory the pure renderings are read at.
const AT: (&str, &str) = ("/corpora", "/batches");

/// An answer set: each batch number with its lines.
type Set = BTreeMap<u64, Vec<String>>;

/// One edit of an answer set and the refusal it must draw.
type Edit<'a> = (&'a dyn Fn(&mut Set), &'a str);

/// A synthetic answer set: every question answered in order, the two
/// words alternating and every seventh `cannot_tell`.
fn answers(exam: &FlowExam, sample: &Value) -> Set {
    let mut out = BTreeMap::new();
    for (i, b) in plan(exam, sample).iter().enumerate() {
        let lines = b.rows.iter().enumerate().map(|(j, r)| {
            let words = batches::words(r["kind"].as_u64().expect("kind"));
            let truth = if (i + j) % 7 == 0 {
                CANNOT_TELL
            } else {
                words[j % 2]
            };
            let reason = "a synthetic answer of the gate's own battery";
            json!({"id": r["audit"], "truth": truth, "reason": reason}).to_string()
        });
        out.insert(b.n, lines.collect());
    }
    out
}

fn manifest(exam: &FlowExam, sample: &Value) -> Value {
    let files = render(exam, sample, AT.0, AT.1);
    let (_, text) = files.last().expect("manifest");
    serde_json::from_str(text).expect("manifest json")
}

/// The gate's synthetic review of an exam's sample: every question
/// answered by `answers` and assembled — the review the precision
/// battery reads for an exam whose generation has none filed yet.
pub(crate) fn synthetic_review(exam: &FlowExam, sample: &Value) -> Value {
    assembled(exam, sample, &answers(exam, sample)).expect("synthetic")
}

/// Assemble a synthetic answer set.
fn assembled(exam: &FlowExam, sample: &Value, set: &Set) -> Result<Value, Vec<String>> {
    let file = |n: u64| set.get(&n).map(|l| l.join("\n") + "\n");
    let auditor = "the gate's own synthetic answer set, no auditor's";
    let manifest = manifest(exam, sample);
    let filed = Filed {
        manifest: &manifest,
        file: &file,
    };
    assemble(exam, sample, filed, auditor, json!({"ce": "synthetic"}))
}

/// Two renderings are byte-identical; the plan cuts every sampled row
/// into exactly one batch of one corpus, numbered 1.., at most
/// BATCH_MAX each; a batch never shows a rank; the language readings
/// are in a batch exactly from the second generation on.
#[test]
fn flow_batches_render_purely() {
    for exam in EXAMS.iter() {
        let sample = exam.sample();
        let once = render(exam, &sample, AT.0, AT.1);
        assert_eq!(
            once,
            render(exam, &sample, AT.0, AT.1),
            "{}: impure",
            exam.lang
        );
        let batches = plan(exam, &sample);
        let total: usize = batches.iter().map(|b| b.rows.len()).sum();
        assert_eq!(
            total,
            sample["rows"].as_array().expect("rows").len(),
            "{}: rows",
            exam.lang
        );
        for (i, b) in batches.iter().enumerate() {
            batch_holds(exam, i, b, &once[i].1);
        }
    }
}

/// Batch i of an exam's plan, as rendered: numbered i + 1, at most
/// BATCH_MAX rows of its one corpus, no rank shown, its answer file
/// named, the language readings in it exactly from the second
/// generation on.
fn batch_holds(exam: &FlowExam, i: usize, b: &batches::Batch, text: &str) {
    let at = format!("{}: batch {}", exam.lang, b.n);
    assert!(b.n == i as u64 + 1 && b.rows.len() <= BATCH_MAX, "{at}");
    for r in &b.rows {
        assert_eq!(r["corpus"], json!(b.corpus), "{at} mixes corpora");
        let rank = r["rank"].as_str().expect("rank");
        assert!(!text.contains(rank), "{at} shows a rank");
    }
    let answers = answer_file(AT.1, exam.lang, b.n);
    assert!(text.contains(&answers), "{at}: answer file");
    let read = text.contains(batches::READINGS_TEXT);
    assert_eq!(read, exam.generation >= 2, "{at}: readings");
}

/// A complete synthetic answer set assembles for every exam and the
/// doc verifies.
#[test]
fn flow_review_assembly_accepts_every_exam() {
    for exam in EXAMS.iter() {
        let sample = exam.sample();
        let doc = assembled(exam, &sample, &answers(exam, &sample))
            .unwrap_or_else(|e| panic!("{}: {e:?}", exam.lang));
        verify_review(exam, &sample, &doc);
    }
}

/// Row 0 of batch 1 replaced, `expected refusal | line` ($ID the row's
/// id, $LONG a 241-character reason); every case also leaves a
/// question unanswered or refused, which the reading names too.
const BROKEN: &str = r#"
not in this batch | {"id": "0000000000000000000000000000000000000000000000000000000000000000", "truth": "cannot_tell", "reason": "a reason long enough to pass"}
is not an answer | {"id": "$ID", "truth": "maybe", "reason": "a reason long enough to pass"}
characters | {"id": "$ID", "truth": "cannot_tell", "reason": "too short"}
characters | {"id": "$ID", "truth": "cannot_tell", "reason": "$LONG"}
fields | {"id": "$ID", "truth": "cannot_tell", "reason": "a reason long enough to pass", "x": 1}
fields | {"id": "$ID", "truth": "cannot_tell"}
not a string | {"id": "$ID", "truth": 1, "reason": "a reason long enough to pass"}
not a JSON object | hello
not a JSON object | $NOLINE
"#;

/// The refusals of one broken answer set, joined.
fn refusals(exam: &FlowExam, sample: &Value, edit: impl Fn(&mut Set)) -> String {
    let mut set = answers(exam, sample);
    edit(&mut set);
    assembled(exam, sample, &set)
        .expect_err("a broken answer set must refuse")
        .join("\n")
}

/// Batch 1's first answer written again at its end.
fn twice(set: &mut Set) {
    let lines = set.get_mut(&1).expect("batch 1");
    lines.push(lines[0].clone());
}

/// Every defect is refused and named by batch and line or id.
#[test]
fn flow_review_assembly_names_every_refusal() {
    let exam = exam("python");
    let sample = exam.sample();
    let id = plan(exam, &sample)[0].rows[0]["audit"]
        .as_str()
        .expect("audit")
        .to_string();
    for case in BROKEN.lines().filter(|l| !l.is_empty()) {
        let (want, line) = case.split_once(" | ").expect("case");
        let line = line
            .replace("$ID", &id)
            .replace("$LONG", &"x".repeat(241))
            .replace("$NOLINE", "");
        let said = refusals(exam, &sample, |s| {
            s.get_mut(&1).expect("batch 1")[0] = line.clone()
        });
        assert!(
            said.contains("batch 1 line 1: ") && said.contains(want),
            "{case}: {said}"
        );
    }
    let first = plan(exam, &sample)[0].rows.len() + 1;
    let again = format!("batch 1 line {first}: id {id} answered twice");
    let unanswered = format!("batch 1: id {id} unanswered");
    let edits: [Edit; 3] = [
        (&twice, &again),
        (&|s| drop(s.get_mut(&1).expect("1").remove(0)), &unanswered),
        (&|s| drop(s.remove(&2)), "batch 2: no answers file"),
    ];
    for (edit, want) in edits {
        let said = refusals(exam, &sample, edit);
        assert!(said.contains(want), "{want}: {said}");
    }
    let mut stale = manifest(exam, &sample);
    stale["batches"][0]["ids"][0] = json!("forged");
    let file = |_: u64| None;
    let filed = Filed {
        manifest: &stale,
        file: &file,
    };
    let err = assemble(exam, &sample, filed, "x", json!({})).expect_err("stale manifest");
    assert!(err[0].starts_with("manifest"), "{err:?}");
}

/// A review doc is on disk exactly when the exam's stage is past
/// Sampled; a filed one verifies and refuses the tamper battery.
#[test]
fn flow_reviews_filed_by_stage() {
    each_filed(&REVIEWS, Stage::Audited, |exam, path| {
        let (sample, doc) = (exam.sample(), load(path));
        verify_review(exam, &sample, &doc);
        tamper::assert_flow_review_tampering(exam, &sample, &doc);
    });
}

/// The battery on synthetic reviews of a one-corpus and the two-corpus
/// exam, so it runs before any auditor's doc is filed.
#[test]
fn a_tampered_flow_review_is_refused() {
    for lang in ["python", "rust"] {
        let exam = exam(lang);
        let sample = exam.sample();
        let doc = synthetic_review(exam, &sample);
        tamper::assert_flow_review_tampering(exam, &sample, &doc);
    }
}
