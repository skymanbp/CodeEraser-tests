use super::*;
use serde_json::json;

fn ask(query: &str) -> Ask {
    Ask {
        rules: None,
        query: Some(query.into()),
        why: false,
    }
}

fn dead_f() -> (Program, Labels) {
    let program = Program::lex(PRELUDE, None, Some("?- dead(F).")).unwrap();
    let labels = Labels {
        nodes: vec!["a.rs".into(), "b.rs".into()],
        ..Labels::default()
    };
    (program, labels)
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

#[test]
fn a_lexical_fault_is_error_zero_at_its_place_and_no_judgment_before_any_core() {
    let r = run(
        Path::new("."),
        None,
        "no-such-core",
        &ask("mention(\"x, F)"),
    )
    .unwrap();
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
    assert_eq!(
        (
            r.tokens,
            r.clauses,
            r.prelude,
            r.goals.len(),
            r.violations()
        ),
        (0, 0, 0, 0, 0)
    );
    let doc = report_json(&r, false);
    let (schema, rules_schema) = (
        doc["schema"].clone(),
        report_json(&r, true)["schema"].clone(),
    );
    assert_eq!(
        (schema, rules_schema),
        (json!(SCHEMA_ID), json!(RULES_SCHEMA_ID))
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
    let r = run(Path::new("."), None, "no-such-core", &ask("in(F, \"[\")")).unwrap();
    assert_eq!(r.errors.len(), 1, "{:?}", r.errors);
    assert_eq!(
        (r.errors[0].at.as_str(), r.errors[0].code),
        ("query 1:10", 0)
    );
    assert!(!r.errors[0].what.is_empty());
    assert!(!r.judged() && r.tokens > 0);
}

#[test]
fn label_reads_the_cores_errors_before_any_goal() {
    let (program, labels) = dead_f();
    let at = program.tokens.len() - 3;
    let erring = wire::Judged {
        errors: vec![(at, 2)],
        ..Default::default()
    };
    let mut r = empty(&ask("dead(F)"), None);
    label(&mut r, &program, &labels, erring).unwrap();
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
    let degraded = wire::Judged {
        degraded: Some("query_too_large".into()),
        ..Default::default()
    };
    let mut r = empty(&ask("dead(F)"), None);
    label(&mut r, &program, &labels, degraded).unwrap();
    assert_eq!(
        (r.degraded.as_deref(), r.judged()),
        (Some("query_too_large"), false)
    );
}

#[test]
fn label_reads_a_healthy_answer_back_through_the_tables() {
    let (program, labels) = dead_f();
    let row = |node, parent, rule, pred| wire::ProofRow {
        goal: 0,
        answer: 0,
        node,
        parent,
        rule,
        pred,
        args: vec![1],
    };
    let healthy = wire::Judged {
        goals: vec![(0, vec![0])],
        preds: BTreeMap::from([(1002, vec![0])]),
        answers: vec![(0, vec![1])],
        proof: vec![row(0, -1, -1, -1), row(1, 0, 3, 1002), row(2, 1, -1, 1)],
        counts: wire::COUNTS.iter().map(|k| (*k, 1)).collect(),
        ..Default::default()
    };
    let mut r = empty(&ask("dead(F)"), None);
    label(&mut r, &program, &labels, healthy).unwrap();
    assert!(r.judged());
    assert_eq!(
        r.goals,
        [GoalFace {
            goal: 0,
            kind: "query",
            name: None,
            columns: vec!["F".into()],
            sorts: vec!["node"],
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
