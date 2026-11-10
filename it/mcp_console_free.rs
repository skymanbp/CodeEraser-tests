//! `ce mcp` is this binary but no console face (v2.35 G9, R3-36): it
//! used to arm progress like every console command, so a forced
//! `CE_PROGRESS=1` painted the churn walk's frames over the protocol's
//! stderr, and the project's `[ui] lang` pinned a machine face. The
//! server now stays inert the way the GUI and the daemon do.

use crate::common;
use serde_json::{Value, json};
use std::io::Write as _;
use std::process::Stdio;

#[test]
fn the_mcp_server_paints_no_progress() {
    let dir = common::tmp("mcp-console-free");
    // a language pin a console face would obey
    common::declare(&dir, "[ui]\nlang = \"zh\"\n");
    common::write_all(&dir, &[("a.py", "def f(a):\n    return a\n")]);
    common::init_and_commit(&dir, "one");
    let calls = [
        ("churn", json!({})),
        ("scan", json!({"path": "../elsewhere"})),
    ];
    let (replies, stderr) = converse(&dir, &calls);
    assert!(
        stderr.trim().is_empty(),
        "no progress frame on a protocol's stderr: {stderr:?}"
    );
    assert_eq!(replies.len(), 3, "{replies:?}");
    assert_eq!(replies[1]["result"]["isError"], false, "{}", replies[1]);
    // and a path outside the served root is a named refusal (G6, R3-40)
    let refusal = &replies[2]["result"];
    assert_eq!(refusal["isError"], true, "{refusal}");
    let text = refusal["content"][0]["text"].as_str().expect("text");
    assert!(text.contains("outside the served root"), "{text}");
    std::fs::remove_dir_all(&dir).ok();
}

/// The whole conversation piped at once: (stdout frames, stderr text).
fn converse(dir: &std::path::Path, calls: &[(&str, Value)]) -> (Vec<Value>, String) {
    let mut frames = vec![json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {}})];
    for (i, (name, args)) in calls.iter().enumerate() {
        let params = json!({"name": name, "arguments": args});
        frames
            .push(json!({"jsonrpc": "2.0", "id": i + 1, "method": "tools/call", "params": params}));
    }
    let mut child = common::ce_command(dir, &[("CE_PROGRESS", "1")])
        .args(["mcp", "."])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mcp");
    let mut stdin = child.stdin.take().expect("stdin");
    frames
        .iter()
        .for_each(|f| writeln!(stdin, "{f}").expect("frame"));
    drop(stdin); // EOF ends serve()
    let out = child.wait_with_output().expect("wait");
    let replies = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| serde_json::from_str(l).expect("a JSON frame"))
        .collect();
    (replies, String::from_utf8_lossy(&out.stderr).into_owned())
}
