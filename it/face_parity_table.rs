//! The capability table face_parity.rs closes against the shipped
//! faces and the core's catalogue, split from that file at the E01
//! 300-line line when each row gained the report documents it carries
//! (plan v2.32 step 6).

/// The capability table, one row per line and eight `|` cells:
/// name (en) | name (zh) | documents | CLI | GUI | plugin | notes (en) |
/// notes (zh). Items within a cell are comma-separated. The documents
/// are the core catalogue's family names (`tables/1` `document`) the
/// row's faces print: every family the catalogue lists sits in exactly
/// one row, and a row that names one carries all three faces or a note
/// for each face it lacks. A CLI item is the subcommand and the flags
/// that name the act (`erase --apply`), compared whole against the
/// subcommand's `--help`; a hook's item names, in brackets, the guard
/// classes its legs judge (`probe --hook [guard+flow+tombstone]`). A
/// GUI item is the tab (`tab:` prefixed) or a Tauri command; a plugin
/// item carries a kind prefix (`mcp:` / `hook:` / `cmd:` / `skill:` /
/// `mcpjson`). A note names the ONE face cell it explains (`gui: …`,
/// several joined by ` ;; `, the en and zh cells in the same order) and
/// is written into that cell, which must be empty. One string literal:
/// a list of same-shaped struct literals is a clone by construction.
const TABLE: &str = "
size / complexity / readability metrics | 尺寸 / 复杂度 / 可读性度量 | scan | scan | tab:reports, scan_report | mcp:scan | |
T1/T2 clone blocks | T1/T2 克隆块 | dedup | dedup | tab:reports, tab:candidates, dedup_report | mcp:check_duplication | |
T3 near-miss clones | T3 近似克隆 | clone | clone | tab:reports, clone_report | mcp:clone | |
the unit universe T3 judges | T3 所判的单元宇宙 | clone-units | clone --units | tab:reports, clone_report | mcp:clone | |
documentation duplication | 文档重复 | docdup | docdup | tab:reports, docdup_report | mcp:docdup | |
reference sites | 引用站点 | sites | graph --sites | tab:reports, sites_report | mcp:graph_sites | |
the mention universe | 提及宇宙 | mentions | graph --mentions | | | gui: CLI only: the census behind the symbol advisory, which every face of `deadcode` carries ;; plugin: CLI only: the MCP `deadcode` tool carries the advisory the census feeds | gui: 只在 CLI：符号顾问背后的普查，`deadcode` 的每一面都带着顾问本身 ;; plugin: 只在 CLI：MCP `deadcode` 工具带着这份普查所喂的顾问
liveness verdicts + symbol advisory | 存活性判决 + 符号顾问 | deadcode | deadcode | tab:reports, deadcode_report | mcp:deadcode | |
graph screen (canvas + liveness) | 图屏（画布 + 存活性） | graphscreen | | tab:graph, graphscreen_report | | cli: GUI only: the canvas is a picture; the same judgment's CLI and MCP faces are `deadcode` ;; plugin: GUI only: the MCP face of the same judgment is `deadcode` | cli: 只在 GUI：画布是一张图；同一判决的 CLI 与 MCP 面是 `deadcode` ;; plugin: 只在 GUI：同一判决的 MCP 面是 `deadcode`
git-window churn | git 窗口变动 | churn | churn | tab:reports, churn_report | mcp:churn | |
three-signal join | 三信号联判 | join | join | tab:candidates, join_report | mcp:join | |
tree-scale structure (split pricing) | 树尺度结构（拆分定价） | structure | structure | tab:structure, structure_report | mcp:structure | |
score trajectory | 分数轨迹 | trend | trend | tab:trend, trend_report | mcp:trend | |
score, ratchet and floor | 分数、棘轮与地板 | check | check | tab:score, check_report | mcp:check | |
same-role advisor (similar units, associative view) | 同角色顾问（相似单元、联想视图） | similar | similar | tab:similar, similar_report | mcp:similar_units | |
code query and architecture rules | 代码查询与架构规则 | query, rules | query, query --prelude, rules | tab:query, query_report, query_prelude, rules_report | mcp:query, mcp:rules | |
intra-function dead code (unreachable, dead stores, unused locals and parameters) | 函数内死代码（不可达、死存储、未用局部量与形参） | flow | flow, flow --check | tab:reports, flow_report, flow_kinds | mcp:flow | |
clone merge suggestions (anti-unification) | 克隆合并建议（反统一） | merge | merge | tab:reports, merge_report | mcp:merge_suggestions | |
architecture analysis (layers, cuts, clusters, impact) | 架构分析（分层、拆环、簇、影响面） | arch | arch | tab:reports, arch_report | mcp:architecture | |
baseline writes | 基线写入 | | baseline | | | gui: CLI only: a machine surface never writes a baseline | gui: 只在 CLI：机器面永不写基线
erase plan | 擦除计划 | erase | erase | tab:erase, erase_preview | mcp:erase, skill:erase | |
erase apply | 擦除执行 | | erase --apply | tab:erase, erase_apply | | plugin: no MCP face: applying is a human act | plugin: 无 MCP 面：执行是人类动作
erase audit log | 擦除审计日志 | erase-trail | erase --log | tab:erase, erase_log_report | mcp:erase_log | |
machine state and guard-class tiers | 本机状态与守卫类档位 | | doctor | tab:doctor, doctor_report | mcp:doctor | |
update check | 更新检查 | | update | tab:update, update_check | mcp:update_check, cmd:update, hook:SessionStart | |
update apply | 更新执行 | | update --yes | tab:update, update_apply | | plugin: the plugin's copy is re-pinned by `/plugin update codeeraser` | plugin: 插件副本由 `/plugin update codeeraser` 重钉
write-time guard | 写入时守卫 | | probe --hook [guard+flow+tombstone] | | hook:PreToolUse | gui: hooks are the plugin's face; flow and tombstone judge the write inside the hook, so no report family of their own — their tiers ride every face's doctor | gui: 钩子即插件之面；flow 与 tombstone 在钩子里判这次写入，故无自己的报告家族——它们的档位随每一面的 doctor 出示
asked-write settlement | ask 档写入的落地记录 | | settle --hook | | hook:PostToolUse | gui: hooks are the plugin's face | gui: 钩子即插件之面
stop audit / git hooks | Stop 审计 / git 钩子 | | audit --hook [guard+flow+tombstone], precommit [guard+flow+tombstone], commitmsg [guard+flow+tombstone] | | hook:Stop | gui: hooks are the plugin's face; precommit and commitmsg are git's; tombstone judges the changeset inside them, its tier on every face's doctor | gui: 钩子即插件之面；precommit 与 commitmsg 挂在 git 里；tombstone 在其中判改动集，档位见每一面的 doctor
session health line | 会话健康行 | | health --hook | | hook:SessionStart | gui: hooks are the plugin's face | gui: 钩子即插件之面
project daemon | 项目 daemon | | daemon, ping | | | gui: started lazily by every face | gui: 每一面惰性启动
read-only report server | 只读报告服务器 | | mcp | | mcpjson | gui: the plugin registers it | gui: 插件自行注册
uninstall | 卸载 | | eject | | | gui: CLI only | gui: 只在 CLI
Claude Code wiring | Claude Code 接线 | | setup, setup --unwire | | | gui: CLI only: the Windows installer calls it, AppImage / dmg users run it once | gui: 只在 CLI：Windows 安装包调用它，AppImage / dmg 用户装后跑一次
bench dashboard | 实测仪表盘 | | | tab:bench, bench_doc | | cli: compiled-in series; README and site carry the same block | cli: 编译内置序列；README 与官网带同一块
root anchoring | 根锚定 | | | default_root, resolve_root | | cli: every command and hook anchors through `root` | cli: 每条命令与钩子都经 `root` 锚定
";

