//! Three-face parity, gated (user directive 2026-08-29: "the plugin,
//! the GUI and the CLI must be equivalent"). One capability table
//! claims, per capability, the CLI subcommands, the GUI screen and
//! Tauri commands, and the plugin surface (hooks, MCP tools, slash
//! commands, skills) that carry it. Every set is DERIVED from the
//! code — clap's enum, the Tauri handler roster and tab strip, the
//! MCP catalog, hooks.json, `plugin/commands`, `plugin/skills` — and
//! the gate holds both directions: a shipped face no row claims, and
//! a claimed face nobody shipped. Deliberate omissions are rows, not
//! silence. The rendered table is embedded in both READMEs between
//! `<!-- parity:begin -->` / `<!-- parity:end -->` and compared byte
//! for byte; `CE_BLESS=1` is the only writer.

use crate::common::repo_root;
use std::collections::BTreeSet;

/// The capability table, one row per line and seven `|` cells:
/// name (en) | name (zh) | CLI | GUI | plugin | note (en) | note (zh).
/// Items within a cell are comma-separated. A CLI item is the
/// subcommand plus any flag that names the act (`erase --apply`); the
/// claim is its first word. A GUI item is the tab (`tab:` prefixed)
/// or a Tauri command; a plugin item carries a kind prefix (`mcp:` /
/// `hook:` / `cmd:` / `skill:` / `mcpjson`). The note is the bilingual
/// reason written into the first empty cell. One string literal: a
/// list of same-shaped struct literals is a clone by construction.
const TABLE: &str = "
size / complexity / readability metrics | 尺寸 / 复杂度 / 可读性度量 | scan | tab:reports, scan_report | mcp:scan | |
T1/T2 clone blocks | T1/T2 克隆块 | dedup | tab:reports, dedup_report | mcp:check_duplication | |
T3 near-miss clones | T3 近似克隆 | clone | tab:reports, clone_report | mcp:clone | |
documentation duplication | 文档重复 | docdup | tab:reports, docdup_report | mcp:docdup | |
reference sites and the mention universe | 引用站点与提及宇宙 | graph | tab:reports, sites_report | mcp:graph_sites | |
liveness verdicts + symbol advisory | 存活性判决 + 符号顾问 | deadcode | tab:graph, graphcanvas_report, deadcode_report | mcp:deadcode | |
git-window churn | git 窗口变动 | churn | tab:candidates, churn_report | mcp:churn | |
three-signal join | 三信号联判 | join | tab:candidates, join_report | mcp:join | |
tree-scale structure (seven axes, split pricing) | 树尺度结构（七轴、拆分定价） | structure | tab:structure, structure_report | mcp:structure | |
score trajectory | 分数轨迹 | trend | tab:trend, trend_report | mcp:trend | |
score, ratchet and floor | 分数、棘轮与地板 | check | tab:score, check_report | mcp:check | |
baseline writes | 基线写入 | baseline | | | CLI only: a machine surface never writes a baseline | 只在 CLI：机器面永不写基线
erase plan | 擦除计划 | erase | tab:erase, erase_preview | mcp:erase, skill:erase | |
erase apply | 擦除执行 | erase --apply | tab:erase, erase_apply | | no MCP face: applying is a human act | 无 MCP 面：执行是人类动作
machine state | 本机状态 | doctor | tab:doctor, doctor_report | mcp:doctor | |
update check | 更新检查 | update | tab:update, update_check | mcp:update_check, cmd:update, hook:SessionStart | |
update apply | 更新执行 | update --yes | tab:update, update_apply | | the plugin's copy is re-pinned by `/plugin update codeeraser` | 插件副本由 `/plugin update codeeraser` 重钉
write-time guard | 写入时守卫 | probe --hook | | hook:PreToolUse | hooks are the plugin's face | 钩子即插件之面
stop audit / pre-commit | Stop 审计 / pre-commit | audit --hook, precommit | | hook:Stop | hooks are the plugin's face | 钩子即插件之面
session health line | 会话健康行 | health --hook | | hook:SessionStart | hooks are the plugin's face | 钩子即插件之面
project daemon | 项目 daemon | daemon, ping | | | started lazily by every face | 每一面惰性启动
read-only report server | 只读报告服务器 | mcp | | mcpjson | the plugin registers it | 插件自行注册
uninstall | 卸载 | eject | | | CLI only | 只在 CLI
bench dashboard | 实测仪表盘 | | tab:bench, bench_doc | | compiled-in series; README and site carry the same block | 编译内置序列；README 与官网带同一块
root anchoring | 根锚定 | | default_root, resolve_root | | every command and hook anchors through `root` | 每条命令与钩子都经 `root` 锚定
";

struct Row {
    en: String,
    zh: String,
    cli: Vec<String>,
    gui: Vec<String>,
    plugin: Vec<String>,
    note: (String, String),
}

fn items(cell: &str) -> Vec<String> {
    cell.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

fn rows() -> Vec<Row> {
    TABLE
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let c: Vec<&str> = l.split('|').map(str::trim).collect();
            assert_eq!(c.len(), 7, "seven cells: {l}");
            Row {
                en: c[0].into(),
                zh: c[1].into(),
                cli: items(c[2]),
                gui: items(c[3]),
                plugin: items(c[4]),
                note: (c[5].into(), c[6].into()),
            }
        })
        .collect()
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// clap's subcommand roster: every `Name {` / `Name(Args),` variant.
fn cli_subcommands() -> BTreeSet<String> {
    let src = read("cli/src/main_cli.rs");
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

fn gui_commands() -> BTreeSet<String> {
    read("gui/src-tauri/src/main.rs")
        .lines()
        .filter_map(|l| l.trim().strip_prefix("commands::"))
        .map(|l| l.trim_end_matches(',').to_string())
        .collect()
}

fn gui_tabs() -> BTreeSet<String> {
    read("gui/ui/index.html")
        .split("data-tab=\"")
        .skip(1)
        .map(|s| format!("tab:{}", s.split('"').next().expect("tab")))
        .collect()
}

fn mcp_tools() -> BTreeSet<String> {
    let src = read("cli/src/mcp/tools.rs");
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
        serde_json::from_str(&read("plugin/hooks/hooks.json")).expect("hooks.json");
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
        "| 能力 | CLI | GUI（屏 · 命令） | 插件（hooks · MCP · 命令 · skill） |\n"
    } else {
        "| capability | CLI | GUI (screen · commands) | plugin (hooks · MCP · commands · skills) |\n"
    });
    out += "|---|---|---|---|\n";
    for r in rows() {
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
        out += &format!("| {name} | {} | {} | {} |\n", cells[0], cells[1], cells[2]);
    }
    out
}

#[test]
fn the_readme_parity_tables_match_their_rendering() {
    let mut drift = Vec::new();
    for (page, zh) in [("README.md", false), ("README.zh.md", true)] {
        let text = read(page);
        let (head, rest) = text
            .split_once("<!-- parity:begin -->\n")
            .unwrap_or_else(|| panic!("{page}: no parity:begin"));
        let (_, tail) = rest
            .split_once("<!-- parity:end -->")
            .unwrap_or_else(|| panic!("{page}: no parity:end"));
        let want = format!(
            "{head}<!-- parity:begin -->\n{}<!-- parity:end -->{tail}",
            render(zh)
        );
        if want != text {
            if crate::facts::blessing() {
                std::fs::write(repo_root().join(page), want).expect("bless");
            } else {
                drift.push(page);
            }
        }
    }
    assert!(
        drift.is_empty(),
        "parity table drifted in {drift:?} — CE_BLESS=1 regenerates"
    );
}
