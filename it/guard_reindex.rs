//! The session's own writes become probe candidates (v2.35 guard lane,
//! GD7). The write-time probe never refreshes the index and nothing
//! refreshed at PostToolUse, so between two Stops a file the session
//! had just written was invisible to the next probe: a copy of code
//! written minutes ago passed. Now the PostToolUse leg asks a RUNNING
//! daemon to re-measure the one file that landed. Legs: a new file and
//! an edited one become candidates; a file excluded, of the scan-only
//! arm, or of a language that never fingerprints is not indexed;
//! without a running daemon nothing happens; an index of another
//! generation is not touched.

use crate::common::{self, run_hook, spawn_daemon_ready, tmp, write_all};
// the files the index holds, read without opening it as an Index (an
// open would rebuild a key it finds stale)
use crate::scoped_index::indexed;
use std::path::Path;

const F: &str = "def own_copy(rows, key, limit):\n    seen = {}\n    for row in rows:\n        k = row.get(key)\n        if k is None:\n            continue\n        bucket = seen.setdefault(k, [])\n        if len(bucket) >= limit:\n            raise ValueError(\"too many rows for %s\" % k)\n        bucket.append(row)\n    total = sum(len(v) for v in seen.values())\n    return sorted(seen.items(), key=lambda kv: (len(kv[1]), kv[0])), total\n";
const G: &str = "def unrelated(x):\n    return x + 1\n";

/// The full hook sequence for one landed write — PreToolUse, the bytes
/// on disk, PostToolUse — with the project's daemon running (the
/// caller's guard holds it; the re-measure never starts one).
fn landed(dir: &Path, rel: &str, text: &str) {
    let (pre, post) = (
        common::pretooluse_envelope_at(dir, rel, "Write", text),
        common::posttooluse_envelope(dir, rel, "Write", "t"),
    );
    let first = common::run_hook_alive(dir, &["probe", "--hook"], &pre, &[]);
    assert!(
        first.trim().is_empty(),
        "{rel}: the first write passes: {first}"
    );
    write_all(dir, &[(rel, text)]);
    common::run_hook_alive(dir, &["settle", "--hook"], &post, &[]);
}

/// A seeded, indexed tree with its daemon up.
fn seeded(tag: &str, files: &[(&str, &str)]) -> (std::path::PathBuf, common::DaemonGuard) {
    let dir = tmp(tag);
    write_all(&dir, files);
    common::build_index(&dir);
    let guard = spawn_daemon_ready(&dir);
    (dir, guard)
}

#[test]
fn a_file_written_this_session_is_a_candidate_for_the_next_write() {
    let (dir, guard) = seeded("guard-reindex-new", &[("other.py", G)]);
    landed(&dir, "x.py", F);
    common::expect_write_denied(&dir, "y.py", F, "x.py:1-");
    drop(guard);
}

#[test]
fn a_function_added_by_an_edit_is_a_candidate_for_the_next_write() {
    let (dir, guard) = seeded("guard-reindex-edit", &[("x.py", G)]);
    landed(&dir, "x.py", &format!("{G}\n\n{F}"));
    common::expect_write_denied(&dir, "y.py", F, "x.py:5-");
    drop(guard);
}

#[test]
fn only_this_indexs_own_measured_files_are_refreshed() {
    let (dir, guard) = seeded(
        "guard-reindex-scope",
        &[("ce.toml", "exclude = [\"vendor/\"]\n"), ("other.py", G)],
    );
    std::fs::create_dir_all(dir.join("vendor")).expect("mkdir");
    landed(&dir, "vendor/v.py", F);
    landed(&dir, "s.sh", "echo one\necho two\n");
    landed(&dir, "n.md", "# note\n");
    landed(&dir, "x.py", F);
    common::shutdown_and_wait(&dir, guard, "daemon after the scope leg");
    let paths = indexed(&dir);
    assert!(
        paths.iter().any(|p| p == "x.py"),
        "the control is indexed: {paths:?}"
    );
    assert!(
        !paths
            .iter()
            .any(|p| p == "vendor/v.py" || p == "s.sh" || p == "n.md"),
        "{paths:?}"
    );
}

#[test]
fn without_a_running_daemon_nothing_happens() {
    let dir = tmp("guard-reindex-nodaemon");
    write_all(&dir, &[("other.py", G)]);
    common::build_index(&dir);
    common::shutdown_daemon(&dir);
    write_all(&dir, &[("x.py", F)]);
    let post = common::posttooluse_envelope(&dir, "x.py", "Write", "t");
    let out = run_hook(&dir, &["settle", "--hook"], &post);
    assert!(out.trim().is_empty(), "{out}");
    assert!(!indexed(&dir).contains(&"x.py".to_string()));
    assert!(
        !common::observe_lines(&dir)
            .iter()
            .any(|l| l["event"] == "settled"),
        "no failure to record"
    );
}

#[test]
fn an_index_of_another_generation_is_not_touched() {
    let (dir, guard) = seeded("guard-reindex-generation", &[("other.py", G)]);
    let conn = rusqlite::Connection::open(dir.join(".ce/index.db")).expect("db");
    conn.pragma_update(None, "user_version", 1)
        .expect("age the key");
    drop(conn);
    let before = indexed(&dir);
    write_all(&dir, &[("x.py", F)]);
    let post = common::posttooluse_envelope(&dir, "x.py", "Write", "t");
    common::run_hook_alive(&dir, &["settle", "--hook"], &post, &[]);
    common::shutdown_and_wait(&dir, guard, "daemon after the generation leg");
    assert_eq!(
        indexed(&dir),
        before,
        "another generation's index is not touched"
    );
    let v: i64 = rusqlite::Connection::open(dir.join(".ce/index.db"))
        .expect("db")
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .expect("version");
    assert_eq!(v, 1, "nor rebuilt");
}
