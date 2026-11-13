//! A wedged core is reaped, not waited on; a working one is not. `read_line`
//! on the core's stdout was unbounded, so a core that stopped answering
//! held the daemon — and every hook queued behind its serial accept loop —
//! forever (2026-08-19 review, finding 20). The 60 s wall deadline that
//! fixed it then killed working cores on a loaded machine and never
//! watched the request's write (plan v2.35 phase D): the link now kills
//! a core that makes no progress for S (CE_CORE_DEADLINE_SECS) or spends
//! more than B CPU-seconds (CE_CORE_CPU_BUDGET_SECS) on one exchange
//! (corelink/watch.rs).
//!
//! Alone in its binary: the two variables are process-global, and
//! integration test files each get their own process. The legs set them
//! under one lock, so the harness's threads never race on the environment.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

static ENV: Mutex<()> = Mutex::new(());

/// Set the rule's two variables for one leg (S, then B: None = shipped).
fn rule(stall: &str, budget: Option<&str>) -> std::sync::MutexGuard<'static, ()> {
    let guard = ENV.lock().unwrap_or_else(PoisonError::into_inner);
    // SAFETY: every leg of this binary touches the environment only while
    // it holds ENV, so no thread reads or writes it concurrently.
    unsafe {
        std::env::set_var("CE_CORE_DEADLINE_SECS", stall);
        match budget {
            Some(b) => std::env::set_var("CE_CORE_CPU_BUDGET_SECS", b),
            None => std::env::remove_var("CE_CORE_CPU_BUDGET_SECS"),
        }
    }
    guard
}

/// `sort` is the canonical read-everything-answer-at-EOF program on
/// both Windows (System32) and Unix: it swallows the hello line and
/// answers nothing, which is exactly a wedge.
#[test]
fn a_core_that_never_answers_is_reaped_within_the_deadline() {
    let _env = rule("1", None);
    let t0 = std::time::Instant::now();
    let err = codeeraser::corelink::Link::open("sort")
        .map(|_| ())
        .expect_err("a silent core must not hand back a live link");
    let waited = t0.elapsed();
    assert!(err.contains("deadline"), "names the deadline: {err}");
    // plan v2.35 C5: what it waited on, and the variable that sets the wait
    assert!(
        err.contains("waiting on hello") && err.contains("CE_CORE_DEADLINE_SECS"),
        "names the request and the knob: {err}"
    );
    assert!(
        waited < std::time::Duration::from_secs(30),
        "reaped in bounded time, not a hung read: {waited:?}"
    );
}

/// What a stub does between reading the hello and answering it.
enum Act {
    /// Burn CPU past S, then answer: a core that works longer than S.
    Burn,
    /// Spin without ever answering: a core past any budget.
    Spin,
    /// Answer the hello without reading it, then sit without reading or
    /// computing: a core that stopped draining.
    Idle,
}

/// The seconds the `Burn` stub works by its own wall clock, on both arms:
/// three times the leg's S and one more. Clocked, not counted, so no
/// machine's speed is sized into the leg: bash counts `SECONDS`, cmd
/// counts the changes of `%time%`'s seconds field in a `goto` loop (one
/// change may come early, so it works between BURN_SECS − 1 and BURN_SECS
/// seconds, either arm).
const BURN_SECS: u64 = 10;

/// A stub's directory, removed when its leg ends, however it ends.
struct Stub {
    dir: PathBuf,
    core: String,
}

impl Drop for Stub {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// A stub core in its own directory (named by leg and process, so no
/// earlier run's is in the way): the script and the hello it answers,
/// copied out of a file and never echoed, the JSON holding characters cmd
/// would read as syntax.
fn stub(leg: &str, act: Act) -> Stub {
    let name = format!("ce-corelink-{leg}-{}", std::process::id());
    let mut stub = Stub {
        dir: std::env::temp_dir().join(name),
        core: String::new(),
    };
    std::fs::create_dir_all(&stub.dir).expect("the stub's directory");
    let hello = serde_json::json!({
        "proto": codeeraser::corelink::PROTO, "type": "hello", "server": "ce-core",
        "version": "stub", "accept": true, "capabilities": ["hello"],
    });
    let reply = stub.dir.join("hello.json");
    std::fs::write(&reply, format!("{hello}\n")).expect("the hello reply");
    let path = stub
        .dir
        .join(if cfg!(windows) { "core.cmd" } else { "core.sh" });
    executable(&path, &script(&reply, &act));
    stub.core = path.to_string_lossy().into_owned();
    stub
}

/// Write `text` as a file the link can run (mode 0755 on unix).
fn executable(path: &Path, text: &str) {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o755);
    let mut file = options.open(path).expect("the script file");
    file.write_all(text.as_bytes()).expect("the stub core");
}

