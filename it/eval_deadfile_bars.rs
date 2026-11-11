//! The dead-file exam's review side on a synthetic fixture (lane
//! v235_licence, phase 2A (b)): BARS §1's evidence check, addendum 1's
//! tag, second readings and settlement, BARS §3's conventions. No real
//! answer is read: the tree, the answers and the classes are made here.
//! §4 and the builder: eval_deadfile_bars_four.rs.

use crate::eval_deadfile_parts::audit::conventions::{Docs, evaluate};
use crate::eval_deadfile_parts::audit::review::{
    Readings, agreement, check, citations, clone_facts, review_row, tag, twice_reasons,
};
use crate::eval_deadfile_positions::{RULES, docs};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The synthetic tree: `Makefile` names `tool.py` on line 3.
pub fn tip(corpus: &str, _commit: &str, path: &str) -> Option<String> {
    let makefile = "all:\n\techo\n\tpython tool.py\n\techo\n\n\n\n\tother\n";
    (corpus, path)
        .eq(&("p1", "Makefile"))
        .then(|| makefile.to_string())
}

fn row(path: &str) -> Value {
    json!({"corpus": "p1", "commit": "c0", "path": path})
}

/// An answer from `word ~ evidence ~ mechanism ~ searched`.
pub fn answer(spec: &str) -> Value {
    let f: Vec<&str> = spec.split(" ~ ").map(str::trim).collect();
    json!({"answer": f[0], "evidence": f[1], "mechanism": f[2], "searched": f[3]})
}

/// `answer spec => expected check`, one per line.
const CHECKS: &str = "
in_use ~ Makefile:3 ~ make ~  => ok
in_use ~ Makefile:99 ~ a convention ~  => convention
in_use ~ Nowhere:1 ~  ~  => fail: in_use names no mechanism and cites no existing path:line
unused ~  ~  ~ grep => fail: unused without a search record
unused ~  ~  ~ git grep -w tool over the tree, CI, Makefile => ok
maybe ~  ~  ~  => fail: not one of the three answer words
unsure ~  ~  ~  => ok";

/// `answer spec => expected tag`.
const TAGS: &str = "
in_use ~ Makefile:5 ~ make ~  => named
in_use ~ Makefile:8 ~ make ~  => generic
in_use ~  ~ a convention ~  => generic
unused ~  ~  ~ x => -";

fn spec_lines(table: &str) -> Vec<(Value, String)> {
    table
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            l.split_once(" => ")
                .map(|(a, w)| (answer(a), w.to_string()))
                .expect("=>")
        })
        .collect()
}

/// The lines of a `given | given => want` table.
pub fn cases(table: &'static str) -> Vec<(Vec<&'static str>, &'static str)> {
    let line = |l: &'static str| {
        l.split_once(" => ")
            .map(|(g, w)| (g.split(" | ").collect(), w))
    };
    table
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(line)
        .collect()
}

#[test]
fn citations_follow_the_main_sessions_pattern() {
    let seen: Vec<String> = citations("see Makefile:3, (src/a.py:12) and a:1:2")
        .iter()
        .map(|(p, l)| format!("{p}:{l}"))
        .collect();
    assert_eq!(seen, ["Makefile:3", "(src/a.py:12", "a:1", "1:2"]);
}

#[test]
fn the_evidence_check_and_the_tag_read_bars_and_addendum_one() {
    let r = row("tool.py");
    for (a, want) in spec_lines(CHECKS) {
        assert_eq!(check(&tip, &r, &a), want, "{a}");
    }
    for (a, want) in spec_lines(TAGS) {
        assert_eq!(tag(&tip, &r, &a).unwrap_or("-"), want, "{a}");
    }
}

