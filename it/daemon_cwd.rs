//! The daemon never pins its root. Spawned from a hook whose cwd is
//! the project root, it inherits that cwd, and Windows refuses to
//! delete a process's cwd — an idling (30 min) or exiting daemon would
//! hold the project against deletion. The demo replay's work tree hit
//! exactly that under the parallel suite: eject's Bye is answered
//! before the exit completes, and the driver's rm raced it. serve()
//! leaves for the system temp dir and says so on its serving line.

use crate::common::{DaemonGuard, daemon_command, tmp};
use std::io::{BufRead, BufReader};
use std::process::Stdio;

#[test]
fn the_daemon_leaves_its_root_for_the_temp_dir() {
    let root = tmp("daemon-cwd");
    let mut cmd = daemon_command(&root, Stdio::piped());
    cmd.current_dir(&root); // the hook's inheritance
    let mut child = cmd.spawn().expect("spawn ce daemon");
    let stderr = child.stderr.take().expect("stderr");
    let _reap = DaemonGuard::from(child); // killed on drop, panic path included
    let serving = BufReader::new(stderr)
        .lines()
        .map(|l| l.expect("stderr line"))
        .find(|l| l.contains("serving"))
        .expect("daemon stderr ended before the serving line");
    let away = std::env::temp_dir().display().to_string();
    let away = away.trim_end_matches(['\\', '/']);
    assert!(serving.contains(&format!("(cwd {away})")), "{serving}");
    assert!(
        !serving.contains(&format!("(cwd {}", root.display())),
        "{serving}"
    );
    // the decisive form on Windows: with the daemon alive, the root is
    // "not empty" (ERROR_DIR_NOT_EMPTY 145), never "in use"
    // (ERROR_SHARING_VIOLATION 32)
    let err = std::fs::remove_dir(&root).expect_err("the root has content");
    if cfg!(windows) {
        assert_eq!(err.raw_os_error(), Some(145), "{err}");
    }
}
