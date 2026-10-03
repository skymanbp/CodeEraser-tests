//! Three-face parity, gated (user directive 2026-08-29: "the plugin,
//! the GUI and the CLI must be equivalent"). One capability table
//! claims, per capability, the CLI subcommands, the GUI screen and
//! Tauri commands, and the plugin surface (hooks, MCP tools, slash
//! commands, skills) that carry it. Every set is DERIVED from the
//! code — clap's enum, the Tauri handler roster and tab strip, the
//! MCP catalog, hooks.json, `plugin/commands`, `plugin/skills` — and
//! the gate holds both directions: a shipped face no row claims, and
//! a claimed face nobody shipped. Deliberate omissions are rows, not
//! silence. The rows' spine is the core's catalogue (plan v2.32 step
//! 6): every report family `tables/1` lists sits in exactly one row,
//! and that row carries the three faces or says why not. The rendered
//! table is embedded in both READMEs between `<!-- parity:begin -->` /
//! `<!-- parity:end -->` and compared byte for byte; `CE_BLESS=1` is
//! the only writer. The table itself lives in face_parity_table.rs.

use crate::common::{repo_root, stub_core};
use crate::face_parity_table::{Row, rows};
use crate::facts::read;
use std::collections::BTreeSet;

/// clap's subcommand roster: every `Name {` / `Name(Args),` variant
/// (cli_table.rs closes the README carrier table against it).
pub(crate) fn cli_subcommands() -> BTreeSet<String> {
    let src = read(&repo_root(), "cli/src/main_cli.rs");
    let body = src.split("pub(crate) enum Cmd {").nth(1).expect("Cmd enum");
    body.lines()
        .filter_map(|l| l.strip_prefix("    "))
        .filter(|l| l.starts_with(|c: char| c.is_ascii_uppercase()))
        .map(|l| {
            l.split(|c: char| !c.is_ascii_alphanumeric())
                .next()
                .expect("variant")
                .to_ascii_lowercase()
        })
        .collect()
}

/// Every command the handler list registers, whichever `commands*`
/// module it lives in (the query family's two sit in commands_query.rs
/// because commands.rs stands at the 300-line line).
fn gui_commands() -> BTreeSet<String> {
    read(&repo_root(), "gui/src-tauri/src/main.rs")
        .lines()
        .filter_map(|l| l.trim().strip_prefix("commands"))
        .filter_map(|l| l.split_once("::"))
        .map(|(_, name)| name.trim_end_matches(',').to_string())
        .collect()
}

fn gui_tabs() -> BTreeSet<String> {
    read(&repo_root(), "gui/ui/index.html")
        .split("data-tab=\"")
        .skip(1)
        .map(|s| format!("tab:{}", s.split('"').next().expect("tab")))
        .collect()
}

fn mcp_tools() -> BTreeSet<String> {
    let src = read(&repo_root(), "cli/src/mcp/tools.rs");
    let table = src.split("pub const TOOLS").nth(1).expect("TOOLS");
    table
        .split("tool!(")
        .skip(1)
        .map(|s| {
            let name = s
                .trim()
                .trim_start_matches('"')
                .split('"')
                .next()
                .expect("name");
            format!("mcp:{name}")
        })
        .collect()
}

fn plugin_surface() -> BTreeSet<String> {
    let hooks: serde_json::Value =
        serde_json::from_str(&read(&repo_root(), "plugin/hooks/hooks.json")).expect("hooks.json");
    let mut out: BTreeSet<String> = hooks["hooks"]
        .as_object()
        .expect("events")
        .keys()
        .map(|k| format!("hook:{k}"))
        .collect();
    for (dir, prefix) in [("plugin/commands", "cmd:"), ("plugin/skills", "skill:")] {
        for e in std::fs::read_dir(repo_root().join(dir))
            .expect(dir)
            .flatten()
        {
            let name = e
                .file_name()
                .to_string_lossy()
                .trim_end_matches(".md")
                .to_string();
            out.insert(format!("{prefix}{name}"));
        }
    }
    if repo_root().join("plugin/.mcp.json").is_file() {
        out.insert("mcpjson".into());
    }
    out
}

