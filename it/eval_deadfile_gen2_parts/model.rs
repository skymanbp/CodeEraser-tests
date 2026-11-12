//! BARS-GEN2 section 2 as code: the events and their fates over one
//! corpus's history, read off a model that holds only what the section
//! names — per commit the tree's files and the product's dead files
//! (root or dependent, code or document), per step the paths git's
//! rename detection pairs and the content it changed. The replay fills
//! the model from git and the product; the legs fill it by hand. Nothing
//! here counts or judges: a fate is a fact of the history.

use std::collections::{BTreeMap, BTreeSet};

/// One dead file in one tree: a ROOT when no file of the tree refers to
/// it (the product's `unref_*`), else a dependent (`unreach_*`); a
/// `document` when the product classes it as a documentation file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dead {
    pub root: bool,
    pub document: bool,
}

/// One commit's tree as the section reads it. `judged` is false when the
/// product refused the tree (G5) or git could not check it out here
/// (G17): a pair touching it yields no event, and `why` says what
/// happened (the exit code and the words). `retried` holds the first
/// run's reason of a tree only the second run judged (R2). `symlink_rows`
/// counts the dead rows dropped as symbolic links (G16).
#[derive(Clone, Debug, Default)]
pub struct Tree {
    pub files: BTreeSet<String>,
    pub dead: BTreeMap<String, Dead>,
    pub judged: bool,
    pub why: Option<String>,
    pub retried: Option<String>,
    pub symlink_rows: usize,
}

/// The diff from the previous commit to this one, as `git diff -M -l0
/// --name-status -z` prints it: `renamed` maps a destination to its
/// source (a rename at the default similarity is the same file),
/// `changed` holds every path (named at this commit) whose content
/// changed — added, modified, or renamed with an edit; a rename alone is
/// no change — `deleted` every path removed, and `touched` how many files
/// the commit changes (one per printed record).
#[derive(Clone, Debug, Default)]
pub struct Step {
    pub renamed: BTreeMap<String, String>,
    pub changed: BTreeSet<String>,
    pub deleted: BTreeSet<String>,
    pub touched: usize,
}

/// A first-parent history, oldest first: `trees[i]` is commit i,
/// `steps[i]` the diff from commit i - 1 (`steps[0]` unused).
#[derive(Clone, Debug, Default)]
pub struct History {
    pub shas: Vec<String>,
    pub trees: Vec<Tree>,
    pub steps: Vec<Step>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Orphaned,
    Born,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Code,
    Document,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub at: usize,
    pub path: String,
    pub state: State,
    pub kind: Kind,
}

/// BARS-GEN2 section 2's four fates; the first that happens decides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    ReReferred,
    KeptUp { focused: bool },
    RemovedUntouched,
    Undetermined,
}

/// An event's fate with where it was decided (G15): `decided_at` the
/// commit whose step decided (None when undetermined), `path_then` the
/// file's name there after the renames (at the tip when undetermined),
/// `met_unjudged` whether the walk passed an unjudged tree (G3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Followed {
    pub fate: Fate,
    pub decided_at: Option<usize>,
    pub path_then: String,
    pub met_unjudged: bool,
}

/// The commit a `kept up` change counts as focused at: at most this
/// many files changed (BARS-GEN2 section 4's judgment constant).
pub const FOCUSED_FILES: usize = 20;
/// The window: `WINDOW` first-parent commits ending `FUTURE` commits
/// before the tip (W = tip~799 .. tip~200).
pub const WINDOW: usize = 600;
pub const FUTURE: usize = 200;

/// The window's commit indices into a history of `len` commits, oldest
/// first: the `WINDOW` commits that end `FUTURE` before the tip, or,
/// with fewer than `WINDOW + FUTURE` commits, every commit up to
/// tip~FUTURE that has a parent.
pub fn window(len: usize) -> std::ops::Range<usize> {
    let end = len.saturating_sub(FUTURE);
    let start = end.saturating_sub(WINDOW).max(1);
    start..end.max(start)
}

/// The path a file had at the parent of commit `c`.
fn before(h: &History, c: usize, path: &str) -> String {
    h.steps[c]
        .renamed
        .get(path)
        .cloned()
        .unwrap_or_else(|| path.to_string())
}

