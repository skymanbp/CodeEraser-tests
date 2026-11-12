//! The dead-file exam, generation 2 (lane v235_licence, part 4B): the
//! record is the main session's bytes, the registry holds BARS-GEN2
//! section 1's ten corpora, and every definition of section 2 is fixed
//! on a synthetic history — a rename is the same file, a root dead at
//! the parent is standing, the first fate decides, `focused` at 20 files
//! and `sweep` at 21, the window of a short history — before the replay
//! (ignored, and refused until the registry names the product commit)
//! prints any count.
//!   CE_DEADFILE_GEN2_ROOT=<clones> CE_DEADFILE_GEN2_COPIES=<scratch dir> cargo test --test it -- --ignored eval_deadfile_gen2::deadfile_gen2_tips --nocapture

use crate::common::repo_root;
use crate::eval_deadfile_gen2_parts::audit::{Row, drawn, names_component, reading_c};
use crate::eval_deadfile_gen2_parts::model::{
    Dead, Event, Fate, History, Kind, State, Step, Tree, events_at, fate, window,
};
use crate::eval_deadfile_gen2_parts::readings::{
    Outcome, Reading, Table, blocks, outcome, sentence, share, tally,
};
use crate::eval_deadfile_gen2_parts::replay::read_status;
use crate::eval_deadfile_gen2_parts::{
    MIN_FIRST_PARENT, REGISTRY_SCHEMA, corpora, pinned, product_commit, registry,
};
use crate::eval_support::content_sha;

/// A history from one line per commit, `;`-separated clauses: `f a b`
/// the tree's files, `r x` its dead roots, `d y` its dependents, `m x`
/// content changed by this commit, `n new<old` a rename (`!` after it:
/// with an edit), `x z` deleted, `t N` files touched, `u` unjudged.
fn history(spec: &str) -> History {
    let mut h = History::default();
    for line in spec.lines().filter(|l| !l.trim().is_empty()) {
        let (mut tree, mut step) = (
            Tree {
                judged: true,
                ..Tree::default()
            },
            Step::default(),
        );
        for clause in line.split(';').map(str::trim).filter(|c| !c.is_empty()) {
            let (head, rest) = clause.split_once(' ').unwrap_or((clause, ""));
            let words = rest.split_whitespace().map(String::from);
            match head {
                "f" => tree.files.extend(words),
                "r" | "d" => {
                    let root = head == "r";
                    for w in words {
                        let document = w.ends_with(".md");
                        tree.dead.insert(w, Dead { root, document });
                    }
                }
                "m" => step.changed.extend(words),
                "x" => step.deleted.extend(words),
                "t" => step.touched = rest.parse().expect("touched"),
                "u" => tree.judged = false,
                "n" => {
                    for w in words {
                        let edited = w.ends_with('!');
                        let (to, from) = w.trim_end_matches('!').split_once('<').expect("new<old");
                        step.renamed.insert(to.into(), from.into());
                        if edited {
                            step.changed.insert(to.into());
                        }
                    }
                }
                _ => panic!("unread clause {clause:?}"),
            }
        }
        h.shas.push(format!("c{}", h.trees.len()));
        h.trees.push(tree);
        h.steps.push(step);
    }
    h
}

/// The one event of commit `c`, and its fate.
fn only(h: &History, c: usize) -> (Event, Fate) {
    let evs = events_at(h, c);
    assert_eq!(evs.len(), 1, "{evs:?}");
    let f = fate(h, &evs[0]);
    (evs[0].clone(), f)
}

