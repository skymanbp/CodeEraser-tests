//! The dead-file exam, generation 2 (lane v235_licence, part 4B): the
//! run's own rules, fixed before any count — the report prints what the
//! bars read (G2) and the side numbers (G3), the doc's words are read
//! strictly (G4), one predicate says "the product judged this tree" (G5),
//! "armed" means the registered product is the one under test (G6), and
//! a run's docs are one per corpus, each chosen row exactly once (G14).

use crate::common::{core_bin, repo_root};
use crate::eval_deadfile_gen2_parts::doc::{checked, done, write};
use crate::eval_deadfile_gen2_parts::judge::{judgment, product_record};
use crate::eval_deadfile_gen2_parts::report::{render, render_sides, table_of};
use serde_json::{Value, json};

/// A run doc from `corpus state kind fate count` rows, `;` between rows.
fn run_doc(spec: &str) -> Value {
    let mut corpora: Vec<Value> = Vec::new();
    for row in spec.split(';') {
        let w: Vec<&str> = row.split_whitespace().collect();
        let [corpus, state, kind, ref fate @ .., count] = w[..] else {
            panic!("row {row:?}")
        };
        let fate = fate.join(" ");
        let n: usize = count.parse().expect("count");
        let events: Vec<Value> = (0..n)
            .map(|i| json!({"path": format!("{fate}{i}"), "state": state, "kind": kind, "fate": fate}))
            .collect();
        match corpora.iter_mut().find(|c| c["corpus"] == corpus) {
            Some(c) => c["events"].as_array_mut().expect("events").extend(events),
            None => corpora.push(json!({"corpus": corpus, "events": events})),
        }
    }
    json!({ "corpora": corpora })
}

/// The report of a run doc holds every line of `lines`.
fn report_has(spec: &str, lines: &str) {
    let text = render(&table_of(&run_doc(spec)));
    let missing: Vec<&str> = lines.lines().filter(|l| !text.contains(l)).collect();
    assert!(missing.is_empty(), "{missing:?} not in\n{text}");
}

