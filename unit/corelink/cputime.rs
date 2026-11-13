//! The physical check of the core's CPU clock (corelink/cputime.rs). The
//! child burns on a thread it spawns until its OWN process clock — a second
//! reader, independent of the one under test (unix
//! `clock_gettime(CLOCK_PROCESS_CPUTIME_ID)`, Windows `GetProcessTimes` on
//! itself) — has advanced one second; the parent's reading of that burn
//! must lie in [0.8 s, 1.5 s], and the child blocked on a read must not
//! move. A wrong unit (ticks read as seconds, mach units read as
//! nanoseconds) lands outside the band by orders of magnitude, and a
//! reader that sees only one thread reads about zero. No count is sized on
//! any machine. The child is this test binary itself, run with one ignored
//! leg selected, so no interpreter and no grandchild stands in for it. The
//! same child answers the page-fault reader: a live process reads a count
//! (above zero where the count is every fault), and on Windows a child
//! whose working set was emptied faults back in when it is woken.
#![cfg(any(windows, target_os = "linux", target_os = "macos"))]

use super::{cpu_time, faults_between, page_faults};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::time::Duration;

/// Set in the child's environment only: the ignored leg below then acts
/// as the child; anywhere else it returns at once.
const CHILD: &str = "CE_CPUTIME_CHILD";

/// The child's own process CPU clock, read by the platform's process clock.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn own_cpu() -> Duration {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: the out-pointer is a local.
    let rc = unsafe { libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut ts) };
    assert_eq!(rc, 0, "clock_gettime(CLOCK_PROCESS_CPUTIME_ID)");
    let secs = u64::try_from(ts.tv_sec).expect("a non-negative clock");
    let nanos = u32::try_from(ts.tv_nsec).expect("nanoseconds under a second");
    Duration::new(secs, nanos)
}

/// The child's own process CPU clock, asked of the system about itself.
#[cfg(windows)]
fn own_cpu() -> Duration {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};
    let mut t = [FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    }; 4];
    let [created, exited, kernel, user] = &mut t;
    // SAFETY: the pseudo-handle names this process; the out-pointers are locals.
    let ok = unsafe { GetProcessTimes(GetCurrentProcess(), created, exited, kernel, user) };
    assert_ne!(ok, 0, "GetProcessTimes on this process");
    let units: u64 = t[2..]
        .iter()
        .map(|f| (u64::from(f.dwHighDateTime) << 32) | u64::from(f.dwLowDateTime))
        .sum();
    Duration::from_nanos(units * 100)
}

/// Burn on a spawned thread until this process's own clock has advanced
/// one second, reading the clock between chunks of work.
fn burn_one_second() {
    std::thread::spawn(|| {
        let start = own_cpu();
        let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
        while own_cpu().saturating_sub(start) < Duration::from_secs(1) {
            for i in 0..100_000u64 {
                x = std::hint::black_box(x.rotate_left(5) ^ i).wrapping_mul(0x0100_0000_01B3);
            }
        }
        std::hint::black_box(x);
    })
    .join()
    .expect("the burning thread");
}

/// The child's side: say `ready`, then for each `burn` line burn and say
/// `done`, for each `wake` line say `woke`; end at EOF. Between lines it is
/// blocked on its stdin.
#[test]
#[ignore = "the child the cputime leg spawns, not a leg of its own"]
fn cpu_child() {
    if std::env::var_os(CHILD).is_none() {
        return;
    }
    println!("ready");
    for line in std::io::stdin().lock().lines() {
        match line.expect("the parent's line").as_str() {
            "burn" => {
                burn_one_second();
                println!("done");
            }
            "wake" => println!("woke"),
            _ => {}
        }
    }
}

/// Read the child's stdout up to a line ending in `word`: the harness
/// prints `test <name> ... ` without a newline, so `ready` ends its line.
fn wait_for(out: &mut BufReader<ChildStdout>, word: &str) {
    let mut line = String::new();
    loop {
        line.clear();
        let n = out.read_line(&mut line).expect("the child's stdout");
        assert!(n > 0, "the child ended before it said {word}");
        if line.trim_end().ends_with(word) {
            return;
        }
    }
}

/// The child, spawned and past its `ready`.
fn ready_child() -> (Child, BufReader<ChildStdout>) {
    let exe = std::env::current_exe().expect("this test binary");
    let mut child = Command::new(exe)
        .args(["--exact", "corelink::cputime::tests::cpu_child"])
        .args(["--ignored", "--nocapture", "--test-threads=1"])
        .env(CHILD, "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn the child");
    let mut out = BufReader::new(child.stdout.take().expect("stdout"));
    wait_for(&mut out, "ready");
    (child, out)
}

#[test]
fn a_burning_child_reads_its_cpu_and_a_blocked_one_does_not_move() {
    let (mut child, mut out) = ready_child();
    let rest = cpu_time(&child).expect("the clock reads on this platform");
    // the window the blocked child is watched over, not a wait on a signal
    std::thread::sleep(Duration::from_secs(1));
    let still = cpu_time(&child).expect("a second reading");
    let idle = still.saturating_sub(rest);
    assert!(
        idle < Duration::from_millis(50),
        "a blocked child moved {idle:?}"
    );
    let mut stdin = child.stdin.take().expect("stdin");
    writeln!(stdin, "burn").expect("ask the burn");
    wait_for(&mut out, "done");
    let burnt = cpu_time(&child)
        .expect("a third reading")
        .saturating_sub(still);
    drop(stdin);
    let _ = child.wait();
    eprintln!("cputime: the burn read {burnt:?}");
    assert!(
        (Duration::from_millis(800)..=Duration::from_millis(1500)).contains(&burnt),
        "one CPU-second by the child's own clock read {burnt:?}"
    );
}

/// The reader answers for a live child; where it counts every fault
/// (Windows, Linux), a process that has started has taken some.
#[test]
fn a_live_child_reads_its_page_faults() {
    let (mut child, _out) = ready_child();
    let n = page_faults(&child);
    drop(child.stdin.take());
    let _ = child.wait();
    let n = n.expect("the count reads on this platform");
    eprintln!("cputime: a ready child read {n} {}", super::FAULTS);
    if cfg!(any(windows, target_os = "linux")) {
        assert!(n > 0, "a started process read no page fault");
    }
}

/// The difference wraps at the reader's width: Windows' count is a u32.
#[test]
fn the_count_wraps_at_its_width() {
    assert_eq!(faults_between(5, 1537), 1532);
    if cfg!(windows) {
        assert_eq!(faults_between(u64::from(u32::MAX) - 1, 3), 5);
    }
}

/// Windows, physical: the waiting child's working set emptied, then woken
/// with one line, reads faults it did not have before - the count moves
/// with the pages the process touches again, not with time.
#[cfg(windows)]
#[test]
fn an_emptied_child_faults_back_in_when_woken() {
    use super::faults_since;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::ProcessStatus::EmptyWorkingSet;
    let (mut child, mut out) = ready_child();
    // SAFETY: the child's own process handle, open while `child` lives.
    let emptied = unsafe { EmptyWorkingSet(child.as_raw_handle()) };
    assert_ne!(emptied, 0, "EmptyWorkingSet on the child");
    let before = page_faults(&child);
    let mut stdin = child.stdin.take().expect("stdin");
    writeln!(stdin, "wake").expect("wake the child");
    wait_for(&mut out, "woke");
    let since = faults_since(&child, before);
    drop(stdin);
    let _ = child.wait();
    eprintln!("cputime: woken after its working set was emptied, {since:?} page faults");
    assert!(
        since.expect("both readings") > 0,
        "a woken emptied child took no fault"
    );
}
