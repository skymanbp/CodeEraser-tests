use super::{Shared, build};
use crate::daemon::judge::Judge;
use crate::daemon::proto::{Request, Response};
use std::sync::Mutex;
use std::time::Instant;

/// The judge must be free the moment the response exists: the
/// caller still has a blocking socket write ahead of it, and
/// while the guard lived across that write one client which
/// stopped reading its end blocked every other connection.
#[test]
fn build_releases_the_judge_before_the_caller_writes() {
    let shared = Shared {
        root: std::env::temp_dir(),
        start: Instant::now(),
        token: String::new(),
        judge: Mutex::new(Judge::default()),
    };
    let (resp, keep) = build(&shared, Request::Ping);
    assert!(matches!(resp, Response::Pong { .. }), "got {resp:?}");
    assert!(keep, "a ping never retires the daemon");
    assert!(
        shared.judge.try_lock().is_ok(),
        "another connection must be able to take the judge now"
    );
}
