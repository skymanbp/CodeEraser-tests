//! The daemon client's test half, split from client.rs at the
//! 300-line dogfood wall when the 2.0.0 staleness legs grew it past
//! its ceiling (the candidates_tests.rs precedent): the scripted
//! stale daemons and the trust ordering they exercise.

use super::{Request, request_if_running, socket_name, stale};
use crate::daemon::cancel::{Canceller, GRACE, PARKED, bounded_with};
use interprocess::local_socket::traits::ListenerExt;
use interprocess::local_socket::{GenericNamespaced, Listener, ListenerOptions, ToNsName};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::Ordering::SeqCst;
use std::time::{Duration, Instant};

/// The gauge is process-wide, so the two legs that read it take
/// turns; a poisoned lock (a failed leg) must not hide the other.
static GAUGE: Mutex<()> = Mutex::new(());

/// The deadline every measured leg hands `bounded_with`.
const DEADLINE: Duration = Duration::from_millis(300);

/// Bind this root's socket, the way every scripted daemon here does.
fn listen(root: &Path) -> Listener {
    let ns = socket_name(root)
        .to_ns_name::<GenericNamespaced>()
        .expect("name");
    ListenerOptions::new().name(ns).create_sync().expect("bind")
}

/// One measured conversation with a daemon that accepts and then
/// says NOTHING for `hold` (from the accept, which the client's own
/// connect drives) before dropping the connection: the refusal's
/// text, how long the client waited, the gauge before it asked, and
/// the daemon's thread to join. The deadline is the caller's — the
/// race leg pins it at zero.
struct Probe {
    err: String,
    elapsed: Duration,
    parked: usize,
    held: std::thread::JoinHandle<()>,
}

fn probe(tag: &str, hold: Duration, deadline: Duration, canceller: Canceller) -> Probe {
    let root = crate::testutil::scratch(tag);
    let listener = listen(&root);
    let held = std::thread::spawn(move || {
        let stream = listener.incoming().next().expect("conn").expect("accept");
        std::thread::sleep(hold);
        drop(stream);
    });
    let parked = PARKED.load(SeqCst);
    let started = Instant::now();
    let err = bounded_with(&root, &Request::Ping, false, deadline, canceller)
        .expect_err("silence must not be an answer")
        .to_string();
    Probe {
        err,
        elapsed: started.elapsed(),
        parked,
        held,
    }
}

/// The gauge settles to `want` within `within`, or the leg fails
/// with the value it saw last.
fn settles(want: usize, within: Duration) {
    let until = Instant::now() + within;
    while PARKED.load(SeqCst) != want && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(PARKED.load(SeqCst), want, "the parked gauge settles");
}

