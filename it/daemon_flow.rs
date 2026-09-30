//! The daemon's `flow` request (daemon proto 2.2.0, plan v2.31 step
//! 5): one side's four tables — exactly as `flow::wire::body`
//! assembles them — forwarded over the daemon-owned core link, and the
//! raw flow.result back, consumed against what was sent. The guard leg
//! walks this road on every write (flow_guard.rs); this is the road
//! alone, and its degraded posture without a core.

use crate::common;
use codeeraser::daemon::client;
use codeeraser::daemon::proto::{FlowTables, Request, Response};
use codeeraser::flow::lower::lower_file;
use codeeraser::flow::wire;
use codeeraser::scan::lang::Lang;

const PY: &str = "def gone():\n    return 1\n    print(\"never\")\n";

#[test]
fn a_flow_request_round_trips_through_the_daemon() {
    let root = common::tmp("daemon-flow");
    let child = common::spawn_daemon_ready(&root);
    let file = lower_file(PY, Lang::Python).expect("lowers");
    let files = std::slice::from_ref(&file);
    let (batches, unsent) = wire::plan(files);
    assert_eq!((batches.len(), unsent.len()), (1, 0));
    let sent = &batches[0];
    let tables: FlowTables = serde_json::from_value(wire::body(sent)).expect("the body's shape");
    let reply = match client::request_if_running(&root, &Request::Flow(tables)).expect("flow") {
        Response::FlowReport { reply } => reply,
        other => panic!("expected a flow report, got {other:?}"),
    };
    let judged = wire::consume(&reply, sent).expect("a healthy reply");
    let kinds: Vec<u8> = judged.findings.iter().map(|f| f.kind).collect();
    assert_eq!(kinds, [0], "one unreachable run: {reply}");
    common::shutdown_and_wait(&root, child, "daemon after flow test");
}
