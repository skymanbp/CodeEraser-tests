use super::*;

/// The retry budget (7.0.0, O63) driven without a spawn: open at
/// birth; a failure closes it for BACKOFF_BASE, the next for twice
/// that, the wait doubling until BACKOFF_CAP and never beyond; a
/// success reopens it at once and reports how many attempts it
/// recovered from — None when nothing was ever wrong, so a healthy
/// daemon never stamps `recovered` on a report.
#[test]
fn backoff_doubles_to_the_cap_and_recovery_reopens() {
    let t0 = Instant::now();
    let mut b = Budget::default();
    assert!(b.open(t0), "fresh: may try");
    assert_eq!(b.recovered(), None, "nothing to recover from");

    assert_eq!(b.failed(t0), BACKOFF_BASE);
    assert!(!b.open(t0 + BACKOFF_BASE / 2), "closed inside the wait");
    assert!(b.open(t0 + BACKOFF_BASE), "open once the wait ran out");
    assert_eq!(b.failed(t0), BACKOFF_BASE * 2, "second failure doubles");
    assert_eq!(b.failed(t0), BACKOFF_BASE * 4);

    let mut long = Budget::default();
    let mut last = Duration::ZERO;
    for _ in 0..80 {
        last = long.failed(t0);
    }
    assert_eq!(last, BACKOFF_CAP, "capped, and the exponent saturates");
    assert_eq!(Budget::backoff(1), BACKOFF_BASE);
    assert_eq!(Budget::backoff(6), BACKOFF_BASE * 32, "below the cap");
    assert_eq!(Budget::backoff(u32::MAX), BACKOFF_CAP);

    assert_eq!(b.recovered(), Some(3), "three attempts recovered from");
    assert!(b.open(t0), "reopened at once");
    assert_eq!(b.recovered(), None, "the count is reported once");
    assert_eq!(b.failed(t0), BACKOFF_BASE, "a fresh streak starts over");
}

/// v2.35 G3 (R3-5): a core that would not open says WHY on every
/// degraded reply until an open succeeds — a bare `core_unavailable`
/// hid the "present but refused" class behind the null stderr of a
/// lazily spawned daemon. The budget is closed first, so no spawn runs.
#[test]
fn a_failed_open_rides_the_degraded_reason() {
    let mut j = Judge::default();
    j.budget.failed(Instant::now());
    assert_eq!(
        j.document(serde_json::json!({}))["reason"],
        "core_unavailable"
    );
    j.open_error = Some("cannot start `x`".into());
    let reply = j.document(serde_json::json!({}));
    assert_eq!(reply["degraded"], true);
    assert_eq!(reply["reason"], "core_unavailable: cannot start `x`");
    let batch = j.judge_changeset(&[]);
    assert_eq!(batch.degraded.as_deref(), Some("no_link: cannot start `x`"));
}
