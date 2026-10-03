//! The capability table face_parity.rs closes against the shipped
//! faces and the core's catalogue, split from that file at the E01
//! 300-line line when each row gained the report documents it carries
//! (plan v2.32 step 6).

/// The capability table, one row per line and eight `|` cells:
/// name (en) | name (zh) | documents | CLI | GUI | plugin | note (en) |
/// note (zh). Items within a cell are comma-separated. The documents
/// are the core catalogue's family names (`tables/1` `document`) the
/// row's faces print: every family the catalogue lists sits in exactly
/// one row, and a row that names one carries all three faces or says
/// why not. A CLI item is the subcommand plus any flag that names the
/// act (`erase --apply`); the claim is its first word. A GUI item is
/// the tab (`tab:` prefixed) or a Tauri command; a plugin item carries
/// a kind prefix (`mcp:` / `hook:` / `cmd:` / `skill:` / `mcpjson`).
/// The note is the bilingual reason written into the first empty face
/// cell. One string literal: a list of same-shaped struct literals is a
/// clone by construction.
const TABLE: &str = "
size / complexity / readability metrics | 尺寸 / 复杂度 / 可读性度量 | scan | scan | tab:reports, scan_report | mcp:scan | |
T1/T2 clone blocks | T1/T2 克隆块 | dedup | dedup | tab:reports, dedup_report | mcp:check_duplication | |
T3 near-miss clones | T3 近似克隆 | clone | clone | tab:reports, clone_report | mcp:clone | |
the unit universe T3 judges | T3 所判的单元宇宙 | clone-units | clone --units | | mcp:clone | no GUI screen: the listing is the judgment's input, read through the CLI and the MCP tool's `units` | 无 GUI 屏：这份清单是判决的输入，经 CLI 与 MCP 工具的 `units` 读取
documentation duplication | 文档重复 | docdup | docdup | tab:reports, docdup_report | mcp:docdup | |
reference sites | 引用站点 | sites | graph --sites | tab:reports, sites_report | mcp:graph_sites | |
the mention universe | 提及宇宙 | mentions | graph --mentions | | | CLI only: the census behind the symbol advisory, which every face of `deadcode` carries | 只在 CLI：符号顾问背后的普查，`deadcode` 的每一面都带着顾问本身
liveness verdicts + symbol advisory | 存活性判决 + 符号顾问 | deadcode | deadcode | tab:reports, deadcode_report | mcp:deadcode | |
graph screen (canvas + liveness) | 图屏（画布 + 存活性） | graphscreen | | tab:graph, graphscreen_report | | GUI only: the canvas is a picture; the same judgment's CLI and MCP faces are `deadcode` | 只在 GUI：画布是一张图；同一判决的 CLI 与 MCP 面是 `deadcode`
git-window churn | git 窗口变动 | churn | churn | tab:candidates, churn_report | mcp:churn | |
three-signal join | 三信号联判 | join | join | tab:candidates, join_report | mcp:join | |
tree-scale structure (split pricing) | 树尺度结构（拆分定价） | structure | structure | tab:structure, structure_report | mcp:structure | |
score trajectory | 分数轨迹 | trend | trend | tab:trend, trend_report | mcp:trend | |
score, ratchet and floor | 分数、棘轮与地板 | check | check | tab:score, check_report | mcp:check | |
same-role advisor (similar units, associative view) | 同角色顾问（相似单元、联想视图） | similar | similar | tab:similar, similar_report | mcp:similar_units | |
code query and architecture rules | 代码查询与架构规则 | query, rules | query, rules | tab:query, query_report, rules_report | mcp:query, mcp:rules | |
intra-function dead code (unreachable, dead stores, unused locals and parameters) | 函数内死代码（不可达、死存储、未用局部量与形参） | flow | flow, flow --check | tab:reports, flow_report, flow_kinds | mcp:flow | |
clone merge suggestions (anti-unification) | 克隆合并建议（反统一） | merge | merge | tab:reports, merge_report | mcp:merge_suggestions | |
architecture analysis (layers, cuts, clusters, impact) | 架构分析（分层、拆环、簇、影响面） | arch | arch | tab:reports, arch_report | mcp:architecture | |
baseline writes | 基线写入 | | baseline | | | CLI only: a machine surface never writes a baseline | 只在 CLI：机器面永不写基线
erase plan | 擦除计划 | erase | erase | tab:erase, erase_preview | mcp:erase, skill:erase | |
erase apply | 擦除执行 | | erase --apply | tab:erase, erase_apply | | no MCP face: applying is a human act | 无 MCP 面：执行是人类动作
erase audit log | 擦除审计日志 | erase-trail | erase --log | tab:erase, erase_log_report | mcp:erase_log | |
machine state | 本机状态 | | doctor | tab:doctor, doctor_report | mcp:doctor | |
update check | 更新检查 | | update | tab:update, update_check | mcp:update_check, cmd:update, hook:SessionStart | |
update apply | 更新执行 | | update --yes | tab:update, update_apply | | the plugin's copy is re-pinned by `/plugin update codeeraser` | 插件副本由 `/plugin update codeeraser` 重钉
write-time guard | 写入时守卫 | | probe --hook | | hook:PreToolUse | hooks are the plugin's face | 钩子即插件之面
asked-write settlement | ask 档写入的落地记录 | | settle --hook | | hook:PostToolUse | hooks are the plugin's face | 钩子即插件之面
stop audit / git hooks | Stop 审计 / git 钩子 | | audit --hook, precommit, commitmsg | | hook:Stop | hooks are the plugin's face; precommit and commitmsg are git's | 钩子即插件之面；precommit 与 commitmsg 挂在 git 里
session health line | 会话健康行 | | health --hook | | hook:SessionStart | hooks are the plugin's face | 钩子即插件之面
project daemon | 项目 daemon | | daemon, ping | | | started lazily by every face | 每一面惰性启动
read-only report server | 只读报告服务器 | | mcp | | mcpjson | the plugin registers it | 插件自行注册
uninstall | 卸载 | | eject | | | CLI only | 只在 CLI
Claude Code wiring | Claude Code 接线 | | setup, setup --unwire | | | CLI only: the Windows installer calls it, AppImage / dmg users run it once | 只在 CLI：Windows 安装包调用它，AppImage / dmg 用户装后跑一次
bench dashboard | 实测仪表盘 | | | tab:bench, bench_doc | | compiled-in series; README and site carry the same block | 编译内置序列；README 与官网带同一块
root anchoring | 根锚定 | | | default_root, resolve_root | | every command and hook anchors through `root` | 每条命令与钩子都经 `root` 锚定
";

pub struct Row {
    pub en: String,
    pub zh: String,
    pub docs: Vec<String>,
    pub cli: Vec<String>,
    pub gui: Vec<String>,
    pub plugin: Vec<String>,
    pub note: (String, String),
}

fn items(cell: &str) -> Vec<String> {
    cell.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
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
                note: (c[6].into(), c[7].into()),
            }
        })
        .collect()
}
