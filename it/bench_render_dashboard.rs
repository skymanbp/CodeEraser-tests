//! Generated benchmark surfaces beyond docs/BENCH.md and the home
//! page chips. CE_BLESS=1 owns only marked blocks; a plain run byte-
//! compares them. All values originate in contracts/bench/bench.json.

use crate::bench_support::render::{doc, latest, measured, rows_with, s};
use serde_json::Value;

fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn dashboard_rows(d: &Value) -> String {
    rows_with(d, "rows", |row| {
        format!(
            "<tr><td><code>{}</code></td><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td><code>{}</code></td><td>{}</td></tr>\n",
            esc(s(row, "version")),
            esc(s(row, "metric")),
            row["p50"],
            row["p95"],
            row["n"],
            esc(s(row, "host")),
            measured(row)
        )
    })
}

fn frozen_rows(d: &Value) -> String {
    rows_with(d, "frozen", |point| {
        format!(
            "<tr><td><code>{}</code></td><td>{}</td><td><code>{}</code></td></tr>\n",
            esc(s(point, "metric")),
            esc(s(point, "value")),
            esc(s(point, "source"))
        )
    })
}

fn render_dashboard(d: &Value, zh: bool) -> String {
    let (latency, frozen, metric, host, measured, version, value, source, note) = if zh {
        (
            "逐版本延迟",
            "冻结评估点",
            "指标",
            "主机",
            "实测日期",
            "版本",
            "值",
            "来源",
            "行来自发布版回放；冻结点保留其账本来源。完整冻结说明仍在 bench.json。",
        )
    } else {
        (
            "Latency by version",
            "Frozen evaluation points",
            "metric",
            "host",
            "measured",
            "version",
            "value",
            "source",
            "Series rows come from release-build replay; frozen points retain their ledger source. Full freeze notes remain in bench.json.",
        )
    };
    format!(
        "<h2>{latency}</h2>\n<div class=\"term data-table\"><div class=\"tablewrap\"><table><thead><tr><th>{version}</th><th>{metric}</th><th>p50 ms</th><th>p95 ms</th><th>n</th><th>{host}</th><th>{measured}</th></tr></thead><tbody>\n{}</tbody></table></div></div>\n<h2>{frozen}</h2>\n<div class=\"term data-table\"><div class=\"tablewrap\"><table><thead><tr><th>{metric}</th><th>{value}</th><th>{source}</th></tr></thead><tbody>\n{}</tbody></table></div><p class=\"cap\">{note}</p></div>\n",
        dashboard_rows(d),
        frozen_rows(d)
    )
}

fn latest_md(d: &Value) -> String {
    let version = latest(d);
    let rows: Vec<_> = d["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .filter(|row| s(row, "version") == version)
        .collect();
    let cells = |field: &str| {
        rows.iter()
            .map(|row| row[field].to_string())
            .collect::<Vec<_>>()
            .join(" | ")
    };
    format!(
        "| percentile | {} |\n|---|{}|\n| p50 ms | {} |\n| p95 ms | {} |\n",
        rows.iter()
            .map(|row| format!("`{}`", s(row, "metric")))
            .collect::<Vec<_>>()
            .join(" | "),
        rows.iter().map(|_| "---:").collect::<Vec<_>>().join("|"),
        cells("p50"),
        cells("p95")
    )
}

/// The README block carries the latest version's latency only; the
/// frozen evaluation points render whole into docs/BENCH.md and both
/// site dashboards (plan v2.21 #30, the README trim), which the
/// block's own tail links.
fn render_readme(d: &Value, zh: bool) -> String {
    let (latest_h, note, bench, site) = if zh {
        (
            "最新版本延迟",
            "所有值均由 `contracts/bench/bench.json` 生成；本块手改会被测试拒绝。",
            "完整回放说明与逐版本系列",
            "网站完整仪表盘",
        )
    } else {
        (
            "Latest-version latency",
            "Every value is generated from `contracts/bench/bench.json`; the test rejects hand edits to this block.",
            "Full replay notes and per-version series",
            "Complete website dashboard",
        )
    };
    format!(
        "### {latest_h} · v{}\n\n{}\n{note} [{bench}](docs/BENCH.md) · [{site}](https://codeeraser.dev{}/bench/)\n",
        latest(d),
        latest_md(d),
        if zh { "/zh" } else { "" }
    )
}

fn frozen<'a>(d: &'a Value, metric: &str) -> &'a Value {
    d["frozen"]
        .as_array()
        .expect("frozen")
        .iter()
        .find(|point| s(point, "metric") == metric)
        .unwrap_or_else(|| panic!("missing frozen metric {metric}"))
}

fn natural_numbers(text: &str) -> Vec<&str> {
    text.split(|c: char| !c.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .collect()
}

fn render_stack_fpr(d: &Value, zh: bool) -> String {
    let fourclass = frozen(d, "fourclass_fpr");
    let guard = frozen(d, "guard_fpr_per500");
    let nums = natural_numbers(s(guard, "detail"));
    let guard_record = format!("{}/{}", nums.last().expect("guard false count"), nums[0]);
    let (title, classifier, probe, false_label, tail) = if zh {
        (
            "误报纪律",
            "判定层",
            "写入探针",
            "误报",
            "其余规则保持 observe。每次晋级都须写入 CHANGELOG 台账。",
        )
    } else {
        (
            "False-positive discipline",
            "classifier",
            "write probe",
            "false positives",
            "Every other class stays observe. Each promotion is recorded in the CHANGELOG ledger.",
        )
    };
    format!(
        "<div class=\"card\"><b>{title}</b><p>{classifier}: <code>{}</code>; {probe}: <code>{guard_record} {false_label}</code>. {tail}</p></div>\n",
        esc(s(fourclass, "value"))
    )
}

fn digits(text: &str) -> Vec<&str> {
    text.split(|c: char| !(c.is_ascii_digit() || c == '.'))
        .filter(|part| part.chars().any(|c| c.is_ascii_digit()))
        .collect()
}

/// One page's dashboard block through the shared splicer (facts::block).
fn gate(rel: &str, marker: &str, rendered: String) {
    crate::facts::block::assert_current(rel, marker, &rendered);
}

#[test]
fn website_dashboard_blocks_match() {
    let d = doc();
    let (en, zh) = (render_dashboard(&d, false), render_dashboard(&d, true));
    assert_eq!(digits(&en), digits(&zh), "dashboard numeric facts drifted");
    gate("site/bench/index.html", "bench", en);
    gate("site/zh/bench/index.html", "bench", zh);
}

#[test]
fn readme_dashboard_blocks_match() {
    let d = doc();
    gate("README.md", "bench", render_readme(&d, false));
    gate("README.zh.md", "bench", render_readme(&d, true));
}

#[test]
fn stack_fpr_blocks_match() {
    let d = doc();
    gate(
        "site/stack/index.html",
        "bench-stack",
        render_stack_fpr(&d, false),
    );
    gate(
        "site/zh/stack/index.html",
        "bench-stack",
        render_stack_fpr(&d, true),
    );
}