/// The script, builtins only, so the stub is the one process the link
/// spawned: a grandchild would hold the request pipe open past the kill.
/// Each shell's pieces first (Windows: cmd, whose console CON the link
/// gives it hidden by CREATE_NO_WINDOW, a read that never returns and
/// costs nothing; unix: bash), then one assembly per act.
fn script(reply: &Path, act: &Act) -> String {
    let reply = reply.display();
    let (head, read, burn, spin, idle, answer) = if cfg!(windows) {
        let tick = "%time:~6,2%";
        (
            "@echo off\r\n",
            "set /p line=\r\n",
            format!(
                "set n=0\r\nset last={tick}\r\n:burn\r\nif \"{tick}\"==\"%last%\" goto burn\r\n\
                 set last={tick}\r\nset /a n+=1\r\nif %n% lss {BURN_SECS} goto burn\r\n"
            ),
            "for /L %%i in (0,0,1) do set x=%%i\r\n",
            "set /p line=<CON\r\n",
            format!("type \"{reply}\"\r\n"),
        )
    } else {
        let lines = format!("while IFS= read -r l; do printf '%s\\n' \"$l\"; done < '{reply}'\n");
        let until = format!("end=$((SECONDS + {BURN_SECS}))\n");
        (
            "#!/bin/bash\n",
            "read line\n",
            until + "while [ $SECONDS -lt $end ]; do :; done\n",
            "while :; do :; done\n",
            "exec sleep 600\n",
            lines,
        )
    };
    let body = match act {
        Act::Burn => format!("{read}{burn}{answer}"),
        Act::Spin => format!("{read}{spin}"),
        Act::Idle => format!("{answer}{idle}"),
    };
    format!("{head}{body}")
}

/// Run `f` on its own thread and give it `bound`: a link that hangs fails
/// the leg instead of holding the test binary.
fn within<T: Send + 'static>(bound: Duration, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(bound)
        .unwrap_or_else(|_| panic!("no answer within {bound:?}: the link hung"))
}

/// Open a link on the stub on its own thread, within `bound`.
fn open(stub: &Stub, bound: Duration) -> Result<(), String> {
    let core = stub.core.clone();
    within(bound, move || {
        codeeraser::corelink::Link::open(&core).map(|_| ())
    })
}

/// Leg 1: a core that computes past S before it answers is waited for,
/// because its CPU clock moves (the wall deadline killed it at S).
#[test]
fn a_core_that_works_past_s_is_waited_for() {
    let _env = rule("3", None);
    let stub = stub("burn", Act::Burn);
    let t0 = Instant::now();
    let got = open(&stub, Duration::from_secs(300));
    let waited = t0.elapsed();
    eprintln!("burn leg: answered after {waited:?}");
    got.expect("a working core's reply is accepted");
    assert!(
        waited > Duration::from_secs(3),
        "the stub worked past S: {waited:?}"
    );
}

/// Leg 2: a core that answers the hello but never drains the request, a
/// line larger than any pipe buffer, is killed at S and the error names
/// the write (the old link blocked in the write for good).
#[test]
fn a_core_that_stops_draining_is_killed_naming_the_write() {
    let _env = rule("1", None);
    let stub = stub("idle", Act::Idle);
    let core = stub.core.clone();
    let err = within(Duration::from_secs(60), move || {
        let (mut link, _) = codeeraser::corelink::Link::open(&core)?;
        let body = serde_json::json!({ "pad": "x".repeat(4 << 20) });
        link.request("probe", body).map(|_| ())
    })
    .expect_err("an undrained request must not hand back a reply");
    eprintln!("idle leg: {err}");
    assert!(err.contains("no progress for 1s"), "names the stall: {err}");
    assert!(
        err.contains("waiting on probe.request"),
        "names the request: {err}"
    );
    let written = err.split(" of ").next().and_then(|h| h.rsplit(' ').next());
    let written: u64 = written.and_then(|n| n.parse().ok()).expect("a byte count");
    assert!(
        written < 4 << 20,
        "the write stopped short of the line: {err}"
    );
    assert!(
        !err.contains("unreadable"),
        "the CPU clock and the page-fault count were both read: {err}"
    );
}

/// Leg 3: a core that spins without answering is killed by the budget, and
/// the error names it.
#[test]
fn a_core_that_spins_is_killed_by_the_budget() {
    let _env = rule("3", Some("1"));
    let stub = stub("spin", Act::Spin);
    let err = open(&stub, Duration::from_secs(120))
        .expect_err("a spinning core must not hand back a live link");
    eprintln!("spin leg: {err}");
    assert!(
        err.contains("CPU budget exceeded (1s)"),
        "names the budget: {err}"
    );
    assert!(
        err.contains("CE_CORE_CPU_BUDGET_SECS"),
        "names the variable: {err}"
    );
}
