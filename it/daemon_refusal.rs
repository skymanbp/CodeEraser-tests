//! A core's refusal is not a dead link (plan v2.33 step 11b item 1).
//! The daemon holds its own core across hooks behind a retry budget
//! that backs off 1 s·2^(n−1) after every failure that LOST the link;
//! a body the core refuses by contract (or over a cap) is answered, the
//! core stateless per request and its session whole, so the refusal is
//! the hook's named degraded reply and the next hook is judged at once
//! — no backoff, no "next try in" line on the daemon's stderr. Before
//! the fix one refused body dropped a healthy core and the next hook
//! inside the backoff window read `core_unavailable`.

use crate::common;
use codeeraser::daemon::client;
use codeeraser::daemon::proto::{Request, Response};
use serde_json::json;

#[test]
fn a_refused_request_keeps_the_daemons_core_for_the_next_hook() {
    let root = common::tmp("daemon-refusal");
    let (child, heard) = common::spawn_daemon_ready_heard(&root);
    let ask = |request: &Request| match client::request_if_running(&root, request) {
        Ok(Response::DocumentReport { reply } | Response::TombstoneReport { reply }) => reply,
        other => panic!("expected a report, got {other:?}"),
    };
    let refused = ask(&Request::Document {
        body: json!({"family": "no-such-family"}),
    });
    let why = refused["reason"].as_str().unwrap_or_default();
    assert!(
        refused["degraded"] == true && why.starts_with("core refused document.request: "),
        "the core's refusal, named: {refused}"
    );
    let judged = ask(&Request::Tombstone {
        rows: vec![[0, 1, 1]],
        budget: None,
    });
    assert!(
        judged.get("degraded").is_none_or(|d| d != true),
        "the next hook is judged by the same core: {judged}"
    );
    common::shutdown_and_wait(&root, child, "daemon after refusal test");
    let said = heard.join().expect("the stderr reader");
    assert!(!said.contains("next try in"), "no backoff: {said}");
}
