//! The guard's input contract has an executor: the three PreToolUse
//! envelopes captured from a real Claude Code session
//! (contracts/fixtures/hook-payloads/, frozen 2026-08-07) are fed,
//! placeholders filled, to `ce probe --hook`. Every other guard leg
//! builds its envelope by hand (common/hooks.rs); this one reads the
//! recorded bytes, so a parser change that broke the real shape, or an
//! edit to the goldens, turns it red. In observe mode the probe stays
//! silent and writes its feed line, and that line carries the golden's
//! own `tool_use_id`: the envelope was read, not refused as malformed
//! (a malformed envelope fails open with no line at all).

use crate::common;

const GOLDENS: [&str; 3] = [
    "edit.golden.json",
    "edit-replace-all.golden.json",
    "write.golden.json",
];

#[test]
fn the_captured_envelopes_are_read_by_the_probe() {
    for name in GOLDENS {
        let dir = common::tmp(&format!("hook-payload-{}", name.replace('.', "-")));
        common::seed_project(&dir, "observe");
        std::fs::create_dir_all(dir.join("src")).expect("src");
        std::fs::write(dir.join("src/example.txt"), "probe line one\n").expect("example.txt");
        let golden = common::repo_root()
            .join("contracts/fixtures/hook-payloads")
            .join(name);
        let text = std::fs::read_to_string(&golden).unwrap_or_else(|e| panic!("{name}: {e}"));
        let at = dir.display().to_string().replace('\\', "/");
        let envelope = text.replace("<PROJECT_DIR>", &at).replace("<HOME>", &at);
        let id = serde_json::from_str::<serde_json::Value>(&envelope).expect("envelope json")
            ["tool_use_id"]
            .clone();
        assert!(
            id.is_string(),
            "{name}: the capture carries its tool_use_id"
        );
        let line = common::silent_hook_observe(&dir, &["probe", "--hook"], &envelope, "probe");
        assert_eq!(
            line["tool_use_id"], id,
            "{name}: the probe read the envelope"
        );
        assert_eq!(line["degraded"], false, "{name}: a healthy probe");
    }
}