/// A scripted pre-1.1.0 daemon: it answers ONE hello with `proto`
/// (checking no credential, as such a daemon does), then records
/// every further line and answers each with a bye. The handle
/// yields the recorded lines once the client drops the socket.
fn stub_daemon(root: &Path, proto: &str) -> std::thread::JoinHandle<Vec<String>> {
    let listener = listen(root);
    let hello_ok = format!(r#"{{"type":"hello_ok","proto":"{proto}"}}"#);
    std::thread::spawn(move || {
        let stream = listener.incoming().next().expect("conn").expect("accept");
        let mut conn = BufReader::new(stream);
        let mut lines: Vec<String> = Vec::new();
        let mut first = true;
        loop {
            let mut got = String::new();
            if conn.read_line(&mut got).unwrap_or(0) == 0 {
                return lines;
            }
            let reply = if first {
                hello_ok.as_str()
            } else {
                r#"{"type":"bye"}"#
            };
            writeln!(conn.get_mut(), "{reply}").expect("write");
            conn.get_mut().flush().expect("flush");
            if !first {
                lines.push(got.trim().to_string()); // post-hello only
            }
            first = false;
        }
    })
}

/// The non-lazy path must RETIRE the daemon whose HelloOk it
/// refuses to trust, not just report it: `ce doctor` rides this
/// path, and reporting alone left an unauthenticated pre-1.1.0
/// daemon serving until a lazy client happened by (review
/// 2026-08-20 #1). The staleness still reaches the caller.
/// Two legs by major: a CROSS-major stale hello (1.x since the
/// 2.0.0 field cut) is reported and NOT sent a shutdown — that
/// daemon owes us `restart` and its shutdown word is not ours to
/// assume; a SAME-major older minor is asked to leave. The
/// same-major leg has no specimen while the current proto sits
/// at an x.0.0 floor (no older minor exists to be stale), and
/// re-arms by itself at the first 2.1.0.
#[test]
fn non_lazy_retires_the_stale_daemon_it_refuses_to_trust() {
    let root = crate::testutil::scratch("client-stale");
    let stub = stub_daemon(&root, "1.0.0");
    let err = request_if_running(&root, &Request::Ping).expect_err("stale is refused");
    assert!(
        err.to_string().contains("stale daemon"),
        "the caller still sees the staleness: {err}"
    );
    let sent = stub.join().expect("stub thread");
    assert!(
        sent.is_empty(),
        "a cross-major daemon is not sent our shutdown word: {sent:?}"
    );
    let Some(older) = same_major_older(super::DAEMON_PROTO) else {
        return; // x.0.0: no same-major stale minor can exist yet
    };
    let root = crate::testutil::scratch("client-stale-minor");
    let stub = stub_daemon(&root, &older);
    request_if_running(&root, &Request::Ping).expect_err("stale is refused");
    let sent = stub.join().expect("stub thread");
    assert_eq!(sent.len(), 1, "one line follows the hello: {sent:?}");
    assert!(
        sent[0].contains("shutdown"),
        "the same-major stale daemon is asked to leave: {sent:?}"
    );
}

/// `major.(minor-1).0` when the minor is above 0 — the newest
/// proto a same-major stale daemon could still speak.
fn same_major_older(proto: &str) -> Option<String> {
    let (major, minor) = super::super::proto::major_minor(proto)?;
    (minor > 0).then(|| format!("{major}.{}.0", minor - 1))
}

/// The staleness ordering the HelloOk trust check rides on: an
/// older major (1.x predates the field cut, 1.0 the credential
/// gate) and garbage are stale; the current proto and NEWER
/// same-major minors (additive by the versioning contract) are
/// not.
#[test]
fn stale_orders_minors_and_distrusts_garbage() {
    assert!(stale("1.0.0"), "pre-credential daemon");
    assert!(stale("1.1.0"), "pre-cut daemon, older major");
    assert!(stale("not-a-version"), "unparseable proto");
    assert!(!stale(super::DAEMON_PROTO), "current");
    assert!(!stale("2.99.0"), "newer same-major minor is additive");
}

/// The #85 close, measured: a daemon that accepts and then says
/// NOTHING no longer parks the client forever. The deadline is
/// passed straight to the bounded call — an env pin here would race
/// the parallel tests reading the same process environment. And the
/// O64 half: the worker the deadline gave up on is TORN DOWN, not
/// parked — the refusal comes back well inside the grace (the
/// cancelled read returned, so nothing was detached), and the gauge
/// never moved, while the daemon still holds its end for seconds.
#[test]
fn a_silent_daemon_meets_the_deadline_not_forever() {
    let _turn = GAUGE.lock().unwrap_or_else(|e| e.into_inner());
    let p = probe(
        "client-silent",
        Duration::from_secs(5),
        DEADLINE,
        Canceller::new(),
    );
    assert!(
        p.elapsed < DEADLINE + GRACE,
        "the worker came back inside the grace, not at its end: {:?}",
        p.elapsed
    );
    assert!(
        p.err.contains("did not answer within") && !p.err.contains("detached"),
        "the refusal names the deadline and no residue: {}",
        p.err
    );
    assert_eq!(PARKED.load(SeqCst), p.parked, "nothing was detached");
    p.held.join().expect("stub");
}

/// One inert-canceller conversation, held to the full parked
/// contract: the whole grace is spent, the residue is named with
/// its stage, the gauge counts the detached worker, and it settles
/// back once the daemon lets go. The daemon holds for longer than
/// deadline + grace, and the inert canceller blinds every deadline
/// observation the worker could bail on, so the detach is certain
/// under any scheduling.
fn parked_contract(tag: &str, deadline: Duration) {
    let _turn = GAUGE.lock().unwrap_or_else(|e| e.into_inner());
    let p = probe(
        tag,
        GRACE + Duration::from_secs(2),
        deadline,
        Canceller::inert(),
    );
    assert!(
        p.elapsed >= deadline + GRACE,
        "the whole grace was spent: {:?} — {}",
        p.elapsed,
        p.err
    );
    assert!(
        p.err.contains("still reading") && p.err.contains("detached"),
        "the residue is named with its stage: {}",
        p.err
    );
    assert_eq!(
        PARKED.load(SeqCst),
        p.parked + 1,
        "the parked worker is counted"
    );
    p.held.join().expect("stub");
    settles(p.parked, Duration::from_secs(3));
}

/// The counterfactual that proves the cancel is what tore the
/// worker down: with an INERT canceller (the pre-O64 behaviour) the
/// same silent daemon leaves the worker parked past the whole grace,
/// the refusal says so by name, and the gauge counts it — until the
/// daemon drops the connection and the worker returns, at which
/// point the gauge goes back down.
#[test]
fn an_inert_canceller_leaves_the_worker_parked_and_counted() {
    parked_contract("client-inert", DEADLINE);
}

/// The race the 2026-08-31 CI red exposed, pinned at its extreme: a
/// ZERO deadline always fires before the worker can register (on
/// that runner a ~1s scheduling stall did the same to the 300ms
/// leg — elapsed 1.306s, generic refusal, no residue). An inert
/// canceller that still let `register` refuse had the worker come
/// straight back and the grace end early: the counterfactual never
/// armed. Now the inert canceller blinds that refusal too, so the
/// worker parks on its read and the whole grace is spent even when
/// the deadline wins every race.
#[test]
fn an_inert_canceller_arms_even_when_the_deadline_wins_the_race() {
    parked_contract("client-inert-race", Duration::ZERO);
}
