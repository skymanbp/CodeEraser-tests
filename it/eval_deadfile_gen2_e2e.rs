//! The dead-file exam, generation 2 (lane v235_licence, part 4B, G9):
//! `replay::history` end to end with the real `ce`, on a history seeded
//! in a temp directory. Trees, steps, `events_at` and `fate` are asserted
//! directly (the 600 / 200 window is not involved). The history holds a
//! dead root with a non-ASCII name, a pure rename and a rename with an
//! edit of a dead root, a referenced file whose reference goes and comes
//! back, a dead root deleted, a tree the product refuses (a malformed
//! `ce.toml`, repaired in the next commit) with a dead root deleted AT
//! the refused commit, a symbolic link, and a dead page referenced only
//! by a link to one of its sections from another dead page (part 4C: a
//! dependent, no event).

use crate::eval_deadfile_gen2_parts::model::{
    Event, Fate, History, Kind, State, events_at, follow,
};
use crate::eval_deadfile_gen2_parts::replay::history;
use crate::eval_deadfile_gen2_seed::seeded;

/// One expected event: `commit path state`, all of them documents.
const EVENTS: &str = "
1 gone.md born
1 old.md born
1 p1.md born
1 zap.md born
1 e1.md born
1 \u{fc}ber.md born
4 ref.md orphaned";

/// Each event's expected fate, by path: `path fate decided-at path-then`
/// (`-` when undetermined, the name at the tip then).
const FATES: &str = "
gone.md removed 5 gone.md
old.md undetermined - new.md
p1.md undetermined - p1.md
zap.md removed 6 zap.md
e1.md kept 3 e2.md
\u{fc}ber.md undetermined - \u{fc}ber.md
ref.md re-referred 8 ref.md";

fn fate_word(f: Fate) -> &'static str {
    match f {
        Fate::ReReferred => "re-referred",
        Fate::KeptUp { focused: true } => "kept",
        Fate::KeptUp { focused: false } => "kept-sweep",
        Fate::RemovedUntouched => "removed",
        Fate::Undetermined => "undetermined",
    }
}

/// Every event of the history, `commit path state`, sorted.
fn all_events(h: &History) -> Vec<Event> {
    (1..h.trees.len()).flat_map(|c| events_at(h, c)).collect()
}

fn expected_events() -> Vec<String> {
    let mut v: Vec<String> = EVENTS
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    v.sort();
    v
}

fn event_lines(evs: &[Event]) -> Vec<String> {
    let mut v: Vec<String> = evs
        .iter()
        .map(|e| {
            let s = if e.state == State::Born {
                "born"
            } else {
                "orphaned"
            };
            format!("{} {} {s}", e.at, e.path)
        })
        .collect();
    v.sort();
    v
}

/// The mismatches of the history against the seed's intent, one line each.
fn mismatches(h: &History) -> Vec<String> {
    let mut bad = Vec::new();
    let judged: Vec<bool> = h.trees.iter().map(|t| t.judged).collect();
    let want: Vec<bool> = (0..h.trees.len()).map(|i| i != 6).collect();
    if judged != want {
        bad.push(format!("judged {judged:?}, want {want:?}"));
    }
    let evs = all_events(h);
    if event_lines(&evs) != expected_events() {
        bad.push(format!(
            "events {:?}, want {:?}",
            event_lines(&evs),
            expected_events()
        ));
    }
    if evs.iter().any(|e| e.kind != Kind::Document) {
        bad.push("an event of kind code".into());
    }
    for line in FATES.lines().filter(|l| !l.is_empty()) {
        let (path, want) = line.split_once(' ').expect("fate row");
        match evs.iter().find(|e| e.path == path) {
            Some(e) => {
                let f = follow(h, e);
                let at = f.decided_at.map_or("-".into(), |k| k.to_string());
                let got = format!("{} {at} {}", fate_word(f.fate), f.path_then);
                if got != want {
                    bad.push(format!("{path}: {got}, want {want}"));
                }
            }
            None => bad.push(format!("{path}: no event")),
        }
    }
    bad.extend(tree_mismatches(h));
    bad
}

/// The dependent, the renames and the symbolic link, read off the trees
/// and steps.
fn tree_mismatches(h: &History) -> Vec<String> {
    let mut bad = Vec::new();
    match h.trees[1].dead.get("p2.md") {
        Some(d) if !d.root => {}
        other => bad.push(format!("p2.md at c1: {other:?}, want a dependent")),
    }
    let (s2, s3) = (&h.steps[2], &h.steps[3]);
    if s2.renamed.get("new.md").map(String::as_str) != Some("old.md")
        || s2.changed.contains("new.md")
    {
        bad.push(format!("step 2 (pure rename): {s2:?}"));
    }
    if s3.renamed.get("e2.md").map(String::as_str) != Some("e1.md") || !s3.changed.contains("e2.md")
    {
        bad.push(format!("step 3 (rename with an edit): {s3:?}"));
    }
    // G16: a link checked out as a small text file (no symlinks here) is
    // a dead row of the product, dropped and counted; a real link is not
    // walked at all
    let (t9, links) = (&h.trees[9], usize::from(cfg!(windows)));
    if t9.dead.contains_key("link.md") || t9.symlink_rows != links {
        let got = (t9.dead.get("link.md"), t9.symlink_rows);
        bad.push(format!("link.md at c9: dead row, dropped {got:?}"));
    }
    // G5: the refused tree says why
    let why = h.trees[6].why.as_deref().unwrap_or("");
    if !(why.starts_with("exit 2: ") && why.contains("ce.toml")) {
        bad.push(format!("c6 unjudged because {why:?}"));
    }
    bad
}

#[test]
fn the_replay_reads_a_seeded_history_with_the_real_product() {
    let (seeded, copy, tip) = seeded("deadfile-gen2-e2e");
    let h = history(&seeded, &copy, &tip);
    assert_eq!(h.trees.len(), 10, "ten commits");
    let bad = mismatches(&h);
    assert!(
        bad.is_empty(),
        "the replay disagrees with the seed:\n{}",
        bad.join("\n")
    );
}

/// G17: a tree git cannot check out here is unjudged with git's first
/// line, and the run goes on (a held index lock stands in for a name the
/// file system refuses: the same failed checkout, on every platform).
#[test]
fn a_tree_git_cannot_check_out_is_unjudged_by_git_s_words() {
    use crate::eval_deadfile_gen2_parts::gitz::{checkout, clone_copy};
    let (seeded, copy, tip) = seeded("deadfile-gen2-checkout");
    clone_copy(&seeded, &copy);
    std::fs::write(copy.join(".git").join("index.lock"), "").expect("lock");
    let why = checkout(&copy, &tip).expect_err("a held lock refuses the checkout");
    assert!(
        why.starts_with("checkout: ") && why.contains("index.lock"),
        "{why}"
    );
}
