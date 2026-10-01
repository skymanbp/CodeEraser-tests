//! A hook that cannot read the core's definition package (plan v2.32
//! step 2) stays inert — exit 0 (run_hook asserts it), nothing on
//! stdout, so never a deny — and never silently: one observe-feed line
//! under the hook's own event with `degraded` true and `reason` the
//! CLI's refusal (hookio/inert.rs), and the SessionStart line names
//! the unavailable tables once per session.

use crate::common;
use serde_json::json;

/// The core the run is pointed at: a path that is no file.
const NO_CORE: &str = "no-such-ce-core";

/// The feed's newest line must be the inert trace of `event`.
fn traced(dir: &std::path::Path, event: &str) {
    let line = common::last_observe(dir);
    assert_eq!(line["event"], event, "{line}");
    assert_eq!(line["degraded"], json!(true), "{line}");
    assert_eq!(line["schema"], codeeraser::hookio::OBSERVE_SCHEMA);
    assert_eq!(line["session_id"], "t", "{line}");
    let why = line["reason"].as_str().unwrap_or_default();
    assert!(why.starts_with("core unavailable"), "{line}");
}

#[test]
fn write_and_stop_hooks_without_the_package_leave_a_feed_line() {
    let dir = common::tmp("hooks-inert");
    let core = dir.join(NO_CORE).display().to_string();
    let env = [("CE_CORE_BIN", core.as_str())];
    let pre = common::pretooluse_envelope(&dir, "Write", "fn a() {}\n");
    let post = common::posttooluse_envelope(&dir, "b.rs", "Write", "t");
    let stop = common::stop_envelope(&dir, false);
    let legs = [
        ("probe", pre, "probe"),
        ("settle", post, "settle"),
        ("audit", stop, "stop_audit"),
    ];
    for (cmd, envelope, event) in legs {
        let out = common::run_hook_env(&dir, &[cmd, "--hook"], &envelope, &env);
        assert_eq!(out.trim(), "", "{cmd}: an inert hook decides nothing");
        traced(&dir, event);
    }
    let file = common::observe_lines(&dir)[0]["file"].clone();
    assert!(file.as_str().is_some_and(|f| f.ends_with("b.rs")), "{file}");
}

#[test]
fn the_session_line_names_the_unavailable_tables() {
    let dir = common::tmp("health-inert");
    let core = dir.join(NO_CORE).display().to_string();
    let (_, ctx) = common::session_start_line_env(&dir, &[("CE_CORE_BIN", &core)]);
    assert!(
        ctx.contains("tables: unavailable — core unavailable"),
        "{ctx}"
    );
    traced(&dir, "health");
}