#[test]
fn a_rename_is_the_same_file() {
    // orphaned at 1, renamed (no edit) at 2 while dead, referred again at 3
    let h = history("f a.py\nf a.py ; r a.py\nf b.py ; r b.py ; n b.py<a.py ; t 1\nf b.py ; t 1");
    let (e, f) = only(&h, 1);
    assert_eq!(
        (e.state, e.kind, f),
        (State::Orphaned, Kind::Code, Fate::ReReferred)
    );
    // a dead file renamed is no birth: dead at the parent under its old name
    let h = history("f x.md ; r x.md\nf y.md ; r y.md ; n y.md<x.md ; t 1");
    assert!(events_at(&h, 1).is_empty(), "standing across a rename");
    // a rename with an edit while dead is a change: kept up
    let h = history("f a.py\nf a.py ; r a.py\nf b.py ; r b.py ; n b.py<a.py! ; t 1");
    assert_eq!(only(&h, 1).1, Fate::KeptUp { focused: true });
}

#[test]
fn a_root_dead_at_the_parent_is_standing_not_an_event() {
    let h = history("f a.py s.py ; d a.py ; r s.py\nf a.py s.py ; r a.py s.py ; t 1");
    assert!(
        events_at(&h, 1).is_empty(),
        "dependent then root; root then root"
    );
    // a dependent is never an event; a new file nothing refers to is born
    let h = history("f a.py\nf a.py b.py c.md ; d b.py ; r c.md ; m b.py c.md ; t 2");
    let (e, f) = only(&h, 1);
    assert_eq!(
        (e.path.as_str(), e.state, e.kind, f),
        ("c.md", State::Born, Kind::Document, Fate::Undetermined)
    );
    // an unjudged tree on either side of the pair yields nothing
    let h = history("f a.py ; u\nf a.py ; r a.py ; t 1");
    assert!(events_at(&h, 1).is_empty());
}

#[test]
fn the_first_fate_that_happens_decides() {
    let base = "f a.py\nf a.py ; r a.py\n";
    let cases = [
        (
            "f a.py ; r a.py ; m a.py ; t 3\nf ; x a.py ; t 1",
            Fate::KeptUp { focused: true },
        ),
        (
            "f ; x a.py ; t 1\nf a.py ; m a.py ; t 1",
            Fate::RemovedUntouched,
        ),
        ("f a.py ; m a.py ; t 1\nf ; x a.py ; t 1", Fate::ReReferred),
        (
            "f a.py ; r a.py ; t 2\nf a.py ; r a.py ; t 1",
            Fate::Undetermined,
        ),
        ("f a.py ; u ; t 1\nf a.py ; t 1", Fate::ReReferred),
    ];
    for (rest, want) in cases {
        let h = history(&format!("{base}{rest}"));
        assert_eq!(only(&h, 1).1, want, "{rest}");
    }
}

#[test]
fn focused_at_twenty_files_sweep_at_twenty_one() {
    for (touched, focused) in [(20, true), (21, false)] {
        let h = history(&format!(
            "f a.py\nf a.py ; r a.py\nf a.py ; r a.py ; m a.py ; t {touched}"
        ));
        assert_eq!(only(&h, 1).1, Fate::KeptUp { focused });
    }
}

#[test]
fn the_window_and_a_window_shorter_than_eight_hundred_commits() {
    // tip = len - 1; W = tip~799 .. tip~200
    assert_eq!(window(1000), 200..800);
    // exactly 800: the root commit has no parent, so no pair
    assert_eq!(window(800), 1..600);
    // Penlight's 713 first-parent commits: every commit with a parent up to tip~200
    assert_eq!(window(713), 1..513);
    assert_eq!(window(150), 1..1, "no commit 200 before the tip: no window");
}

#[test]
fn the_name_status_lines_read_as_section_two_says() {
    let mut s = Step::default();
    for line in [
        "R100\told.py\tnew.py",
        "R087\tx.md\ty.md",
        "A\tb.py",
        "M\tc.py",
        "D\td.py",
        "T\te",
        "C075\tf.py\tg.py",
    ] {
        read_status(&mut s, line);
    }
    assert_eq!(s.touched, 7);
    assert_eq!(s.renamed.get("new.py").map(String::as_str), Some("old.py"));
    let changed: Vec<&str> = s.changed.iter().map(String::as_str).collect();
    assert_eq!(
        changed,
        ["b.py", "c.py", "e", "g.py", "y.md"],
        "a rename alone is no change"
    );
    assert!(s.deleted.contains("d.py"));
}