/// `first | second | settled => answer | status`, answers by name.
const SETTLE: &str = "
used | - | - => in_use | ok
used | unused | - => - | unsettled: two different answers
used | unused | unsure => unsure | settled by evidence
unsure | unused | - => - | unsettled: two different answers
unsure | unused | unsure => unsure | settled by evidence
broken | used | - => in_use | replaced: first answer failed the check
broken | - | - => - | blocked: answer failed the check, no passing second reading";

#[test]
fn readings_settle_by_check_and_settlement_never_by_hand() {
    let named: BTreeMap<&str, Value> = [
        ("used", answer("in_use ~ Makefile:3 ~ make ~ ")),
        (
            "unused",
            answer("unused ~  ~  ~ git grep -w tool over the tree, CI, Makefile"),
        ),
        ("broken", answer("unused ~  ~  ~ x")),
        ("unsure", answer("unsure ~  ~  ~ ")),
    ]
    .into_iter()
    .collect();
    for (g, want) in cases(SETTLE) {
        let readings = Readings {
            first: named.get(g[0]),
            second: named.get(g[1]),
            settled: (g[2] != "-").then_some(g[2]),
            candidate: false,
        };
        let got = review_row(&clone_facts(&tip), &row("tool.py"), readings);
        let read = format!(
            "{} | {}",
            got["answer"].as_str().unwrap_or("-"),
            got["status"].as_str().unwrap_or("")
        );
        assert_eq!(read, want, "{g:?}");
    }
}

#[test]
fn second_readings_and_agreement() {
    let (unsure, unused) = (
        answer("unsure ~  ~  ~ "),
        answer("unused ~  ~  ~ searched everything that could name it"),
    );
    assert!(twice_reasons(&row("a.py"), &unsure, false).contains(&"unsure"));
    assert!(twice_reasons(&row("a.py"), &unused, true).contains(&"unused_convention"));
    assert!(!twice_reasons(&row("a.py"), &unused, false).contains(&"unused_convention"));
    let pairs = |differ: usize| -> Vec<Value> {
        (0..10)
            .map(|i| json!({"first": {"answer": "in_use", "tag": "named"}, "second": {"answer": if i < differ { "unused" } else { "in_use" }}}))
            .collect()
    };
    let one = agreement(&pairs(1));
    assert_eq!(
        (
            one["overall"]["share"].as_f64(),
            one["every_row_twice"].as_bool()
        ),
        (Some(0.9), Some(false))
    );
    assert_eq!(agreement(&pairs(2))["every_row_twice"], json!(true));
}

#[test]
fn a_convention_passes_only_by_all_three_rules() {
    let review = |path: &str, word: &str, mech: &str| json!({"corpus": "p", "path": path, "answer": word, "first": {"mechanism": mech, "evidence": ""}});
    let rows = [
        review("a/package-info.java", "in_use", "javac reads it"),
        review("b/package-info.java", "in_use", "package docs"),
    ];
    let passes = |rows: &[Value], docs: &Docs| {
        evaluate(rows, docs)
            .into_iter()
            .find(|v| v["name"] == "package-info.java")
            .map(|v| v["passes"].clone())
    };
    assert_eq!(passes(&rows, &Docs::new()), Some(json!(false)), "no input");
    assert_eq!(
        passes(&rows, &docs(RULES, false)),
        Some(json!(false)),
        "uncited"
    );
    let mut placeless = docs(RULES, true);
    placeless.get_mut("package-info.java").expect("row")["position"] = Value::Null;
    assert_eq!(
        passes(&rows, &placeless),
        Some(json!(false)),
        "no position rule"
    );
    let docs = docs(RULES, true);
    assert_eq!(passes(&rows, &docs), Some(json!(true)));
    let refuted = [rows[0].clone(), review("c/package-info.java", "unsure", "")];
    assert_eq!(
        passes(&refuted, &docs),
        Some(json!(false)),
        "an unsure row refutes"
    );
    let untold = [review("a/package-info.java", "in_use", "read by the build")];
    assert_eq!(
        passes(&untold, &docs),
        Some(json!(false)),
        "no answer names the tool"
    );
}