/// The three face columns a note may name, in table order.
pub const FACES: [&str; 3] = ["cli", "gui", "plugin"];

pub struct Row {
    pub en: String,
    pub zh: String,
    pub docs: Vec<String>,
    pub cli: Vec<String>,
    pub gui: Vec<String>,
    pub plugin: Vec<String>,
    /// `(face, en, zh)`, one per face cell a note explains.
    pub notes: Vec<(String, String, String)>,
}

impl Row {
    /// The face cells in FACES order.
    pub fn faces(&self) -> [&Vec<String>; 3] {
        [&self.cli, &self.gui, &self.plugin]
    }
}

fn items(cell: &str) -> Vec<String> {
    cell.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// `face: text ;; face: text` in each language, paired in order: the
/// two cells name the same faces, each a FACES column.
fn notes(en: &str, zh: &str) -> Vec<(String, String, String)> {
    let split = |cell: &str| -> Vec<(String, String)> {
        cell.split(" ;; ")
            .filter(|s| !s.trim().is_empty())
            .map(|s| {
                let (face, text) = s.split_once(": ").expect("a note is `face: text`");
                assert!(FACES.contains(&face.trim()), "a note names no face: {s}");
                (face.trim().to_string(), text.trim().to_string())
            })
            .collect()
    };
    let (en, zh) = (split(en), split(zh));
    assert_eq!(en.len(), zh.len(), "the en and zh notes pair up: {en:?}");
    en.into_iter()
        .zip(zh)
        .map(|((f, e), (g, z))| {
            assert_eq!(f, g, "the en and zh notes name the same face");
            (f, e, z)
        })
        .collect()
}

pub fn rows() -> Vec<Row> {
    TABLE
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let c: Vec<&str> = l.split('|').map(str::trim).collect();
            assert_eq!(c.len(), 8, "eight cells: {l}");
            Row {
                en: c[0].into(),
                zh: c[1].into(),
                docs: items(c[2]),
                cli: items(c[3]),
                gui: items(c[4]),
                plugin: items(c[5]),
                notes: notes(c[6], c[7]),
            }
        })
        .collect()
}
