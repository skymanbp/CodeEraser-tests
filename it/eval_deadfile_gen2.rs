//! The dead-file exam, generation 2 (lane v235_licence, part 4B):
//! every definition of BARS-GEN2 section 2 fixed on a synthetic history
//! before the replay (ignored, and refused until the registry names the
//! product commit) prints any count: a rename is the same file, a root
//! dead at the parent is standing, the first fate decides, `focused` at
//! 20 files and `sweep` at 21, the window of a short history, and the
//! name-status lines the replay reads. The readings, the bars, the
//! record and the registry are fixed in eval_deadfile_gen2_bars.rs; the
//! real-corpus legs live in eval_deadfile_gen2_parts/run.rs.

use crate::eval_deadfile_gen2_parts::model::{
    Dead, Event, Fate, History, Kind, State, Step, Tree, events_at, fate, window,
};
use crate::eval_deadfile_gen2_parts::replay::read_status;

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
