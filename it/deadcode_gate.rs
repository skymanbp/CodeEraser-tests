//! The dead-file gate end to end (plan v2.35, the dead-file amendment,
//! phase 5B): real trees under a project root, a baseline written by the
//! real `ce baseline`, then `ce deadcode --check` and `ce check` through
//! the walk, the ladder and the core. One literal table: `== name` opens
//! a scenario in a fresh tree; then, in order, `toml <text>` declares
//! ce.toml whole (`;` a line break), `write <path>: <text>` writes a file,
//! `mv <from> <to>` renames one, `strip` takes `deadPoint`
//! off the committed baseline, `has <key>` / `lacks <key>` read the
//! committed baseline, and `run <cmd> <exit> <words>` runs one command
//! and holds its exit code and the words its output (stdout and stderr)
//! must carry — `deadcode` is `ce deadcode . --check`, `check` is
//! `ce check .`, `baseline` is `ce baseline .`, `baseline!` the same
//! under CE_ACCEPT_BASELINE=1.

use crate::common;
use crate::common::mcp::McpSession;
use crate::common::{WHOLESALE, core_bin, tmp};
use serde_json::Value;
use std::path::Path;

const SCENARIOS: &str = r#"
== deadcode-gate-default
write main.ts: import './lib';
write lib.ts: export const v = 1;
toml [graph];entry_globs = ["main.ts"]
run baseline! 0 baseline written
has deadPoint
write n.ts: export const z = 1;
run deadcode 0 unreferenced (born): n.ts
run deadcode 0 listed, not blocked
run check 0 -> pass
== deadcode-gate-born
write main.ts: import './lib';
write lib.ts: export const v = 1;
write s.ts: export const s = 1;
toml [graph];entry_globs = ["main.ts"];[deadcode];block = ["born"]
run baseline! 0 baseline written
run deadcode 0 unreferenced (standing): s.ts
run check 0 -> pass
write n.ts: export const z = 1;
run deadcode 1 deadcode check: 1 root(s) blocked (born): n.ts
run check 1 failed: dead_added
== deadcode-gate-orphaned
write main.ts: import './lib';
write lib.ts: export const v = 1;
toml [graph];entry_globs = ["main.ts"];[deadcode];block = ["orphaned"]
run baseline! 0 baseline written
write main.ts: export const m = 1;
run deadcode 1 deadcode check: 1 root(s) blocked (orphaned): lib.ts
run check 1 failed: dead_added
write main.ts: import './lib';
write n.ts: export const z = 1;
run deadcode 0 unreferenced (born): n.ts
run check 0 -> pass
== deadcode-gate-no-baseline
write a.ts: export const a = 1;
toml [deadcode];block = ["born"]
run deadcode 2 no committed baseline records which files were unreferenced
run check 2 no committed baseline records which files were unreferenced
== deadcode-gate-no-point
write a.ts: export const a = 1;
toml [deadcode];block = ["born"]
run baseline! 0 baseline written
strip
run deadcode 2 CE_ACCEPT_BASELINE=1 ce baseline .
run check 2 CE_ACCEPT_BASELINE=1 ce baseline .
== deadcode-gate-load
write a.ts: export const a = 1;
toml [deadcode];block = ["standing"]
run deadcode 2 "standing" cannot be blocked — the inventory never blocks
run check 2 "standing" cannot be blocked — the inventory never blocks
toml [deadcode];block = ["later"]
run deadcode 2 unknown state "later"
run check 2 unknown state "later"
== deadcode-gate-baseline
write a.ts: export const a = 1;
toml [deadcode];block = ["orphaned", "born"]
run baseline! 0 baseline written
has deadPoint
write n.ts: export const z = 1;
run baseline 1 refused by dead_added
run baseline! 0 baseline written
run deadcode 0 unreferenced (standing): n.ts
== deadcode-gate-rename
write s.ts: export const s = 1;
run baseline! 0 baseline written
mv s.ts t.ts
run deadcode 0 unreferenced (born): t.ts
== deadcode-gate-digest
write a.ts: export const a = 1;
toml [deadcode];block = []
run baseline! 0 baseline written
lacks knobsDigest
toml [deadcode];block = ["born"]
run check 1 failed: knobs_digest
run baseline! 0 baseline written
has knobsDigest
"#;

