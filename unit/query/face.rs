// The query document (plan v2.32 step 3: the request this side sends,
// laid out by the core this process measures with and bound back): the
// program's own faults, the core's errors, a degraded answer and a
// healthy one labelled through the tables.
use super::*;
use crate::query::report::{AnswerFace, ErrorFace, GoalFace, Report};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;

fn ask(query: &str) -> Ask {
    Ask {
        rules: None,
        query: Some(query.into()),
        why: false,
    }
}

fn core() -> String {
    crate::daemon::judge::core_bin().expect("a core")
}

fn dead_f() -> (Program, Labels) {
    let program = Program::lex(PRELUDE, None, Some("?- dead(F).")).unwrap();
    let labels = Labels {
        nodes: vec!["a.rs".into(), "b.rs".into()],
        ..Labels::default()
    };
    (program, labels)
}

/// `?- dead(F).` answered by `j`, laid out and read.
fn laid_out(j: wire::Judged) -> (Program, Report) {
    let (program, labels) = dead_f();
    let mut names = Names {
        rules_file: None,
        query: Some("?- dead(F).".into()),
        program: None,
        heads: columns::goals(&program),
        labels: Some(labels),
        why: crate::document::Why::default(),
    };
    let req = Request::new("query").fact("tokens", program.tokens.len());
    names.program = Some(Program::lex(PRELUDE, None, Some("?- dead(F).")).unwrap());
    let req = answered(req, &mut names, j).unwrap();
    let doc = document::assemble(&core(), finish(req, &names), &names).unwrap();
    (program, Report::deserialize(&doc).unwrap())
}

/// `?-` in front and `.` behind, each only when the text lacks it.
#[test]
fn a_question_is_wrapped_once_into_query_form() {
    let wrapped: Vec<String> = ["dead(F)", "?- dead(F).", "  dead(F).  ", "?- dead(F)"]
        .iter()
        .map(|q| question(q))
        .collect();
    assert!(wrapped.iter().all(|w| w == "?- dead(F)."), "{wrapped:?}");
}

/// A question asked of this directory, its document and the reader.
fn asked(query: &str) -> (Value, Report) {
    let doc = run(Path::new("."), None, &core(), &ask(query)).unwrap();
    let r = Report::deserialize(&doc).unwrap();
    (doc, r)
}

#[test]
fn a_lexical_fault_is_error_zero_at_its_place_and_no_judgment() {
    let (doc, r) = asked("mention(\"x, F)");
    assert_eq!(
        r.errors,
        [ErrorFace {
            at: "query 1:12".into(),
            token: None,
            code: 0,
            what: "unterminated string".into(),
        }]
    );
    assert!(!r.judged());
    let p = &r.program;
    assert_eq!(
        (
            p.tokens,
            p.clauses,
            p.prelude,
            r.goals.len(),
            r.violations()
        ),
        (0, 0, 0, 0, 0)
    );
    let rules = Ask {
        query: None,
        rules: Some(("ce.rules".into(), "assert x(F) :- mention(\"x, F).".into())),
        why: false,
    };
    let rules_doc = run(Path::new("."), None, &core(), &rules).unwrap();
    assert_eq!(
        (&doc["schema"], &rules_doc["schema"]),
        (&json!(SCHEMA_ID), &json!(RULES_SCHEMA_ID))
    );
    assert_eq!(
        (
            &doc["errors"][0]["code"],
            &doc["program"]["query"],
            &doc["counts"]["answers"]
        ),
        (&json!(0), &json!("?- mention(\"x, F)."), &json!(0))
    );
}

#[test]
fn an_unreadable_glob_is_a_program_error_at_the_glob_token() {
    let (_, r) = asked("in(F, \"[\")");
    assert_eq!(r.errors.len(), 1, "{:?}", r.errors);
    assert_eq!(
        (r.errors[0].at.as_str(), r.errors[0].code),
        ("query 1:10", 0)
    );
    assert!(!r.errors[0].what.is_empty());
    assert!(!r.judged() && r.program.tokens > 0);
}

#[test]
fn the_cores_errors_come_before_any_goal() {
    let at = dead_f().0.tokens.len() - 3;
    let (program, r) = laid_out(wire::Judged {
        errors: vec![(at, 2)],
        ..Default::default()
    });
    assert_eq!(
        r.errors,
        [ErrorFace {
            at: program.locate(at),
            token: Some(at),
            code: 2,
            what: "unknown predicate".into(),
        }]
    );
    assert!(r.goals.is_empty() && !r.judged());
    let (_, r) = laid_out(wire::Judged {
        degraded: Some("query_too_large".into()),
        ..Default::default()
    });
    assert_eq!(
        (r.degraded.as_deref(), r.judged()),
        (Some("query_too_large"), false)
    );
}

#[test]
fn a_healthy_answer_reads_back_through_the_tables() {
    let row = |node, parent, rule, pred| wire::ProofRow {
        goal: 0,
        answer: 0,
        node,
        parent,
        rule,
        pred,
        args: vec![1],
    };
    let (_, r) = laid_out(wire::Judged {
        goals: vec![(0, vec![0])],
        preds: BTreeMap::from([(1002, vec![0])]),
        answers: vec![(0, vec![1])],
        proof: vec![row(0, -1, -1, -1), row(1, 0, 3, 1002), row(2, 1, -1, 1)],
        counts: wire::COUNTS.iter().map(|k| (*k, 1)).collect(),
        ..Default::default()
    });
    assert!(r.judged());
    assert_eq!(
        r.goals,
        [GoalFace {
            goal: 0,
            kind: "query".into(),
            name: None,
            columns: vec!["F".into()],
            sorts: vec!["node".into()],
        }]
    );
    assert_eq!(
        r.answers,
        [AnswerFace {
            goal: 0,
            values: vec!["b.rs".into()],
            raw: vec![json!(1)],
        }]
    );
    let chain: Vec<(&str, &str)> = r
        .proof
        .iter()
        .map(|p| (p.pred.as_str(), p.args[0].as_str()))
        .collect();
    assert_eq!(chain, [("?-", "b.rs"), ("dead", "b.rs"), ("file", "b.rs")]);
    assert_eq!(r.counts["answers"], 1);
}
