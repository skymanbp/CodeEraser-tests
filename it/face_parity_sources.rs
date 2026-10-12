//! The face sources the three-face parity gates read (plan v2.33 step
//! 10), split from face_parity_options.rs at the E01 line: the CLI's
//! help off the built binary, the MCP catalog off `cli/src/mcp/tools.rs`,
//! the Tauri signatures off `gui/src-tauri/src/commands*.rs`, all from
//! one tree — the repository, or a negative probe's scratch copy
//! (`CE_PARITY_ROOT`, and `CE_PARITY_BIN` for a ce built from it).

use crate::common::repo_root;
use crate::facts::read;
use crate::refusal_text::literals;
use std::path::PathBuf;

/// The tree the gate reads: the repository, or a negative probe's copy.
pub fn parity_root() -> PathBuf {
    std::env::var_os("CE_PARITY_ROOT").map_or_else(repo_root, PathBuf::from)
}

/// A source file of that tree, line ends read as `\n` (a checkout may
/// carry CRLF).
pub fn src(rel: &str) -> String {
    read(&parity_root(), rel).replace("\r\n", "\n")
}

/// `ce <sub> --help` in English.
pub fn help(sub: &str) -> String {
    let bin = std::env::var_os("CE_PARITY_BIN")
        .map_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_ce")), PathBuf::from);
    let out = std::process::Command::new(bin)
        .args(["--lang", "en", sub, "--help"])
        .output()
        .expect("ce --help");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The long flags and the positional arguments `ce <sub> --help` lists.
pub fn cli_names(sub: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in help(sub).lines() {
        let t = line.trim_start();
        let flag = t.strip_prefix("-h, ").unwrap_or(t).strip_prefix("--");
        if let Some(f) = flag {
            out.push(f.split([' ', ',']).next().expect("flag").to_string());
        } else if let Some(arg) = t.strip_prefix('[').or_else(|| t.strip_prefix('<')) {
            let word = arg.split([']', '>']).next().expect("arg");
            if word.chars().all(|c| c.is_ascii_uppercase() || c == '_') && !word.is_empty() {
                out.push(word.to_ascii_lowercase());
            }
        }
    }
    out
}

/// The MCP tool's input properties, read off its catalog row: each
/// `(name, type, …)` tuple, and each shared const the row names.
pub fn mcp_names(tool: &str) -> Vec<String> {
    let text = src("cli/src/mcp/tools.rs");
    let types = ["integer", "string", "boolean", "array"];
    let props = |block: &str| -> Vec<String> {
        let l = literals(block);
        (1..l.len())
            .filter(|&i| types.contains(&l[i].as_str()))
            .map(|i| l[i - 1].clone())
            .collect()
    };
    let head = format!("tool!(\n        \"{tool}\",");
    let block = text.split(&head).nth(1).expect("the tool's row");
    let block = block.split("tool!(").next().expect("row");
    let block = block.split("\n];").next().expect("the table's end");
    let mut out = props(block);
    for c in text.split("\nconst ").skip(1) {
        let (name, body) = c.split_once(':').expect("a const");
        if block.contains(&format!("{name}]")) || block.contains(&format!("{name},")) {
            out.extend(props(body.split(';').next().expect("const body")));
        }
    }
    out
}

/// A Tauri command's parameters, read off its signature (or the
/// `face_cmd!` invocation that mints it: `(name: Type, …) =>` is the
/// options arm, a bare body the plain one).
pub fn tauri_names(cmd: &str) -> Vec<String> {
    for file in ["commands.rs", "commands_query.rs", "commands_flow.rs"] {
        let text = src(&format!("gui/src-tauri/src/{file}"));
        for head in [format!("pub async fn {cmd}("), format!("pub fn {cmd}(")] {
            if let Some(rest) = text.split(&head).nth(1) {
                let sig = rest.split(')').next().expect("signature");
                return sig
                    .split(',')
                    .filter_map(|p| p.split_once(':').map(|(n, _)| n.trim().to_string()))
                    .collect();
            }
        }
        for call in text.split("face_cmd!(").skip(1) {
            let parts: Vec<&str> = call.splitn(3, ',').map(str::trim).collect();
            if parts[0] == cmd {
                let options = parts[2]
                    .strip_prefix('(')
                    .filter(|_| parts[2].contains(") =>"));
                let list = options.map_or("", |o| o.split(") =>").next().expect("options"));
                return list
                    .split(',')
                    .filter_map(|p| p.split_once(':').map(|(n, _)| n.trim().to_string()))
                    .collect();
            }
        }
    }
    panic!("{cmd}: no Tauri command of that name")
}
