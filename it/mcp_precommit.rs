//! MCP server round-trip (self-consistency; real-client compliance is
//! a 0.x preview acceptance item) + pre-commit gate e2e on a real
//! git repo with staged duplication. The report-face comparison and
//! the knob relay sit in mcp_reports.rs (split, plan v2.33 step 11b).

use crate::common;
use crate::common::{McpSession, git, rust_fn, tmp};

/// Server over a seeded project (the harness lives in common/mcp.rs).
fn mcp_session(name: &str) -> McpSession {
    let dir = tmp(name);
    common::seed_clone_pair(&dir);
    McpSession::over(&dir)
}

#[test]
fn mcp_initialize_and_list() {
    let mut s = mcp_session("mcp-init");
    let init = s.ask(serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2024-11-05", "capabilities": {}}
    }));
    assert_eq!(init["result"]["serverInfo"]["name"], "codeeraser");
    let list = s.ask(serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}));
    let names: Vec<&str> = list["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .map(|t| t["name"].as_str().expect("name"))
        .collect();
    // M7-P2 ruling ③: the full read-only report face — and nothing
    // with a write verb (no baseline, no config, no establish; the
    // trend cache is index-cache bookkeeping, not a write action). The
    // served list is the TOOLS table, each name once (the parity gate
    // reads that table against every face; no order is promised).
    let served: std::collections::BTreeSet<String> =
        names.iter().map(|n| format!("mcp:{n}")).collect();
    assert_eq!(served.len(), names.len(), "a tool served twice: {names:?}");
    assert_eq!(served, crate::face_parity::mcp_tools());
    // The write verbs stay absent BY NAME, not by nobody having added
    // them: `erase` reaches the PLAN (read-only) and `apply` has no
    // face at all, so a machine surface cannot delete on its own
    // authority. This is the assertion that turns that into a rule.
    for forbidden in ["erase_apply", "apply", "baseline", "establish"] {
        assert!(
            !names.contains(&forbidden),
            "a write verb reached the read-only catalog: {forbidden}"
        );
    }
    s.finish();
}

#[test]
fn mcp_tool_calls_and_unknown_method() {
    let mut s = mcp_session("mcp-tools");
    let dup = s.ask(serde_json::json!({
        "jsonrpc": "2.0", "id": 3, "method": "tools/call",
        "params": {"name": "check_duplication", "arguments": {}}
    }));
    assert_eq!(dup["result"]["isError"], false);
    let text = dup["result"]["content"][0]["text"].as_str().expect("text");
    let report: serde_json::Value = serde_json::from_str(text).expect("report json");
    assert!(
        !report["blocks"].as_array().expect("blocks").is_empty(),
        "T2 pair must be reported over MCP"
    );
    let scan = s.ask(serde_json::json!({
        "jsonrpc": "2.0", "id": 4, "method": "tools/call",
        "params": {"name": "scan", "arguments": {}}
    }));
    let stext = scan["result"]["content"][0]["text"].as_str().expect("text");
    let sreport: serde_json::Value = serde_json::from_str(stext).expect("scan json");
    assert_eq!(
        sreport["schema"],
        crate::common::stub_core::catalogue_schema("scan")
    );
    // unknown method gets a JSON-RPC error, not a hang or crash
    let err = s.ask(serde_json::json!({"jsonrpc": "2.0", "id": 5, "method": "nope"}));
    assert_eq!(err["error"]["code"], -32601);
    s.finish();
}

/// A9f: a broken index degrades the precommit VISIBLY — exit 0 even
/// in deny (unverifiable state must not brick the commit), but the
/// human is told, never shown a clean pass.
#[test]
fn precommit_broken_index_is_visible_not_silent() {
    let dir = tmp("precommit-degraded");
    common::seed_git_clone_repo(&dir, "deny");
    common::corrupt_index(&dir);
    let out = common::run_ce(&dir, &["precommit"]);
    assert!(out.status.success(), "degraded fails open, never bricks");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("DEGRADED"),
        "degradation said out loud: {text}"
    );
    assert!(
        text.contains("staged file(s)"),
        "degraded line keeps the staged summary: {text}"
    );
    let line = common::last_observe(&dir);
    assert_eq!(line["event"], "precommit", "not mislabeled stop_audit");
    assert_eq!(line["degraded"], true);
}

#[test]
fn precommit_blocks_staged_duplication_in_deny() {
    let dir = tmp("precommit");
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    std::fs::write(dir.join("ce.toml"), "[guard]\nmode = \"deny\"\n").expect("ce.toml");
    git(&dir, &["init", "-q"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-qm", "seed"]);
    // clean staged change passes
    std::fs::write(dir.join("c.rs"), "fn fresh(n: u8) -> u8 { n / 2 }\n").expect("c.rs");
    git(&dir, &["add", "c.rs"]);
    let ok = common::run_ce(&dir, &["precommit"]);
    assert!(ok.status.success(), "clean staged change passes");
    // staged T2 clone blocks in deny mode
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs");
    git(&dir, &["add", "b.rs"]);
    let blocked = common::run_ce(&dir, &["precommit"]);
    assert!(!blocked.status.success(), "deny mode must block");
    let text = String::from_utf8_lossy(&blocked.stdout);
    assert!(text.contains("b.rs"), "report names the clone: {text}");
}