/// `(fates, (false, denominator) under A, under B)`.
#[test]
fn the_two_readings() {
    let f = [
        Fate::ReReferred,
        Fate::KeptUp { focused: true },
        Fate::KeptUp { focused: false },
        Fate::RemovedUntouched,
        Fate::Undetermined,
    ];
    assert_eq!(share(Reading::A, &f), (4, 5));
    assert_eq!(share(Reading::B, &f), (2, 3));
    // reading C: the one undetermined added at an audited share of 1 / 2
    assert_eq!(reading_c(&f, 1, 2), (2, 4));
    assert_eq!(reading_c(&f, 2, 2), (3, 4));
}

/// A table of `corpus state kind removed false` rows, `;` between rows:
/// `false` re-referred fates and `removed` removed-untouched ones.
fn table(spec: &str) -> Table {
    use {Kind::*, State::*};
    let mut t = Table::new();
    for row in spec.split(';') {
        let w: Vec<&str> = row.split_whitespace().collect();
        let [corpus, state, kind, removed, bad] = w[..] else {
            panic!("table row {row:?}")
        };
        let state = if state == "orphaned" { Orphaned } else { Born };
        let kind = if kind == "document" { Document } else { Code };
        let n = |s: &str| s.parse::<usize>().expect("count");
        let ev = |path: String| Event {
            at: 1,
            path,
            state,
            kind,
        };
        let evs: Vec<(Event, Fate)> = (0..n(removed))
            .map(|i| (ev(format!("r{i}")), Fate::RemovedUntouched))
            .chain((0..n(bad)).map(|i| (ev(format!("b{i}")), Fate::ReReferred)))
            .collect();
        tally(corpus, &evs, &mut t);
    }
    t
}

/// `rows => state outcome`, one case a line; `#` lines say why.
const BARS: &str = "
# 2 false in 100 misses the 1.0 % bar
p orphaned code 98 2 => orphaned Listed
# 99 events: not judged
p born code 99 0 => born NotJudged
# pooled 2 / 300 passes, but one corpus of 30 events at 2 false (6.7 %) is above 5.0 %
p orphaned code 270 0; q orphaned code 28 2 => orphaned Listed
# the same corpus with 29 events is below the corpus floor: blocks
p orphaned code 271 0; q orphaned code 27 2 => orphaned FailsByDefault
# a kind with 100 events of its own answers the bars by itself
p orphaned code 400 0; q orphaned document 98 2 => orphaned Listed";

#[test]
fn the_bars_and_their_four_outcomes() {
    use {Kind::*, State::*};
    // 1 false in 100: at most 1.0 %, blocks under both readings
    let t = table("p orphaned code 99 1");
    assert_eq!(
        (blocks(&t, Orphaned, Reading::A), outcome(&t, Orphaned)),
        (Some(true), Outcome::FailsByDefault)
    );
    let cases = BARS.lines().filter_map(|l| l.split_once(" => "));
    let got: Vec<String> = cases
        .clone()
        .map(|(rows, want)| {
            let state = if want.starts_with("orphaned") {
                Orphaned
            } else {
                Born
            };
            format!("{rows} => {}", want.split(' ').next().unwrap_or(""))
                + &format!(" {:?}", outcome(&table(rows), state))
        })
        .collect();
    let want: Vec<String> = cases
        .map(|(rows, want)| format!("{rows} => {want}"))
        .collect();
    assert_eq!(got, want);
    // A and B disagree: an undetermined counts false under A only
    let mut t = table("p born code 100 0");
    let ev = Event {
        at: 1,
        path: "u".into(),
        state: Born,
        kind: Code,
    };
    tally(
        "p",
        &[
            (ev.clone(), Fate::Undetermined),
            (ev.clone(), Fate::Undetermined),
        ],
        &mut t,
    );
    assert_eq!(outcome(&t, Born), Outcome::OwnerQuestion);
    for o in [
        Outcome::FailsByDefault,
        Outcome::Listed,
        Outcome::OwnerQuestion,
        Outcome::NotJudged,
    ] {
        assert!(sentence("orphaned", o).starts_with("orphaned "), "{o:?}");
    }
}

