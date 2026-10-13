//! Three-face parity, the road (plan v2.33 step 10): a capability row
//! and equal option sets still allow a GUI command that answers off a
//! road of its own. Here each report pair's Tauri command is read off
//! its body in `gui/src-tauri/src/commands*.rs` and must reach exactly
//! one family face in `crate::faces`, the same one the pair's MCP
//! adapter (`cli/src/mcp/adapters.rs`, a minted row read off its own
//! line of the `plain!` table) reaches: the GUI passes its arguments through to the
//! face the MCP tool calls, so both answer the one document. A face
//! that answers through a sibling face (the erase preview is the erase
//! face plus the diff it rendered) is read as that sibling.

use crate::face_parity_options::{none, pairs};
use crate::face_parity_sources::src;
use std::collections::BTreeSet;

const PLUMBING: [&str; 5] = ["doc", "gated_doc", "Plumb", "self", "core"];

/// The `faces::<fn>` names a body reaches, minus the plumbing, each
/// read through the sibling face it answers by.
fn face_calls(body: &str) -> BTreeSet<String> {
    body.split("faces::")
        .skip(1)
        .map(|t| {
            t.chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect()
        })
        .filter(|n: &String| !PLUMBING.contains(&n.as_str()))
        .map(|n| through(&n))
        .collect()
}

/// The face `name` answers by: itself, or the sibling face its body
/// in `cli/src/faces/` calls.
fn through(name: &str) -> String {
    let text = src("cli/src/faces/mod.rs") + &src("cli/src/faces/ask.rs");
    let Some(body) = span(
        &text,
        &format!("pub fn {name}("),
        "
}
",
    ) else {
        return name.to_string();
    };
    let sibling = text
        .split("pub fn ")
        .skip(1)
        .filter_map(|t| t.split('(').next())
        .filter(|m| *m != name && !PLUMBING.contains(m))
        .find(|m| body.contains(&format!(" {m}(")) || body.contains(&format!("({m}(")));
    sibling.map_or_else(|| name.to_string(), through)
}

/// The text from `head` to the first `end` after it, if `head` is there.
fn span<'a>(text: &'a str, head: &str, end: &str) -> Option<&'a str> {
    let rest = text.split_once(head)?.1;
    Some(rest.split(end).next().unwrap_or(rest))
}

/// A Tauri command's body: its `fn`, or the `face_cmd!` call that
/// mints it.
fn tauri_body(cmd: &str) -> String {
    for file in ["commands.rs", "commands_query.rs", "commands_flow.rs"] {
        let text = src(&format!("gui/src-tauri/src/{file}"));
        let found = span(&text, &format!(" fn {cmd}("), "\n}\n")
            .or_else(|| span(&text, &format!("face_cmd!({cmd},"), ";\n"));
        if let Some(body) = found {
            return body.to_string();
        }
    }
    panic!("{cmd}: no Tauri command of that name")
}

/// An MCP tool's adapter body: the fn its catalog row names, or the
/// `plain!` row that mints it (the row names its face call).
fn mcp_body(tool: &str) -> String {
    let catalog = src("cli/src/mcp/tools.rs");
    let row = span(&catalog, &format!("tool!(\n        \"{tool}\","), ",").expect("the tool's row");
    let run = row.trim();
    let adapters = src("cli/src/mcp/adapters.rs");
    if let Some(body) = span(&adapters, &format!("fn {run}("), "\n}\n") {
        return body.to_string();
    }
    span(&adapters, &format!("    {run} => "), "\n")
        .expect("a plain! row")
        .to_string()
}

/// Per pair: the GUI's one family face, or what is wrong with it.
pub fn road_mismatches() -> Vec<String> {
    let mut out = Vec::new();
    for line in pairs() {
        let w: Vec<&str> = line.split_whitespace().collect();
        let (tool, cmd) = (w[1], w[2]);
        let (gui, mcp) = (face_calls(&tauri_body(cmd)), face_calls(&mcp_body(tool)));
        if gui.len() != 1 {
            out.push(format!(
                "{cmd}: reaches {gui:?}, not exactly one family face"
            ));
        } else if !gui.is_subset(&mcp) {
            out.push(format!(
                "{cmd}: reaches {gui:?}; MCP {tool} reaches {mcp:?}"
            ));
        }
    }
    out
}

#[test]
fn every_gui_command_passes_through_to_the_face_its_mcp_tool_calls() {
    none(road_mismatches());
}
