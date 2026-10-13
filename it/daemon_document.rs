//! The daemon's `document` request (daemon proto 2.3.0, plan v2.32
//! step 5): a `guard` request body — the rules as integer rows, every
//! string by its reference class (plan v2.33 W7) — forwarded over the
//! daemon-owned core link, and the raw document.result back, its lines
//! spelled by the core. The guard walks this road on every decision
//! that speaks (guard_say.rs); this is the road alone, in both
//! languages.

use crate::common;
use codeeraser::daemon::client;
use codeeraser::daemon::proto::{Request, Response};
use codeeraser::document::lines::{Stream, spelled_lines};
use serde_json::json;

#[test]
fn a_guard_request_round_trips_through_the_daemon_in_both_languages() {
    let root = common::tmp("daemon-document");
    let child = common::spawn_daemon_ready(&root);
    let said = |lang: u8| {
        let body = json!({
            "family": "guard",
            "ranges": {"files": 1, "matches": 0, "places": 0, "units": 0, "errors": 1},
            "rows": {"say": [[1, 0, 760, 750, 1, 0, 0], [5, 0, 0, 0, 0, 0, 0]], "matches": [], "places": []},
            "facts": {}, "degraded": null, "lang": lang,
            "strings": {"file": ["src/目录/a.rs"], "match_file": [], "place_file": [], "unit": [],
                        "error": ["TOML parse error at line 1"]},
        });
        let reply = match client::request_if_running(&root, &Request::Document { body }) {
            Ok(Response::DocumentReport { reply }) => reply,
            other => panic!("expected a document, got {other:?}"),
        };
        let (lines, fail) = spelled_lines(&reply).expect("spelled");
        assert!(
            !fail && lines.iter().all(|l| l.stream == Stream::Out),
            "{reply}"
        );
        lines.into_iter().map(|l| l.text).collect::<Vec<_>>()
    };
    assert_eq!(
        said(0),
        [
            "ce: this write leaves src/目录/a.rs at 760 lines, past the hard budget of 750 (plan §4.1). \
             Split the file instead of growing it. (ce.toml drifted from the fenced baseline: judged \
             with the shipped budgets)",
            "(ce.toml unreadable, guard degraded to observe: TOML parse error at line 1)",
        ]
    );
    assert_eq!(
        said(1),
        [
            "ce：这次写入会让 src/目录/a.rs 达到 760 行，越过 750 行的硬预算（计划 §4.1）。\
             请拆分文件，而不是继续让它长大。 （ce.toml 已偏离围栏基线：改按出厂预算判决）",
            "（ce.toml 不可读，守卫已降级为 observe：TOML parse error at line 1）",
        ]
    );
    common::shutdown_and_wait(&root, child, "daemon after document test");
}
