//! The flow ledger's EXECUTOR (plan v2.31 step 4 commit D; design
//! booklet analysis-track.md §5.5 「回放台账」): the ten rows of
//! contracts/eval/fpr-flow-v1.json, one per flow exam in the exam
//! table's order, each on the exam's first corpus at its pinned tip
//! over the shared window, each reading closing its own arithmetic
//! (`findings = tp + false + removed + undetermined`, strict and narrow
//! alike, per kind and over the rated kinds), and docs/FPR-REPLAY.md
//! quoting the three tables the rows derive. No rate floor here: a
//! flow language is admitted by its precision doc
//! (eval_flow_precision.rs, the mask leg; booklet §13 item 6); this leg
//! only holds the ledger whole — every judged language has its row.
//! The three legs that read the doc are staged by `FLOW_LEDGER`
//! (commit D1 Pending, D2 Frozen); the synthetic e2e is not.
//! Plus one e2e over a synthetic three-commit, two-language history,
//! so the counting rule itself is tested and not only its record.

use crate::common::ledger::{WINDOW, count, two_commits};
use crate::common::{chain_of, commit_all, core_bin, git_out, repo_root, write_all};
use crate::eval_flow_parts::EXAMS;
use crate::eval_support::doc_refused;
use crate::fpr_flow_replay_parts::render::{advisory_table, kind_table, table};
use crate::fpr_flow_replay_parts::{
    COUNTS, LEDGER, OUTCOME, Outcome, RATED, READINGS, corpus_of, rated, walk::replay,
};
use codeeraser::corelink::Link;
use codeeraser::scan::lang::Lang;
use serde_json::{Value, json};

/// One row against its exam: the language, corpus and tip the exam
/// table names, the shared window, the scalar counts present, each
/// reading's closure, and the harness commit on this history.
fn coherent(c: &Value, lang: &str) {
    let (corpus, tip) = corpus_of(crate::eval_flow_parts::exam(lang));
    let at = |k: &str| format!("{lang}: {k}");
    let pinned = [&c["lang"], &c["corpus"], &c["tip"]];
    assert_eq!(
        pinned,
        [&json!(lang), &json!(corpus), &json!(tip)],
        "{}",
        at("the exam's corpus")
    );
    assert_eq!(count(c, "window"), WINDOW as u64, "{}", at("window"));
    let n: Vec<u64> = COUNTS.split_whitespace().map(|k| count(c, k)).collect();
    assert!(n[0] <= WINDOW as u64 && n[1] > 0, "{}", at("an empty walk"));
    let kinds = c["kinds"].as_array().cloned().unwrap_or_default();
    let ks: Vec<Option<u64>> = kinds.iter().map(|k| k["kind"].as_u64()).collect();
    assert_eq!(
        ks,
        [Some(0), Some(1), Some(2), Some(3)],
        "{}",
        at("kinds 0..3")
    );
    for k in &kinds {
        closes(k, &at(&format!("kind {}", k["kind"])));
    }
    closes(c, &at("the rated kinds"));
    for r in READINGS {
        for key in OUTCOME.split_whitespace() {
            let sum: u64 = kinds[..RATED].iter().map(|k| count(&k[r], key)).sum();
            assert_eq!(
                count(&c[r], key),
                sum,
                "{}",
                at(&format!("{r} {key} = kinds 0..2"))
            );
        }
    }
    let commit = c["harness"]["commit"].as_str().unwrap_or_default();
    let (ok, _) = git_out(
        &repo_root(),
        &["merge-base", "--is-ancestor", commit, "HEAD"],
    );
    assert!(ok, "{}", at("harness.commit is not on this history"));
    let date = c["measured_at"].as_str().unwrap_or_default();
    assert!(
        date.len() == 10 && date.as_bytes()[4] == b'-',
        "{}",
        at("measured_at")
    );
}

/// Both readings of an outcome pair close — `findings = tp + false +
/// removed + undetermined` each — and the narrow falses are no more
/// than the strict ones.
fn closes(o: &Value, at: &str) {
    for r in READINGS {
        let n = |k: &str| {
            o[r][k]
                .as_u64()
                .unwrap_or_else(|| panic!("{at}: no {r} {k}"))
        };
        let ended = n("resolved_tp") + n("resolved_false") + n("removed") + n("undetermined");
        assert_eq!(
            n("findings"),
            ended,
            "{at}: {r} findings = tp + false + removed + undetermined"
        );
    }
    let f = |r: &str| o[r]["resolved_false"].as_u64();
    assert!(
        f("narrow") <= f("strict"),
        "{at}: narrow false <= strict false"
    );
}

/// Where the ledger stands, read the way the exam table reads its
/// stage words: `Pending` = the instrument and this gate are in and the
/// rows are measured on the next commit's clean tree (commit D1 / D2,
/// the precision docs' two-commit precedent: a row's `harness` must name
/// a commit whose tree held the instrument, `dirty = false`), so the
/// frozen doc must not be on disk yet; `Frozen` = the rows are filed and
/// every leg reads them.
#[derive(Clone, Copy, PartialEq)]
enum Stage {
    Pending,
    Frozen,
}

const FLOW_LEDGER: Stage = Stage::Pending;

/// Whether the ledger legs read rows: `Frozen` reads them; `Pending`
/// asserts the frozen doc is absent, by its path, and reads nothing.
fn filed() -> bool {
    let on_disk = repo_root().join(LEDGER.rel).exists();
    if FLOW_LEDGER == Stage::Frozen {
        return true;
    }
    assert!(
        !on_disk,
        "{}: on disk while FLOW_LEDGER is Pending; flip it to Frozen with the doc",
        LEDGER.rel
    );
    false
}

