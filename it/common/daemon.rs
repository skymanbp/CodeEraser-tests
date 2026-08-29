//! Daemon e2e machinery (split from common/mod.rs when the 1.1.0
//! auth battery's raw-line helper pushed it past the 300-line
//! dogfood wall — the gates.rs/gitio.rs submodule pattern).

use std::path::Path;
use std::process::{Child, Command, Stdio};

/// The spawned daemon, KILLED ON DROP unless a shutdown helper
/// disarms it first. A test that panicked between spawn and shutdown
/// used to leak its daemon; the leaked process kept serving the
/// wiped fixture root (its auth token gone with the wipe), and the
/// next run's spawn starved deterministically on a socket it could
/// never bind nor authenticate to — hit for real during the merge
/// loops (2026-08-26). Tests never touch the inner Child: they pass
/// the guard back into shutdown_and_wait / wait_exit, which disarm.
pub struct DaemonGuard {
    child: Option<Child>,
}

impl From<Child> for DaemonGuard {
    fn from(child: Child) -> Self {
        DaemonGuard { child: Some(child) }
    }
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait(); // reap — no zombie on the panic path
        }
    }
}

/// The `ce daemon <root>` command every spawner shares: null stdin
/// and stdout, `stderr` as the caller needs it (inherited for CI
/// logs, piped to read the serving line), the test idle window.
pub fn daemon_command(root: &Path, stderr: Stdio) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ce"));
    cmd.arg("daemon")
        .arg(root)
        .env("CE_DAEMON_IDLE_SECS", "120")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(stderr);
    cmd
}

/// Spawn the real `ce daemon` for `root` and wait until it answers a
/// ping. Tests must NEVER go through the lazy-spawning
/// `client::request`: it respawns current_exe(), which inside a test
/// harness is the TEST binary — libtest then treats the `daemon` arg
/// as a name filter and runs `*_daemon` tests NESTED, wiping shared
/// tmp dirs and double-serving sockets mid-test (the Windows-CI
/// cold-start flake class). `request_if_running` never spawns; a
/// connect failure is a loud test failure, not silent process spray.
/// Daemon stderr is inherited so CI logs show its cold-start lines.
pub fn spawn_daemon_ready(root: &Path) -> DaemonGuard {
    use codeeraser::daemon::{client, proto::Request};
    let child = daemon_command(root, Stdio::inherit())
        .spawn()
        .expect("spawn ce daemon");
    let guard = DaemonGuard::from(child);
    // 30 s, not 5: readiness is a liveness wait, not a latency claim,
    // and inside the merged `it` crate a cold daemon start co-runs
    // with the whole suite's thread pool — the old 5 s window blew
    // once in 80 amplified loops (2026-08-26). A dead daemon still
    // fails loudly, just later. The panic path needs no manual kill:
    // the guard's Drop reaps.
    for _ in 0..300 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if client::request_if_running(root, &Request::Ping).is_ok() {
            return guard;
        }
    }
    panic!("daemon never came up");
}

/// A raw connection to the daemon for `root`, bypassing the client
/// (its hello/token/respawn behaviors are exactly what the auth,
/// skew, stall, and oversize batteries need to step around).
pub fn raw_daemon_connect(root: &Path) -> std::io::BufReader<interprocess::local_socket::Stream> {
    use interprocess::local_socket::traits::Stream as _;
    use interprocess::local_socket::{GenericNamespaced, Stream, ToNsName};
    let ns = codeeraser::daemon::proto::socket_name(root)
        .to_ns_name::<GenericNamespaced>()
        .expect("name");
    std::io::BufReader::new(Stream::connect(ns).expect("connect"))
}

/// One raw NDJSON line → the daemon's one reply line.
pub fn raw_daemon_line(
    root: &Path,
    req: &codeeraser::daemon::proto::Request,
) -> codeeraser::daemon::proto::Response {
    use std::io::{BufRead, Write};
    let mut conn = raw_daemon_connect(root);
    writeln!(
        conn.get_mut(),
        "{}",
        serde_json::to_string(req).expect("ser")
    )
    .expect("write");
    conn.get_mut().flush().expect("flush");
    let mut reply = String::new();
    conn.read_line(&mut reply).expect("read");
    serde_json::from_str(reply.trim()).expect("parse")
}

