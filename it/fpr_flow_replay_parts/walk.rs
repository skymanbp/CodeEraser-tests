//! The flow ledger's walk over history: every first-parent commit of
//! the window against its parent, each changed file of the language an
//! event — renames followed (`-M50%`: a unit keeps its identity when
//! its file moves, and a pure rename, like stringr's 36 `.r` -> `.R`
//! files, is no edit at all), the parent and child versions judged
//! (side.rs) and the book told (book.rs). Git is read through the
//! history helpers every FPR instrument shares.

use super::book::{Book, Event, Tally};
use super::shape::{Side, Span};
use super::side::side;
use crate::common::{blob, git_out};
use codeeraser::corelink::Link;
use codeeraser::scan::lang::Lang;
use std::path::Path;

/// One changed file: its parent-side path if the parent had it as the
/// language, its child-side path likewise.
#[derive(Debug, PartialEq, Eq)]
pub struct Change {
    pub old: Option<String>,
    pub new: Option<String>,
}

/// The language's changed files between two commits, renames paired.
/// `-z` keeps non-ASCII paths unquoted.
pub fn changes(repo: &Path, parent: &str, commit: &str, lang: Lang) -> Vec<Change> {
    let args = ["diff", "--name-status", "-z", "-M50%", parent, commit];
    let (ok, out) = git_out(repo, &args);
    assert!(ok, "git {args:?} in {}", repo.display());
    let ours = |p: &str| Lang::from_path(Path::new(p)) == Some(lang);
    let mut fields = out.split('\0').filter(|f| !f.is_empty());
    let mut found = Vec::new();
    while let Some(status) = fields.next() {
        let first = fields.next().expect("a path after the status").to_string();
        let (old, new) = match status.as_bytes()[0] {
            b'A' => (None, Some(first)),
            b'D' => (Some(first), None),
            b'R' | b'C' => (Some(first), fields.next().map(str::to_string)),
            _ => (Some(first.clone()), Some(first)),
        };
        let change = Change {
            old: old.filter(|p| ours(p)),
            new: new.filter(|p| ours(p)),
        };
        if change.old.is_some() || change.new.is_some() {
            found.push(change);
        }
    }
    found
}

/// The parent-side lines an edit touched, from `git diff -U0`'s hunk
/// heads: `-a,b` covers a..a+b-1; a pure insertion (b = 0) after line
/// a touches its two neighbours a and a + 1.
pub fn hunks(repo: &Path, before: &str, after: &str) -> Vec<Span> {
    let args = [
        "diff",
        "-U0",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
    ];
    let (_, out) = git_out(repo, &[&args[..], &[before, after]].concat());
    out.lines()
        .filter_map(|l| l.strip_prefix("@@ -"))
        .filter_map(|l| l.split(' ').next())
        .map(|range| {
            let (a, b) = range.split_once(',').unwrap_or((range, "1"));
            let (a, b): (u32, u32) = (a.parse().unwrap_or(0), b.parse().unwrap_or(1));
            if b == 0 { (a, a + 1) } else { (a, a + b - 1) }
        })
        .collect()
}

/// One version of a file at a commit, judged.
fn judged(link: &mut Link, repo: &Path, rev: &str, rel: &str, lang: Lang) -> Side {
    let bytes = blob(repo, rev, rel);
    let text = String::from_utf8_lossy(&bytes);
    side(link, &text, lang, &format!("{}@{rev}", rel))
}

/// The window walked oldest first (`commits[0]` the seed): every event
/// told to one book.
pub fn replay(repo: &Path, lang: Lang, commits: &[String], link: &mut Link) -> Tally {
    let mut book = Book::default();
    for pair in commits.windows(2) {
        let (parent, commit) = (&pair[0], &pair[1]);
        for ch in changes(repo, parent, commit, lang) {
            let old = ch
                .old
                .as_deref()
                .map(|p| judged(link, repo, parent, p, lang));
            let new = ch
                .new
                .as_deref()
                .map(|p| judged(link, repo, commit, p, lang));
            let touched = match (&ch.old, &ch.new) {
                (Some(o), Some(n)) => {
                    hunks(repo, &format!("{parent}:{o}"), &format!("{commit}:{n}"))
                }
                _ => Vec::new(),
            };
            book.event(&Event {
                old: ch.old.as_deref().zip(old.as_ref()),
                new: ch.new.as_deref().zip(new.as_ref()),
                hunks: &touched,
            });
            for (path, (name, params, _), (kind, anchor)) in book.fell.drain(..) {
                println!(
                    "  FALSE {} {path} {name}/{params} kind {kind}: {anchor}",
                    &commit[..8]
                );
            }
        }
    }
    book.finish()
}