/// The report reads a replay doc back and prints every fate, both
/// shares and the leave-one-out line per state.
#[test]
fn the_report_reads_a_replay_doc() {
    report_has(
        "p orphaned code removed untouched 1; p born document kept up sweep 1; \
         q orphaned code re-referred 1",
        "| p | 1 | 0 | 0 | 0 | 1 | 0 | 0/1 (0.0 %
| pooled | 2 | 1 | 0 | 0 | 1 | 0 | 1/2 (50.0 %
leave one out, reading A: without p 1/1 (100.0 %
orphaned has fewer than 100 events",
    );
}

/// The bars of a state printed condition by condition (G2): a reader
/// checks the outcome from these integers alone.
#[test]
fn the_report_prints_what_the_bars_read() {
    report_has(
        "p orphaned code removed untouched 100; q orphaned document removed untouched 28; \
         q orphaned document re-referred 2",
        "| q document | 30 | 2 | 0 | 0 | 28 | 0 |
| pooled code | 100 | 0 | 0 | 0 | 100 | 0 |
leave one out document, reading B: without p 2/30
(1) 130 events, at least 100: holds
(2) pooled 2/130 at most 1.0 %: fails
(3) p (100 events) 0/100 at most 5.0 %: holds
(3) q (30 events) 2/30 at most 5.0 %: fails
(4) kind code (100 events, at least 100):
(2) pooled 0/100 at most 1.0 %: holds
(4) kind document (30 events, fewer than 100): not read
orphaned misses the bars under readings A and B
(1) 0 events, at least 100: fails
born has fewer than 100 events",
    );
}

/// G3: the side numbers per corpus and pooled, every unjudged tree with
/// its commit and reason.
#[test]
fn the_report_prints_the_side_numbers() {
    let side = |c: &str, n: u64| {
        json!({"corpus": c, "unjudged_pairs": n, "unjudged_after_window": 1, "met_unjudged": 2,
               "dependents": 3, "symlink_rows": 0,
               "unjudged_trees": [{"commit": "c7", "reason": "exit 2: ce deadcode: parse ce.toml"}]})
    };
    let text = render_sides(&[side("p", 4), side("q", 5)]);
    for want in [
        "| p | 4 | 1 | 2 | 3 | 0 |",
        "| pooled | 9 | 2 | 4 | 6 | 0 |",
        "unjudged \"q\" \"c7\": \"exit 2: ce deadcode: parse ce.toml\"",
    ] {
        assert!(text.contains(want), "{want:?} not in\n{text}");
    }
}

/// G4: a word no table holds is refused by name.
#[test]
#[should_panic(expected = "unknown state \"orphan\"")]
fn a_misspelt_state_is_refused() {
    table_of(&run_doc("p orphan code removed untouched 1"));
}

#[test]
#[should_panic(expected = "unknown kind \"docs\"")]
fn a_misspelt_kind_is_refused() {
    table_of(&run_doc("p born docs removed untouched 1"));
}

/// G5's cases, one a line: `exit | stdout | stderr => reading`, `-` an
/// empty stream, `x` no exit code. `judged`, `tree <reason>` (unjudged,
/// counted) or `run <reason>` (refused by name).
const JUDGMENTS: &str = r#"
0 | {"degraded":null,"dead":[]} | - => judged
1 | {"degraded":null,"dead":[]} | - => judged
0 | {"degraded":"graph_too_large"} | - => tree exit 0: degraded: "graph_too_large"
2 | - | ce deadcode: parse ce.toml: TOML parse error => tree exit 2: ce deadcode: parse ce.toml: TOML parse error
0 | not a document | warn => tree exit 0: stdout does not parse: warn
x | - | - => tree no exit code
2 | - | ce deadcode: core unavailable: no core => run exit 2: ce deadcode: core unavailable: no core ("core unavailable")
2 | - | core reply deadline exceeded: no progress — core killed => run exit 2: core reply deadline exceeded: no progress — core killed ("core killed")
2 | - | proto mismatch: core 9.0.0 vs ce 10.0.0 => run exit 2: proto mismatch: core 9.0.0 vs ce 10.0.0 ("proto mismatch")"#;

/// One case of JUDGMENTS read by G5's predicate.
fn read(case: &str) -> String {
    let cols: Vec<&str> = case.split(" | ").collect();
    let [code, out, err] = cols[..] else {
        panic!("case {case:?}")
    };
    let bytes = |s: &str| {
        if s == "-" {
            Vec::new()
        } else {
            s.as_bytes().to_vec()
        }
    };
    match judgment(code.parse().ok(), &bytes(out), &bytes(err)) {
        Ok(Ok(_)) => "judged".to_string(),
        Ok(Err(why)) => format!("tree {why}"),
        Err(run) => format!("run {run}"),
    }
}

#[test]
fn one_predicate_says_the_product_judged_the_tree() {
    let (cases, want): (Vec<&str>, Vec<&str>) = JUDGMENTS
        .trim()
        .lines()
        .map(|l| l.rsplit_once(" => ").expect("case"))
        .unzip();
    let got: Vec<String> = cases.iter().map(|c| read(c)).collect();
    assert_eq!(got, want);
}

/// G6: the registered product is HEAD's, path by path; a commit whose
/// `cli/src` differs, or no commit at all, is refused by name.
#[test]
fn armed_means_the_registered_product_is_the_one_under_test() {
    let root = repo_root();
    let head = crate::common::gitio::git_ok(&root, &["rev-parse", "HEAD"]);
    let rec = product_record(&root, head.trim()).expect("HEAD's product is HEAD's");
    assert_eq!(rec["objects"].as_object().map(|o| o.len()), Some(5));
    let core = crate::eval_deadfile_gen2_parts::judge::file_sha(&core_bin());
    assert_eq!(
        (rec["core_sha256"].as_str(), rec["head_tree"].is_string()),
        (Some(core.as_str()), true)
    );
    let touched =
        crate::common::gitio::git_ok(&root, &["log", "-1", "--format=%H", "--", "cli/src"]);
    let before = format!("{}^", touched.trim());
    let err = product_record(&root, &before).expect_err("cli/src moved since");
    assert!(err.starts_with("cli/src: "), "{err}");
    let none = product_record(&root, &"0".repeat(40)).expect_err("no commit");
    assert!(none.ends_with("no such commit here"), "{none}");
}

/// G14: one doc per corpus, resumable; a run's docs hold every chosen row
/// exactly once and one product record.
#[test]
fn a_run_is_one_doc_per_corpus() {
    let out = crate::common::tmp("deadfile-gen2-docs");
    let (p, q) = (json!({"commit": "a"}), json!({"commit": "b"}));
    let doc = |c: &str, prod: &Value| json!({"corpus": c, "tip": "t1", "product": prod});
    write(&out, &doc("flask", &p));
    assert!(done(&out, "flask", "t1", &p));
    assert!(!done(&out, "flask", "t2", &p), "another tip");
    assert!(!done(&out, "flask", "t1", &q), "another product");
    assert!(!done(&out, "fd", "t1", &p), "not yet written");
    let (f, g) = (doc("flask", &p), doc("fd", &p));
    assert_eq!(checked(&[f.clone(), g.clone()], &["flask", "fd"]), Ok(()));
    assert_eq!(
        checked(std::slice::from_ref(&f), &["flask", "fd"]),
        Err("fd: 0 docs, want exactly one".into())
    );
    assert_eq!(
        checked(&[f.clone(), f.clone(), g.clone()], &["flask", "fd"]),
        Err("flask: 2 docs, want exactly one".into())
    );
    assert!(checked(&[f.clone(), g], &["flask"]).is_err_and(|e| e.contains("no chosen row")));
    let other = doc("fd", &q);
    assert!(
        checked(&[f, other], &["flask", "fd"]).is_err_and(|e| e.contains("another product record"))
    );
}