/// Shut down the daemon for `dir` AND for every nested project root
/// under it, and CONFIRM each one is gone. A hook judges a write
/// under a nested project (a gated submodule, a nested repository)
/// at that project's own root (root::judging_root, plan v2.18 step
/// #12), so the daemon it lazily starts lives THERE: a teardown that
/// asked only `dir` left one serving `…/suite` for its whole idle
/// window, holding target/debug/ce.exe against the dogfood relink
/// on Windows (CI 33261672033 twice; the census step named it on
/// 33262869599). The confirmation is a liveness wait, spawn_daemon_
/// ready's shape: a daemon still cold-starting refuses the first
/// shutdown (its token lands after the bind), and a refused connect
/// is the only proof it is gone (client::is_running). Never the
/// lazy-spawning path (see spawn_daemon_ready). Residue named: a
/// spawn the hook gave up waiting for (its 2 s budget) has no socket
/// to ask yet and is not seen here — the hook then also reports a
/// degraded probe of ≥ 2 s, which no leg accepts silently.
pub fn shutdown_daemon(dir: &Path) {
    for root in judged_roots(dir) {
        shutdown_confirmed(&root);
    }
}

/// `dir` plus every directory below it carrying a `ce.toml` — the
/// roots a hook run from `dir` can judge at (judging_root delegates
/// only to a nested project with a gate of its own).
fn judged_roots(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut roots = vec![dir.to_path_buf()];
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if !p.is_dir() || matches!(e.file_name().to_str(), Some(".git" | ".ce")) {
                continue;
            }
            if p.join("ce.toml").is_file() {
                roots.push(p.clone());
            }
            stack.push(p);
        }
    }
    roots
}

/// Shutdown asked until the socket is gone — 30 s, the readiness
/// bound above; a daemon that outlives it fails the test out loud.
fn shutdown_confirmed(root: &Path) {
    use codeeraser::daemon::{client, proto::Request};
    for _ in 0..300 {
        if !client::is_running(root) {
            return;
        }
        let _ = client::request_if_running(root, &Request::Shutdown);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("daemon at {} outlived its shutdown", root.display());
}

/// The daemon must still ANSWER (it survived whatever the test threw
/// at it), then take the clean shutdown — the shared tail of the
/// refusal-and-survival cases (auth refusals, second-bind refusal).
pub fn assert_alive_then_shutdown(root: &Path, guard: DaemonGuard, what: &str) {
    use codeeraser::daemon::{client, proto::Request, proto::Response};
    let r = client::request_if_running(root, &Request::Ping).expect("ping the survivor");
    assert!(matches!(r, Response::Pong { .. }), "got {r:?}");
    shutdown_and_wait(root, guard, what);
}

/// Clean daemon shutdown (Bye asserted) then reaped exit — the
/// shared tail of every daemon e2e case.
pub fn shutdown_and_wait(root: &Path, guard: DaemonGuard, what: &str) {
    use codeeraser::daemon::{client, proto::Request, proto::Response};
    match client::request_if_running(root, &Request::Shutdown).expect("shutdown") {
        Response::Bye => {}
        other => panic!("expected bye, got {other:?}"),
    }
    wait_exit(guard, what);
}

/// Wait ~5s for the daemon to exit; kill it and panic on timeout
/// (the disarmed guard's Drop is inert, so the timeout path reaps by
/// hand — and a panic BEFORE the wait still reaps through the guard).
pub fn wait_exit(mut guard: DaemonGuard, what: &str) {
    let mut child = guard.child.take().expect("guard already disarmed");
    for _ in 0..50 {
        if let Ok(Some(_)) = child.try_wait() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let _ = child.kill();
    let _ = child.wait(); // reap — no zombie on the panic path
    panic!("{what} did not exit");
}
