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
/// product refused the tree: a pair touching it yields no event.
#[derive(Clone, Debug, Default)]
pub struct Tree {
    pub files: BTreeSet<String>,
    pub dead: BTreeMap<String, Dead>,
    pub judged: bool,
}

/// The diff from the previous commit to this one, as `git diff -M
/// --name-status` prints it: `renamed` maps a destination to its source
/// (a rename at the default similarity is the same file), `changed`
/// holds every path (named at this commit) whose content changed —
/// added, modified, or renamed with an edit; a rename alone is no change
/// — `deleted` every path removed, and `touched` how many files the
/// commit changes (one per printed line).
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
/// would have decided first). An unjudged tree decides nothing.
pub fn fate(h: &History, ev: &Event) -> Fate {
    let mut name = ev.path.clone();
    for k in ev.at + 1..h.trees.len() {
        let step = &h.steps[k];
        if let Some((to, _)) = step.renamed.iter().find(|(_, from)| **from == name) {
            name = to.clone();
        }
        let tree = &h.trees[k];
        if !tree.judged {
            continue;
        }
        if tree.files.contains(&name) && !tree.dead.contains_key(&name) {
            return Fate::ReReferred;
        }
        if tree.dead.contains_key(&name) && step.changed.contains(&name) {
            return Fate::KeptUp {
                focused: step.touched <= FOCUSED_FILES,
            };
        }
        if step.deleted.contains(&name) {
            return Fate::RemovedUntouched;
        }
    }
    Fate::Undetermined
}

/// Every event of the window with its fate; the dependents of the
/// window's trees counted beside (the side table: dead, not a root).
pub fn replayed(h: &History) -> (Vec<(Event, Fate)>, usize) {
    let mut out = Vec::new();
    let mut dependents = 0;
    for c in window(h.trees.len()) {
        dependents += h.trees[c].dead.values().filter(|d| !d.root).count();
        for ev in events_at(h, c) {
            let f = fate(h, &ev);
            out.push((ev, f));
        }
    }
    (out, dependents)
}