#[test]
fn the_gate_holds_its_states_against_the_committed_baseline() {
    let core = core_bin();
    let mut dir = tmp("deadcode-gate-unopened");
    let mut seen = 0;
    for line in SCENARIOS.trim().lines() {
        if let Some(name) = line.strip_prefix("== ") {
            dir = tmp(name);
            seen += 1;
            continue;
        }
        let (verb, rest) = line.split_once(' ').unwrap_or((line, ""));
        match verb {
            "toml" => common::declare(&dir, &format!("{}\n", rest.replace(';', "\n"))),
            "write" => {
                let (path, text) = rest.split_once(": ").expect("path: text");
                std::fs::write(dir.join(path), format!("{text}\n")).expect(path);
            }
            "mv" => {
                let (from, to) = rest.split_once(' ').expect("from to");
                std::fs::rename(dir.join(from), dir.join(to)).expect(from);
            }
            "strip" => edit_baseline(&dir, |b| drop(b.remove("deadPoint").expect("a point"))),
            "has" | "lacks" => edit_baseline(&dir, |b| {
                assert_eq!(b.contains_key(rest), verb == "has", "{line}: {b:?}")
            }),
            "run" => run(&dir, &core, rest),
            _ => panic!("unread line: {line}"),
        }
    }
    assert_eq!(seen, 9, "every scenario ran");
}

/// One command of the table: its exit code and the words it must say.
fn run(dir: &Path, core: &str, step: &str) {
    let mut parts = step.splitn(3, ' ');
    let (cmd, exit, words) = (
        parts.next().expect("a command"),
        parts.next().expect("an exit code"),
        parts.next().expect("words"),
    );
    let (args, env): (&[&str], &[(&str, &str)]) = match cmd {
        "deadcode" => (&["deadcode", ".", "--check"], &[]),
        "check" => (&["check", "."], &[]),
        "baseline" => (&["baseline", "."], &[]),
        "baseline!" => (&["baseline", "."], WHOLESALE),
        _ => panic!("unknown command {cmd}"),
    };
    let args: Vec<&str> = args.iter().copied().chain(["--core", core]).collect();
    let (code, out, err) = common::ce_triple(dir, &args, env);
    let said = format!("{out}{err}");
    assert_eq!(
        code.map(|c| c.to_string()).as_deref(),
        Some(exit),
        "{dir:?} {step}: {said}"
    );
    assert!(said.contains(words), "{dir:?} {step}: {said}");
}

/// Read (and, through `f`, edit) the committed baseline's object.
fn edit_baseline(dir: &Path, f: impl FnOnce(&mut serde_json::Map<String, Value>)) {
    let path = dir.join("ce-baseline.json");
    let mut doc: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("a baseline"))
        .expect("baseline json");
    f(doc.as_object_mut().expect("an object"));
    std::fs::write(&path, format!("{doc}\n")).expect("rewrite");
}

/// The MCP tools return the same documents: `deadcode` carries the gate
/// and each root's state and blocking, `check` the blocking roots in the
/// ratchet beside `dead_added` and the effective weights.
#[test]
fn the_mcp_tools_carry_the_gate() {
    let dir = common::gates::standing_root_under_block("deadcode-gate-mcp");
    std::fs::write(dir.join("n.ts"), "export const n = 1;\n").expect("n.ts");
    let mut s = McpSession::over(&dir);
    let dead: Value =
        serde_json::from_str(&s.relayed(1, "deadcode", serde_json::json!({}))).expect("doc");
    let check: Value =
        serde_json::from_str(&s.relayed(2, "check", serde_json::json!({}))).expect("doc");
    s.finish();
    assert_eq!(
        dead["gate"],
        serde_json::json!({"block": ["born"], "comparison": "baseline", "blocked": 1, "answered": true}),
        "{dead}"
    );
    assert_eq!(
        (
            dead["counts"]["born"].clone(),
            dead["counts"]["standing"].clone()
        ),
        (1.into(), 1.into()),
        "{dead}"
    );
    let rows: Vec<(&str, &str, bool)> = dead["dead"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|r| {
            (
                r["name"].as_str().unwrap(),
                r["state"].as_str().unwrap(),
                r["blocking"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![("n.ts", "born", true), ("s.ts", "standing", false)],
        "{dead}"
    );
    assert_eq!(
        check["ratchet"]["deadAdded"].as_array().map(Vec::len),
        Some(1),
        "{check}"
    );
    assert!(
        check["ratchet"]["failed"]
            .to_string()
            .contains("dead_added"),
        "{check}"
    );
    // the effective weights ride the check document (early review F6): no
    // [score.weights] declared here, so the dead axis (4) weighs 0
    assert_eq!(
        check["weights"],
        serde_json::json!([[0, 1], [1, 1], [2, 1], [3, 1], [4, 0], [5, 1], [6, 1]]),
        "{check}"
    );
}
