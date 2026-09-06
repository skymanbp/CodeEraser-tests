//! History reads shared by the duplicate-write, tombstone, zone and
//! changeset FPR instruments. Git operations have one owner; callers
//! select their event population and replay direction.
//!
//! CE_FPR_TIP = the commit whose first-parent chain is walked
//! (default HEAD; the ledger's requests window ends at 1f6589ec);
//! CE_FPR_LIMIT = only the newest N commits, seeded from the tree
//! before them (a smoke run; the ledgers' numbers are full runs).
//! CE_FPR_REPO is read by each instrument's own entry point, so a
//! synthetic-history test can call the walk on a repository it built.

use super::gitio::git_out;
use std::path::Path;

/// A git command's stdout as lines, success asserted.
pub fn git_lines(repo: &Path, args: &[&str]) -> Vec<String> {
    let (ok, out) = git_out(repo, args);
    assert!(ok, "git {args:?} in {}", repo.display());
    out.lines().map(str::to_string).collect()
}

/// Git's first-parent order, newest first. Forward replays reverse it.
pub fn first_parent(repo: &Path, tip: &str) -> Vec<String> {
    git_lines(repo, &["rev-list", "--first-parent", tip])
}

/// A revision is present, including the parent needed by a changeset.
pub fn revision_exists(repo: &Path, rev: &str) -> bool {
    git_out(repo, &["rev-parse", "--verify", "-q", rev]).0
}

/// A blob's BYTES — not lossy text: a file is replayed as the bytes
/// it was.
pub fn blob(repo: &Path, rev: &str, rel: &str) -> Vec<u8> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["show", &format!("{rev}:{rel}")])
        .output()
        .expect("git show");
    assert!(out.status.success(), "git show {rev}:{rel}");
    out.stdout
}

/// `(status, path)` for every path `commit` changed against `parent`
/// — renames as delete + add, the way a Write sees them. Adds,
/// modifications and deletions; each caller filters for the
/// population it measures, so the git call and its parse live once.
pub fn changed(repo: &Path, parent: &str, commit: &str) -> Vec<(char, String)> {
    git_lines(
        repo,
        &["diff", "--name-status", "--no-renames", parent, commit],
    )
    .into_iter()
    .filter_map(|l| {
        let (status, path) = l.split_once('\t')?;
        let status = status.chars().next()?;
        matches!(status, 'A' | 'M' | 'D').then(|| (status, path.to_string()))
    })
    .collect()
}

/// The commits to replay, oldest first: the whole first-parent chain
/// under the tip, or under CE_FPR_LIMIT the newest N with the commit
/// before them as the seed.
pub fn chain(repo: &Path) -> Vec<String> {
    let tip = std::env::var("CE_FPR_TIP").unwrap_or_else(|_| "HEAD".into());
    let mut all = first_parent(repo, &tip);
    all.reverse();
    assert!(all.len() >= 2, "need history to replay");
    match std::env::var("CE_FPR_LIMIT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        Some(n) if n < all.len() - 1 => all[all.len() - n - 1..].to_vec(),
        _ => all,
    }
}
