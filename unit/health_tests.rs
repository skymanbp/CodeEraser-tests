//! health.rs unit battery, split out per the E01 file cap (the
//! store.rs / store_tests.rs precedent): the observe feed's degraded
//! counter, whose contract is that it sees a producer wherever that
//! producer stamps itself.

use super::line_degraded;
use serde_json::json;

/// The nested case is the whole point: a Stop audit whose dedup leg
/// was fine but whose ce-core was missing carries `degraded: false`
/// at the top level and its reason one level down, and the old counter
/// read only the top. The shapes are the ones the writers produce
/// (v2.35 G4, R3-1): a top-level `true`, a nested REASON string
/// (audit.rs `daemon_unavailable` / `no_git`, the flow and similar legs,
/// the core's own refusal), and the tombstone leg's two-level
/// `tombstone.judged.degraded`. Null and `false` are healthy, and an
/// unnamed producer two levels down is still not one.
#[test]
fn a_nested_producer_counts_and_a_healthy_line_does_not() {
    assert!(line_degraded(
        &json!({"event": "probe", "degraded": true, "reason": "x"})
    ));
    assert!(line_degraded(&json!({
        "event": "stop_audit", "degraded": false,
        "fourclass": {"pairs": 3, "degraded": "daemon_unavailable"}
    })));
    assert!(line_degraded(&json!({
        "event": "stop_audit", "degraded": false,
        "flow": {"degraded": "NOT NULL constraint failed: x"}
    })));
    assert!(line_degraded(&json!({
        "event": "probe", "degraded": false,
        "tombstone": {"rows": 1, "judged": {"degraded": "core_unavailable"}}
    })));
    assert!(!line_degraded(&json!({
        "event": "stop_audit", "degraded": false,
        "fourclass": {"pairs": 3, "degraded": null},
        "tombstone": {"judged": {"sites": [], "over": false}}
    })));
    assert!(!line_degraded(&json!({"a": {"b": {"degraded": true}}})));
}