fn claimed(pick: fn(&Row) -> &[String]) -> BTreeSet<String> {
    rows()
        .iter()
        .flat_map(|r| {
            pick(r)
                .iter()
                .map(|s| s.split(' ').next().expect("word").to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn every_face_is_claimed_by_a_row_and_every_claim_ships() {
    let (tabs, commands): (BTreeSet<String>, BTreeSet<String>) = claimed(|r| &r.gui)
        .into_iter()
        .partition(|g| g.starts_with("tab:"));
    let cases = [
        ("CLI subcommands", cli_subcommands(), claimed(|r| &r.cli)),
        ("GUI commands", gui_commands(), commands),
        ("GUI tabs", gui_tabs(), tabs),
        (
            "plugin surface",
            mcp_tools().union(&plugin_surface()).cloned().collect(),
            claimed(|r| &r.plugin),
        ),
    ];
    for (what, derived, claimed) in &cases {
        assert!(!derived.is_empty(), "{what}: the derivation found nothing");
        let unclaimed: Vec<_> = derived.difference(claimed).collect();
        let unshipped: Vec<_> = claimed.difference(derived).collect();
        assert!(
            unclaimed.is_empty(),
            "{what} shipped but claimed by no row: {unclaimed:?}"
        );
        assert!(
            unshipped.is_empty(),
            "{what} claimed but not shipped: {unshipped:?}"
        );
    }
}

/// The catalogue closes the table (plan v2.32 step 6): the families a
/// row names are the core's own, each in one row only, every family is
/// named, and a row that carries a report document has a CLI, a GUI and
/// a plugin face or a note saying why not. A family the core gains
/// without a face, or a face that drops out from under a family, is red
/// here before it is silent on a screen.
#[test]
fn every_catalogue_family_sits_in_one_row_with_its_faces() {
    let catalogue: BTreeSet<String> = stub_core::real_tables()["document"]
        .as_object()
        .expect("the catalogue's families")
        .keys()
        .cloned()
        .collect();
    let table = rows();
    let named: Vec<&String> = table.iter().flat_map(|r| &r.docs).collect();
    let once: BTreeSet<String> = named.iter().map(|d| d.to_string()).collect();
    assert_eq!(named.len(), once.len(), "a family in two rows: {named:?}");
    let unclaimed: Vec<_> = catalogue.difference(&once).collect();
    let unknown: Vec<_> = once.difference(&catalogue).collect();
    assert!(unclaimed.is_empty(), "families no row names: {unclaimed:?}");
    assert!(
        unknown.is_empty(),
        "families the core does not list: {unknown:?}"
    );
    let faceless: Vec<&str> = table
        .iter()
        .filter(|r| !r.docs.is_empty() && r.note.0.is_empty())
        .filter(|r| [&r.cli, &r.gui, &r.plugin].iter().any(|f| f.is_empty()))
        .map(|r| r.en.as_str())
        .collect();
    assert!(faceless.is_empty(), "a face missing, no note: {faceless:?}");
}

/// The webview's grants are the documented set and no more: core, the
/// event channel the `ce-task` feed rides, and the dialog plugin's
/// `open` alone (the folder picker behind the root field, 2026-09-10).
/// gui.md calls it "the one dialog the shell opens"; a sentence like
/// that needs a reader, or the next plugin's default set (message,
/// save, ask, confirm) ships under it unread.
#[test]
fn the_webview_grants_exactly_the_documented_permissions() {
    let root = repo_root();
    let cap: serde_json::Value =
        serde_json::from_str(&read(&root, "gui/src-tauri/capabilities/default.json"))
            .expect("capability json");
    let granted: Vec<&str> = cap["permissions"]
        .as_array()
        .expect("permissions")
        .iter()
        .map(|p| p.as_str().expect("a permission id"))
        .collect();
    assert_eq!(
        granted,
        ["core:default", "core:event:default", "dialog:allow-open"]
    );
    assert!(
        read(&root, "docs/reference/gui.md")
            .contains("granted `dialog:allow-open` and nothing else"),
        "gui.md no longer states the grant this leg holds"
    );
}

fn plugin_word(item: &str) -> String {
    if let Some(cmd) = item.strip_prefix("cmd:") {
        return format!("`/codeeraser:{cmd}`");
    }
    for (prefix, word) in [("mcp:", "MCP "), ("hook:", "hook "), ("skill:", "skill ")] {
        if let Some(rest) = item.strip_prefix(prefix) {
            return format!("{word}`{rest}`");
        }
    }
    if item == "mcpjson" {
        return "`.mcp.json`".into();
    }
    format!("`{item}`")
}

fn cell(items: Vec<String>) -> String {
    if items.is_empty() {
        "—".into()
    } else {
        items.join(", ")
    }
}

fn render(zh: bool) -> String {
    let mut out = String::from(if zh {
        "| 能力 | 报告文档（核目录） | CLI | GUI（屏 · 命令） | 插件（hooks · MCP · 命令 · skill） |\n"
    } else {
        "| capability | report document (core catalogue) | CLI | GUI (screen · commands) | plugin (hooks · MCP · commands · skills) |\n"
    });
    out += "|---|---|---|---|---|\n";
    for r in rows() {
        let docs = cell(r.docs.iter().map(|d| format!("`{d}`")).collect());
        let mut cells = [
            cell(r.cli.iter().map(|c| format!("`ce {c}`")).collect()),
            cell(
                r.gui
                    .iter()
                    .map(|g| format!("`{}`", g.trim_start_matches("tab:")))
                    .collect(),
            ),
            cell(r.plugin.iter().map(|p| plugin_word(p)).collect()),
        ];
        let note = if zh { &r.note.1 } else { &r.note.0 };
        if !note.is_empty()
            && let Some(empty) = cells.iter_mut().find(|c| *c == "—")
        {
            *empty = format!("— {note}");
        }
        let name = if zh { &r.zh } else { &r.en };
        out += &format!(
            "| {name} | {docs} | {} | {} | {} |\n",
            cells[0], cells[1], cells[2]
        );
    }
    out
}

#[test]
fn the_readme_parity_tables_match_their_rendering() {
    let drift: Vec<String> = [("README.md", false), ("README.zh.md", true)]
        .into_iter()
        .filter_map(|(page, zh)| crate::facts::block::splice(page, "parity", &render(zh)))
        .collect();
    assert!(drift.is_empty(), "{}", drift.join("\n"));
}
