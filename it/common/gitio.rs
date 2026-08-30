//! Test-side git spawner (throwaway identity, success asserted),
//! split from common/mod.rs at the repo's own 300-line dogfood gate.

use std::path::Path;
use std::process::Command;

/// Run git in `dir` with a throwaway identity; panic on failure.
pub fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.email=t@t", "-c", "user.name=t"])
        .args(args)
        .output()
        .expect("git");
    assert!(out.status.success(), "git {args:?}: {out:?}");
}

/// Run git in `dir` and READ it: `(succeeded, stdout)`. The identity is
/// the caller's, because every user of this form interrogates history
/// rather than writing it. Both halves are answers: `merge-base
/// --is-ancestor` states its finding in the exit status and says
/// nothing on stdout, while `log`/`ls-tree` state theirs the other way
/// round.
pub fn git_out(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}