/// The events at commit `c`: each root of `c`'s tree that is new there —
/// `born` when the file did not exist at the parent, `orphaned` when it
/// existed and was not dead. A root dead at the parent (root or
/// dependent) is standing, no event; a dependent is never one. A pair
/// with an unjudged tree yields nothing.
pub fn events_at(h: &History, c: usize) -> Vec<Event> {
    let (p, now) = (&h.trees[c - 1], &h.trees[c]);
    if !(p.judged && now.judged) {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (path, d) in now.dead.iter().filter(|(_, d)| d.root) {
        let was = before(h, c, path);
        let state = if !p.files.contains(&was) {
            State::Born
        } else if p.dead.contains_key(&was) {
            continue;
        } else {
            State::Orphaned
        };
        let kind = if d.document {
            Kind::Document
        } else {
            Kind::Code
        };
        out.push(Event {
            at: c,
            path: path.clone(),
            state,
            kind,
        });
    }
    out
}

/// The event's fate, read forward from its commit to the tip along the
/// renames: at each later commit, in this order, the file exists and is
/// not dead (re-referred); it is still dead and its content changed
/// (kept up, focused when the commit changes at most FOCUSED_FILES
/// files); it was deleted (removed untouched — a content change before
/// would have decided first). At an unjudged commit (G10) the deadness
/// is carried from the last judged tree and git's facts are read as
/// usual: rule 1 cannot fire there, a change keeps up, a deletion
/// removes untouched.
pub fn follow(h: &History, ev: &Event) -> Followed {
    let (mut name, mut dead, mut met) = (ev.path.clone(), true, false);
    for k in ev.at + 1..h.trees.len() {
        let step = &h.steps[k];
        if let Some((to, _)) = step.renamed.iter().find(|(_, from)| **from == name) {
            name = to.clone();
        }
        let tree = &h.trees[k];
        let fate = if tree.judged {
            dead = tree.dead.contains_key(&name);
            (tree.files.contains(&name) && !dead).then_some(Fate::ReReferred)
        } else {
            met = true;
            None
        };
        let fate = fate.or_else(|| {
            if dead && step.changed.contains(&name) {
                Some(Fate::KeptUp {
                    focused: step.touched <= FOCUSED_FILES,
                })
            } else if step.deleted.contains(&name) {
                Some(Fate::RemovedUntouched)
            } else {
                None
            }
        });
        if let Some(fate) = fate {
            return Followed {
                fate,
                decided_at: Some(k),
                path_then: name,
                met_unjudged: met,
            };
        }
    }
    Followed {
        fate: Fate::Undetermined,
        decided_at: None,
        path_then: name,
        met_unjudged: met,
    }
}

/// The event's fate alone.
pub fn fate(h: &History, ev: &Event) -> Fate {
    follow(h, ev).fate
}

/// The side numbers of one corpus (G3, G16): window pairs with an
/// unjudged side, unjudged trees after the window, events whose fate walk
/// met an unjudged tree, the dependents of the window's trees (dead, not
/// a root), the dead rows every tree of the chain dropped as symbolic
/// links, and the trees of the chain judged only when asked again (R2).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sides {
    pub unjudged_pairs: usize,
    pub unjudged_after: usize,
    pub met_unjudged: usize,
    pub dependents: usize,
    pub symlink_rows: usize,
    pub retried: usize,
}

/// Every event of the window with its fate, and the side numbers.
pub fn replayed(h: &History) -> (Vec<(Event, Followed)>, Sides) {
    let w = window(h.trees.len());
    let mut sides = Sides {
        unjudged_after: h.trees[w.end.min(h.trees.len())..]
            .iter()
            .filter(|t| !t.judged)
            .count(),
        symlink_rows: h.trees.iter().map(|t| t.symlink_rows).sum(),
        retried: h.trees.iter().filter(|t| t.retried.is_some()).count(),
        ..Sides::default()
    };
    let mut out = Vec::new();
    for c in w {
        sides.unjudged_pairs += usize::from(!(h.trees[c - 1].judged && h.trees[c].judged));
        sides.dependents += h.trees[c].dead.values().filter(|d| !d.root).count();
        for ev in events_at(h, c) {
            let f = follow(h, &ev);
            sides.met_unjudged += usize::from(f.met_unjudged);
            out.push((ev, f));
        }
    }
    (out, sides)
}
