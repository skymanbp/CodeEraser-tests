//! The liveness rule over synthetic samples (corelink/watch.rs): a core
//! that works is never stalled, however slow the wall clock; a core that
//! moves neither signal for S is; a core past B CPU-seconds is over budget.

use super::{Faults, Sample, Verdict, Watch};
use std::time::Duration;

const S: Duration = Duration::from_secs(60);
const B: Duration = Duration::from_secs(600);

/// The page-fault count as the caller hands it over: read, and not.
const READ: Faults = Faults {
    noun: "page faults",
    count: Some(1532),
};
const UNREAD: Faults = Faults {
    noun: "page faults",
    count: None,
};

fn at(secs: u64, bytes: u64, cpu_ms: Option<u64>) -> Sample {
    Sample {
        at: Duration::from_secs(secs),
        bytes,
        cpu: cpu_ms.map(Duration::from_millis),
    }
}

/// The verdicts of a watch that starts at `first` and reads `then`.
fn run(first: Sample, then: &[Sample]) -> (Vec<Verdict>, Watch) {
    let mut w = Watch::new(S, B, first);
    let verdicts = then.iter().map(|s| w.step(*s)).collect();
    (verdicts, w)
}

/// Seventy samples of `step(k)` after `first`, every one of them Wait.
fn waits_throughout(first: Sample, step: impl Fn(u64) -> Sample) -> bool {
    let then: Vec<Sample> = (1..=70).map(step).collect();
    run(first, &then).0.iter().all(|v| *v == Verdict::Wait)
}

/// Progress of either kind waits past an hour of wall time (the wall rule
/// killed at 60 s): a starved core whose CPU clock moves 10 ms every 59 s,
/// and a core draining a large line while its CPU clock is flat.
#[test]
fn either_signal_moving_waits_at_any_wall_time() {
    let starved = waits_throughout(at(0, 100, Some(0)), |k| at(59 * k, 100, Some(10 * k)));
    assert!(starved, "a starved core that works was stalled");
    let draining = waits_throughout(at(0, 0, Some(0)), |k| at(50 * k, 65536 * k, Some(0)));
    assert!(draining, "a core draining the request was stalled");
}

/// Neither signal moves: Wait until S has passed since the last movement,
/// then Stalled.
#[test]
fn a_core_flat_for_s_is_stalled() {
    let flat = [
        at(10, 100, Some(700)),
        at(69, 100, Some(700)),
        at(70, 100, Some(700)),
    ];
    let (verdicts, _) = run(at(0, 0, Some(500)), &flat);
    assert_eq!(verdicts, [Verdict::Wait, Verdict::Wait, Verdict::Stalled]);
}

/// The exchange's spend is counted from its first reading, not from the
/// core's start: past B is over budget, at B it is not.
#[test]
fn cpu_past_b_on_one_exchange_is_over_budget() {
    let spends = [at(1, 10, Some(604_000)), at(2, 10, Some(604_001))];
    let (verdicts, w) = run(at(0, 10, Some(4_000)), &spends);
    assert_eq!(verdicts, [Verdict::Wait, Verdict::OverBudget]);
    assert_eq!(w.spent(), Some(Duration::from_millis(600_001)));
}

/// An unreadable clock: the bytes alone are progress, so once the line is
/// written S is a reply deadline again; a lost reading between two equal
/// ones is no movement.
#[test]
fn an_unreadable_clock_reads_the_bytes_alone() {
    let blind = [at(30, 900, None), at(89, 900, None), at(90, 900, None)];
    let (verdicts, w) = run(at(0, 0, None), &blind);
    assert_eq!(verdicts, [Verdict::Wait, Verdict::Wait, Verdict::Stalled]);
    assert_eq!(w.spent(), None);
    let (flicker, _) = run(
        at(0, 9, Some(300)),
        &[at(40, 9, None), at(60, 9, Some(300))],
    );
    assert_eq!(flicker, [Verdict::Wait, Verdict::Stalled]);
}

fn says(text: &str, parts: &[&str]) {
    for part in parts {
        assert!(text.contains(part), "{part:?} in {text}");
    }
}

/// Each killing verdict names its clause, its readings and the variable
/// that sets it; an unreadable clock is said by name.
#[test]
fn the_error_names_the_clause_the_readings_and_the_seam() {
    let (_, w) = run(at(0, 0, Some(0)), &[at(5, 1234, Some(5_450))]);
    let stalled = w.explain(Verdict::Stalled, 2000, "clone.request (1999 bytes)", READ);
    says(
        &stalled,
        &[
            "deadline exceeded",
            "no progress for 60s",
            "waiting on clone.request (1999 bytes)",
            "1234 of 2000 B written",
            "core CPU 5.45 s and 1532 page faults on it",
            "CE_CORE_DEADLINE_SECS",
        ],
    );
    let over = w.explain(
        Verdict::OverBudget,
        2000,
        "bags.request (1999 bytes)",
        UNREAD,
    );
    says(
        &over,
        &[
            "CPU budget exceeded (600s)",
            "core CPU 5.45 s on it, page faults unreadable",
            "CE_CORE_CPU_BUDGET_SECS",
        ],
    );
    let (_, unread) = run(at(0, 0, None), &[]);
    says(
        &unread.explain(Verdict::Stalled, 1, "hello", READ),
        &["core CPU unreadable"],
    );
}

/// The stall line whole, in its three states: both readings, the count
/// unreadable, the clock unreadable. A whole request written and a CPU
/// clock that never moved is the line that cannot tell a core waiting on
/// the disk from a wedged one; the count is what tells them apart.
#[test]
fn the_stall_line_in_its_three_states() {
    let (_, w) = run(at(0, 83_974, Some(0)), &[]);
    let (_, blind) = run(at(0, 9, None), &[]);
    let on = "clone.request (83973 bytes)";
    let lines = [
        w.explain(Verdict::Stalled, 83_974, on, READ),
        w.explain(Verdict::Stalled, 83_974, on, UNREAD),
        blind.explain(Verdict::Stalled, 10, "hello", READ),
    ];
    let head = "core reply deadline exceeded: no progress for 60s waiting on";
    let tail = "— core killed (CE_CORE_DEADLINE_SECS sets the wait)";
    assert_eq!(
        lines,
        [
            format!(
                "{head} {on}: 83974 of 83974 B written, core CPU 0.00 s and 1532 page faults on it {tail}"
            ),
            format!(
                "{head} {on}: 83974 of 83974 B written, core CPU 0.00 s on it, page faults unreadable {tail}"
            ),
            format!(
                "{head} hello: 9 of 10 B written, core CPU unreadable, so the bytes written were the only \
                 progress read; 1532 page faults on it {tail}"
            ),
        ]
    );
}
