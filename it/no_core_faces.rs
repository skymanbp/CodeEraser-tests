//! The surfaces that must keep answering when no core can be reached
//! (v2.35 G3, R3-2 / R3-3 / R3-6). The MCP server used to die on the
//! first measuring call — `tables::get()` exits 2 — so the client saw
//! a disconnect and its next `doctor` call went unanswered; `ce eject`,
//! the uninstall, refused to run at all without a core, dry run
//! included; `ce doctor` judged the handshake only, so a core that
//! shook hands and could not answer `tables/1` read healthy.

use crate::common;
use crate::common::McpSession;
use serde_json::{Value, json};
use std::path::Path;

/// `CE_CORE_BIN` naming nothing, inside the leg's own directory.
fn no_core(dir: &Path) -> String {
    dir.join("no-such-core/ce-core").display().to_string()
}

/// A JSON document carried as a tool reply's text.
fn text_json(reply: &Value) -> Value {
    serde_json::from_str(reply["content"][0]["text"].as_str().expect("text")).expect("json")
}

#[test]
fn mcp_names_the_missing_package_and_keeps_serving() {
    let dir = common::tmp("nocore-mcp");
    common::seed_clone_pair(&dir);
    let core = no_core(&dir);
    let mut s = McpSession::over_env(&dir, &[("CE_CORE_BIN", &core)]);
    assert_eq!(s.initialize(), "codeeraser");
    let scan = s.call(2, "scan", json!({}));
    let said = scan["content"][0]["text"].as_str().unwrap_or_default();
    assert!(
        scan["isError"] == true && said.contains("core unavailable"),
        "a measuring tool refuses by name, as a result: {scan}"
    );
    // the server is still there, and the machine tool answers
    let doctor = s.call(3, "doctor", json!({}));
    assert_eq!(
        doctor["isError"], false,
        "doctor answers without a core: {doctor}"
    );
    assert_eq!(
        text_json(&doctor)["tables"]["ok"],
        false,
        "the absence is the finding"
    );
    s.finish();
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn eject_runs_without_a_core() {
    let dir = common::tmp("nocore-eject");
    for d in [".ce", "pkg/.ce"] {
        std::fs::create_dir_all(dir.join(d)).expect(d);
    }
    common::declare(&dir, "\n");
    let core = no_core(&dir);
    let env = [("CE_CORE_BIN", core.as_str())];

    let dry = common::run_ce_env(&dir, &["eject"], &env);
    let said = String::from_utf8_lossy(&dry.stdout).replace('\\', "/");
    assert!(dry.status.success(), "the dry run answers: {dry:?}");
    assert!(said.contains("pkg/.ce"), "the stray is listed: {said}");
    assert!(dir.join("pkg/.ce").exists(), "a dry run removes nothing");

    let yes = common::run_ce_env(&dir, &["eject", "--yes"], &env);
    assert!(yes.status.success(), "the uninstall runs: {yes:?}");
    assert!(!dir.join(".ce").exists() && !dir.join("pkg/.ce").exists());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn doctor_folds_the_definition_package_into_its_verdict() {
    let dir = common::tmp("nocore-doctor");
    let json_args = ["doctor", "--format", "json"];
    let healthy = common::run_ce(&dir, &json_args);
    let d: Value = serde_json::from_slice(&healthy.stdout).expect("json");
    assert_eq!(d["tables"], json!({"ok": true, "reason": null}));
    assert!(
        healthy.status.success(),
        "a readable package and a handshake: exit 0"
    );

    let core = no_core(&dir);
    let env = [("CE_CORE_BIN", core.as_str())];
    let console = common::run_ce_env(&dir, &["doctor"], &env);
    assert_eq!(console.status.code(), Some(2), "{console:?}");
    let out = common::run_ce_env(&dir, &json_args, &env);
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    let tables = serde_json::from_slice::<Value>(&out.stdout).expect("json")["tables"].clone();
    let reason = tables["reason"].as_str().unwrap_or_default();
    assert!(
        tables["ok"] == false && reason.contains("core unavailable"),
        "the package's absence is named: {tables}"
    );
    assert!(!dir.join(".ce").exists(), "the diagnostic created no state");
    std::fs::remove_dir_all(&dir).ok();
}
