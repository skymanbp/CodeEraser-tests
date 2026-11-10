//! A hook that does not judge leaves a trace (A9f), on the three paths
//! that passed in silence (v2.35 G4): an envelope over the intake cap
//! (R3-13, the largest Writes), a POSIX-form `cwd` on Windows (P-A3),
//! and the Stop audit's four-class leg with no daemon running (P-A2:
//! `daemon_unavailable` under a healthy top-level bit, every Stop of a
//! session that never wrote first).

use crate::common;
use std::io::Write as _;
use std::path::Path;
use std::process::Stdio;

/// A project the hook process runs in.
fn project(tag: &str) -> std::path::PathBuf {
    let dir = common::tmp(tag);
    std::fs::write(dir.join("ce.toml"), "\n").expect("ce.toml");
    dir
}

/// `ce probe --hook` on `stdin`, the exit asserted 0. The hook stops
/// reading at the cap, so the tail of an over-cap write may meet a
/// closed pipe: that error is the point, not a failure.
fn probe(dir: &Path, stdin: &[u8]) {
    let mut child = common::ce_command(dir, &[])
        .args(["probe", "--hook"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn probe");
    let _ = child.stdin.take().expect("stdin").write_all(stdin);
    let status = child.wait().expect("wait");
    assert!(status.success(), "a hook always exits 0");
}

/// The feed's last line is the probe's degraded trace naming `needle`.
fn traced(dir: &Path, needle: &str) {
    let line = common::last_observe(dir);
    assert_eq!(line["event"], "probe");
    assert_eq!(line["degraded"], true);
    let reason = line["reason"].as_str().expect("reason");
    assert!(reason.contains(needle), "the trace names why: {reason}");
}

#[test]
fn an_over_cap_envelope_leaves_a_degraded_line() {
    let dir = project("trace-overcap");
    let file = dir.join("x.py").display().to_string().replace('\\', "/");
    let envelope = serde_json::json!({
        "session_id": "t", "transcript_path": "t",
        "cwd": dir.display().to_string().replace('\\', "/"),
        "hook_event_name": "PreToolUse", "tool_name": "Write",
        "tool_input": {"file_path": file, "content": "x".repeat(5 << 20)}
    });
    probe(&dir, envelope.to_string().as_bytes());
    traced(&dir, "cap");
    std::fs::remove_dir_all(&dir).ok();
}

#[cfg(windows)]
#[test]
fn a_posix_cwd_leaves_a_degraded_line() {
    let dir = project("trace-posix");
    let native = dir.display().to_string().replace('\\', "/");
    let (drive, rest) = native.split_once(':').expect("a drive letter");
    let posix = format!("/{}{rest}", drive.to_ascii_lowercase());
    let envelope = serde_json::json!({
        "session_id": "t", "transcript_path": "t", "cwd": posix,
        "hook_event_name": "PreToolUse", "tool_name": "Write",
        "tool_input": {"file_path": format!("{posix}/x.py"), "content": "y = 1\n"}
    });
    probe(&dir, envelope.to_string().as_bytes());
    traced(&dir, "not a Windows path");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_cold_stop_judges_the_four_class_leg() {
    let dir = project("trace-cold-stop");
    std::fs::write(dir.join("a.py"), "def f(a):\n    return a + 1\n").expect("a.py");
    common::init_and_commit(&dir, "one");
    std::fs::write(
        dir.join("a.py"),
        "def f(a):\n    return a + 1\n\ndef g(b):\n    return b * 2\n",
    )
    .expect("grow a.py");
    common::run_hook(
        &dir,
        &["audit", "--hook"],
        &common::stop_envelope(&dir, false),
    );
    let line = common::observe_lines(&dir)
        .into_iter()
        .rfind(|l| l["event"] == "stop_audit")
        .expect("a stop_audit line");
    assert_eq!(
        line["fourclass"]["degraded"],
        serde_json::Value::Null,
        "the cold daemon was started and judged: {}",
        line["fourclass"]
    );
    assert_eq!(line["fourclass"]["added_novel"], 3, "{}", line["fourclass"]);
    std::fs::remove_dir_all(&dir).ok();
}
