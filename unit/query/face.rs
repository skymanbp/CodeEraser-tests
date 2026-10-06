// The query document (plan v2.32 step 3: the request this side sends,
// laid out by the core this process measures with and bound back): the
// program's own faults, the core's errors, a degraded answer and a
// healthy one labelled through the tables.
use super::*;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

// The query document's reader: these legs' own (the console that once
// read it is the core's since plan v2.32 step 5).
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
struct ProgramFace {
    rules_file: Option<String>,
    query: Option<String>,
    why: bool,
    tokens: usize,
    clauses: usize,
    prelude: usize,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
struct GoalFace {
    goal: usize,
    kind: String,
    name: Option<String>,
    columns: Vec<String>,
    sorts: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
struct AnswerFace {
    goal: usize,
    values: Vec<String>,
    raw: Vec<Value>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
struct ProofFace {
    goal: usize,
    answer: usize,
    node: i64,
    parent: i64,
    rule: i64,
    pred: String,
    args: Vec<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
struct ErrorFace {
    at: String,
    token: Option<usize>,
    code: i64,
    what: String,
}

/// The bound document, read.
#[derive(Deserialize)]
struct Report {
    program: ProgramFace,
    goals: Vec<GoalFace>,
    answers: Vec<AnswerFace>,
    proof: Vec<ProofFace>,
    errors: Vec<ErrorFace>,
    counts: BTreeMap<String, u64>,
    degraded: Option<String>,
}

impl Report {
    fn violations(&self) -> u64 {
        self.counts.get("violations").copied().unwrap_or(0)
    }

    /// Whether the document carries a judgment: no lexical or program
    /// error, and a core that answered.
    fn judged(&self) -> bool {
        self.errors.is_empty() && self.degraded.is_none()
    }
}

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

/// `?- dead(F).` as the core lexes it, and two labelled nodes.
fn dead_f() -> (Lexed, Labels) {
    let mut link = document::open(&core()).expect("a core link");
    let texts = lexed::texts(PRELUDE, None, Some("?- dead(F)."));
    let Ok(Lex::Program(program)) = lexed::lex(&mut link, &texts) else {
        panic!("the question lexes");
    };
    let labels = Labels {
        nodes: vec!["a.rs".into(), "b.rs".into()],
        ..Labels::default()
    };
    (program, labels)
}

/// `?- dead(F).` answered by `j`, laid out and read.
fn laid_out(j: wire::Judged) -> (Lexed, Report) {
    let (program, labels) = dead_f();
    let req = Request::new("query").fact("tokens", program.tokens);
    let mut names = Names {
        rules_file: None,
        query: Some("?- dead(F).".into()),
        program: Some(program),
        labels: Some(labels),
        why: crate::document::Why::default(),
    };
    let req = answered(req, &mut names, j).unwrap();
    let doc = document::assemble(&core(), finish(req, &names), &names)
        .unwrap()
        .document;
    (dead_f().0, Report::deserialize(&doc).unwrap())
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
    let doc = run(Path::new("."), None, &core(), &ask(query))
        .unwrap()
        .document;
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
    let rules_doc = run(Path::new("."), None, &core(), &rules).unwrap().document;
    assert_eq!(
        (&doc["schema"], &rules_doc["schema"]),
        (
            &json!("ce.query-report/0.1.0"),
            &json!("ce.rules-report/0.1.0")
        )
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
    let at = dead_f().0.tokens - 3;
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
