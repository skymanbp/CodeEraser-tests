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

/// Run git in `dir` and READ it, success asserted (the stderr named):
/// stdout as git wrote it. For a reader whose empty stdout is an
/// answer — a "nothing changed" reading, say —
/// a git that failed (a missing rev, a shallow clone) must not pass as
/// that answer; it is a named refusal instead.
pub fn git_ok(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git");
    assert!(
        out.status.success(),
        "git {args:?} in {} failed: {}",
        dir.display(),
        String::from_utf8_lossy(&out.stderr).trim()
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
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
