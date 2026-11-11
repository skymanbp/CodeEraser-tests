//! A hook that does not judge leaves a trace (A9f), on the three paths
//! that passed in silence (v2.35 G4): an envelope over the intake cap
//! (R3-13, the largest Writes), a POSIX-form `cwd` on Windows (P-A3),
//! and the Stop audit's four-class leg with no daemon running (P-A2:
//! `daemon_unavailable` under a healthy top-level bit, every Stop of a
//! session that never wrote first). The legs where no daemon can
//! answer — the Stop's four-class leg reading `daemon_unavailable`, the
//! probe failing open — are pinned here on purpose, with a stand-in
//! holding the socket, never by a race; the cold legs run their hooks
//! through run_hook_cold / run_hook_alive, never the probe pre-warm.

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

/// A committed `a.py` grown by one function since: the Stop's
/// four-class leg has a pair to judge.
fn grown(tag: &str) -> std::path::PathBuf {
    let dir = project(tag);
    std::fs::write(dir.join("a.py"), "def f(a):\n    return a + 1\n").expect("a.py");
    common::init_and_commit(&dir, "one");
    std::fs::write(
        dir.join("a.py"),
        "def f(a):\n    return a + 1\n\ndef g(b):\n    return b * 2\n",
    )
    .expect("grow a.py");
    dir
}

/// The feed's last `stop_audit` line.
fn stop_line(dir: &Path) -> serde_json::Value {
    common::observe_lines(dir)
        .into_iter()
        .rfind(|l| l["event"] == "stop_audit")
        .expect("a stop_audit line")
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
    let dir = grown("trace-cold-stop");
    common::run_hook_cold(
        &dir,
        &["audit", "--hook"],
        &common::stop_envelope(&dir, false),
        &[],
    );
    let line = stop_line(&dir);
    assert_eq!(
        line["fourclass"]["degraded"],
        serde_json::Value::Null,
        "the cold daemon was started and judged: {}",
        line["fourclass"]
    );
    assert_eq!(line["fourclass"]["added_novel"], 3, "{}", line["fourclass"]);
    std::fs::remove_dir_all(&dir).ok();
}

/// A stand-in holding `root`'s daemon socket that accepts every
/// connection and never says a word. A real daemon cannot bind beside
/// it (the bind sees a live holder and leaves — server/bind.rs), and
/// the client's first connect lands on it, so no daemon is ever spawned
/// and every request ends at the client's deadline: unavailable by
/// construction, whatever the machine's load.
struct Mute {
    name: String,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    thread: std::thread::JoinHandle<()>,
}

/// The daemon socket's name as the local-socket layer takes it.
fn ns(name: &str) -> interprocess::local_socket::Name<'static> {
    use interprocess::local_socket::{GenericNamespaced, ToNsName};
    name.to_owned()
        .to_ns_name::<GenericNamespaced>()
        .expect("name")
}

fn mute(root: &Path) -> Mute {
    use interprocess::local_socket::ListenerOptions;
    use interprocess::local_socket::traits::ListenerExt as _;
    use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
    let name = codeeraser::daemon::proto::socket_name(root);
    let listener = ListenerOptions::new()
        .name(ns(&name))
        .create_sync()
        .expect("bind");
    let stop = std::sync::Arc::new(AtomicBool::new(false));
    let seen = stop.clone();
    let thread = std::thread::spawn(move || {
        let mut held = Vec::new();
        for conn in listener.incoming() {
            if seen.load(SeqCst) {
                break;
            }
            held.extend(conn.ok());
        }
    });
    Mute { name, stop, thread }
}

impl Mute {
    /// Stop accepting and wait for the stand-in to let go of the name:
    /// one connect wakes its accept (repeated while the name is busy
    /// between two accepts), 30 s the liveness bound common/daemon.rs
    /// uses.
    fn release(self) {
        use interprocess::local_socket::Stream;
        use interprocess::local_socket::traits::Stream as _;
        self.stop.store(true, std::sync::atomic::Ordering::SeqCst);
        for _ in 0..300 {
            if self.thread.is_finished() {
                return self.thread.join().expect("the stand-in");
            }
            let _ = Stream::connect(ns(&self.name));
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        panic!("the stand-in never let go of {}", self.name);
    }
}

#[test]
fn an_unreachable_daemon_degrades_the_four_class_leg() {
    let dir = grown("trace-mute-stop");
    let stand_in = mute(&dir);
    common::run_hook_alive(
        &dir,
        &["audit", "--hook"],
        &common::stop_envelope(&dir, false),
        &[("CE_CLIENT_DEADLINE_SECS", "1")],
    );
    stand_in.release();
    common::shutdown_daemon(&dir);
    let line = stop_line(&dir);
    assert_eq!(
        line["fourclass"],
        serde_json::json!({"degraded": "daemon_unavailable"}),
        "the leg names why it did not judge"
    );
    assert_eq!(
        line["degraded"], false,
        "the top-level bit is the dedup leg's, which measured"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// The probe with no daemon able to come up: the stand-in holds the
/// socket and never answers, so the write passes (fail-open, nothing on
/// stdout) and the probe line says it did not judge. No timing decides
/// it — the client's deadline only bounds how long the hook waits.
#[test]
fn an_unreachable_daemon_lets_the_probe_fail_open() {
    let dir = project("trace-mute-probe");
    std::fs::write(
        dir.join("a.py"),
        "def f(a):
    return a + 1
",
    )
    .expect("a.py");
    common::build_index(&dir);
    let stand_in = mute(&dir);
    let out = common::run_hook_alive(
        &dir,
        &["probe", "--hook"],
        &common::pretooluse_envelope_at(
            &dir,
            "b.py",
            "Write",
            "def f(a):
    return a + 1
",
        ),
        &[("CE_CLIENT_DEADLINE_SECS", "1")],
    );
    stand_in.release();
    common::shutdown_daemon(&dir);
    assert_eq!(out.trim(), "", "a probe that cannot judge says nothing");
    let line = common::last_observe(&dir);
    assert_eq!(line["event"], "probe", "{line}");
    assert_eq!(line["degraded"], true, "{line}");
    std::fs::remove_dir_all(&dir).ok();
}