/// The report reads a replay doc back and prints every fate, both
/// shares and the leave-one-out line per state.
#[test]
fn the_report_reads_a_replay_doc() {
    use crate::eval_deadfile_gen2_parts::report::{render, table_of};
    let doc = serde_json::json!({"corpora": [
        {"corpus": "p", "events": [
            {"path": "a.py", "state": "orphaned", "kind": "code", "fate": "removed untouched"},
            {"path": "b.md", "state": "born", "kind": "document", "fate": "kept up sweep"}]},
        {"corpus": "q", "events": [
            {"path": "c.py", "state": "orphaned", "kind": "code", "fate": "re-referred"}]}]});
    let text = render(&table_of(&doc));
    let lines = "| p | 1 | 0 | 0 | 0 | 1 | 0 | 0/1 (0.0 %
| pooled | 2 | 1 | 0 | 0 | 1 | 0 | 1/2 (50.0 %
leave one out, reading A: without p 1/1 (100.0 %
orphaned has fewer than 100 events";
    let missing: Vec<&str> = lines.lines().filter(|l| !text.contains(l)).collect();
    assert!(missing.is_empty(), "{missing:?} not in\n{text}");
}

#[test]
fn the_audit_draws_by_key_and_reads_whole_components() {
    let rows: Vec<Row> = (0..70)
        .map(|i| Row {
            corpus: "p".into(),
            commit: format!("c{i}"),
            path: "a.py".into(),
            state: State::Born,
        })
        .collect();
    let picked = drawn(&rows, State::Born);
    assert_eq!(picked.len(), 60);
    assert_eq!(drawn(&rows, State::Orphaned), Vec::<Row>::new());
    let first = rows
        .iter()
        .min_by_key(|r| crate::eval_deadfile_gen2_parts::audit::key(r))
        .expect("row");
    assert_eq!(&picked[0], first);
    assert!(names_component(
        "sphinx reads docs/conf.py at build",
        "docs/conf.py"
    ));
    assert!(
        !names_component("a myconf.py loader", "docs/conf.py"),
        "N8: no substring match"
    );
}

#[test]
fn the_record_is_the_main_sessions_bytes() {
    for (rel, sha) in pinned() {
        let text =
            std::fs::read_to_string(repo_root().join("contracts/eval").join(rel)).expect(rel);
        assert_eq!(content_sha(&text), sha, "{rel}: not the copied bytes");
    }
}

#[test]
fn the_registry_holds_section_one_before_any_count() {
    let reg = registry();
    assert_eq!(reg["schema"], REGISTRY_SCHEMA);
    let rows = reg["corpora"].as_array().expect("corpora");
    assert_eq!(rows.len(), corpora().len());
    for (row, (lang, first, replacement)) in rows.iter().zip(corpora()) {
        assert_eq!(
            (
                row["language"].as_str(),
                row["first"].as_str(),
                row["replacement"].as_str()
            ),
            (Some(lang), Some(first), Some(replacement))
        );
        let chosen = row["chosen"].as_str().expect("chosen");
        assert!(
            chosen == first || (chosen == replacement && row["replaced_because"].is_string()),
            "{lang}"
        );
        let tip = row["tip"].as_str().expect("tip");
        assert!(oid(tip), "{lang}: {tip}");
        let n = row["first_parent_commits"].as_u64().expect("count");
        assert!(
            chosen != first || n >= MIN_FIRST_PARENT,
            "{lang}: {n} < {MIN_FIRST_PARENT}"
        );
    }
    if let Some(c) = product_commit(&reg) {
        assert!(oid(c), "{c}");
    }
}

/// A full commit id: forty hex digits.
fn oid(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