/// Every row coherent, in the exam table's order.
fn check(rows: &[Value]) {
    for (c, exam) in rows.iter().zip(EXAMS.iter()) {
        coherent(c, exam.lang);
    }
}

fn langs() -> Vec<&'static str> {
    EXAMS.iter().map(|e| e.lang).collect()
}

/// The ten rows in the exam table's order, each coherent; and
/// docs/FPR-REPLAY.md quoting the three tables as the rows derive them.
#[test]
fn every_flow_language_is_ledgered_on_its_own_history() {
    if !filed() {
        return;
    }
    let rows = LEDGER.rows(&langs());
    check(&rows);
    for printed in [table(&rows), kind_table(&rows), advisory_table(&rows)] {
        LEDGER.page_quotes("docs/FPR-REPLAY.md", &printed);
    }
}

/// The ledger is whole for the judged set: every language in
/// flow::judged_mask() has its own row (admission itself is the
/// precision docs').
#[test]
fn every_judged_flow_language_has_its_row() {
    if !filed() {
        return;
    }
    let rows = LEDGER.corpora();
    let mask = codeeraser::flow::judged_mask();
    for lang in Lang::with_grammar().filter(|l| mask & (1 << *l as i64) != 0) {
        let name = lang.name();
        let found = rows.iter().any(|c| c["lang"] == name);
        assert!(
            found,
            "{name}: judged by flow/1 but no row in {}",
            LEDGER.rel
        );
    }
}

/// A frozen count moved by hand reddens the gate, by the row's name.
#[test]
fn a_moved_frozen_count_is_refused_by_name() {
    if !filed() {
        return;
    }
    let rows = LEDGER.rows(&langs());
    assert!(!doc_refused(&json!(rows), &|d| check(
        d.as_array().unwrap()
    )));
    let mut forged = rows.clone();
    let tp = count(&forged[0]["strict"], "resolved_tp");
    forged[0]["strict"]["resolved_tp"] = json!(tp + 1);
    let said = std::panic::catch_unwind(|| check(&forged)).expect_err("a moved count must refuse");
    let said = said.downcast_ref::<String>().cloned().unwrap_or_default();
    let row = format!("{}: ", EXAMS[0].lang);
    assert!(
        said.contains(&row),
        "refused without naming the row: {said}"
    );
}

/// The Python file the synthetic history edits. Second commit: `f`
/// loses its dead store (true in both readings), `g` edits the line of
/// its unused local and keeps it (false in both), `h` edits another
/// line and keeps its unused local (strict false; narrow leaves it
/// open), `k` is deleted (removed), `r` is only re-spaced (no edit).
/// Third commit: `h` is edited away from its local again (settled
/// once in the strict reading, still open in the narrow one).
const SEED_PY: &str = "def f(a):\n    x = 1\n    x = a\n    return x\n\n\ndef g(a):\n    z = a + 1\n    return a\n\n\ndef h(a):\n    w = a + 1\n    return a\n\n\ndef k(a):\n    q = a + 1\n    return a\n\n\ndef r(a):\n    v = a + 1\n    return a\n";
const THEN_PY: &str = "def f(a):\n    x = a\n    return x\n\n\ndef g(a):\n    z = a + 2\n    return a\n\n\ndef h(a):\n    w = a + 1\n    return a * 2\n\n\ndef r(a):\n    v=a+1\n    return  a\n";
const LAST_PY: &str = "def f(a):\n    x = a\n    return x\n\n\ndef g(a):\n    z = a + 2\n    return a\n\n\ndef h(a):\n    w = a + 1\n    return a * 3\n\n\ndef r(a):\n    v=a+1\n    return  a\n";
/// A Lua file with a dead store both commits keep, edited elsewhere.
const SEED_LUA: &str = "local function m(a)\n  local y = 1\n  y = a\n  return y\nend\n";
const THEN_LUA: &str = "local function m(a)\n  local y = 1\n  y = a\n  return y + 1\nend\n";

/// A synthetic three-commit history in two languages, read for Python
/// and for Lua: each walk counts only its own language's events, and
/// each outcome lands where the rule puts it in each reading.
#[test]
fn a_synthetic_history_lands_each_outcome() {
    let seed = [("a.py", SEED_PY), ("b.lua", SEED_LUA)];
    let repo = two_commits(
        "fpr-flow-e2e",
        &seed,
        &[("a.py", THEN_PY), ("b.lua", THEN_LUA)],
    );
    write_all(&repo, &[("a.py", LAST_PY)]);
    commit_all(&repo, "third");
    let commits = chain_of(&repo, "HEAD", None);
    let (mut link, _) = Link::open(&core_bin()).expect("open core");
    let py = replay(&repo, Lang::Python, &commits, &mut link);
    // found / true / false / removed / undetermined, strict then narrow
    let want: [Outcome; 2] = [[5, 1, 2, 1, 1], [5, 1, 1, 1, 2]];
    let walked = (py.events, py.units_judged, py.dynamic_skipped, py.unjudged);
    assert_eq!(
        (walked, rated(&py)),
        ((2, 17, 0, 0), want),
        "the Python walk"
    );
    let found: Vec<usize> = py.kinds.iter().map(|k| k[0][0]).collect();
    assert_eq!(
        found,
        [0, 1, 4, 0],
        "the dead store and the four unused locals"
    );
    let lua = rated(&replay(&repo, Lang::Lua, &commits, &mut link));
    assert_eq!(
        (lua[0][2], lua[1][2]),
        (1, 0),
        "the Lua walk: strict false 1, narrow 0"
    );
}
