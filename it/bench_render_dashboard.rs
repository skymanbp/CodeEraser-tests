//! Generated benchmark surfaces beyond docs/BENCH.md and the home
//! page chips. CE_BLESS=1 owns only marked blocks; a plain run byte-
//! compares them. All values originate in contracts/bench/bench.json.

use crate::bench_support::frozen;
use crate::bench_support::render::{
    doc, join, latest, measured, names_the_release, rows_with, s, unmeasured_note,
};
use serde_json::Value;

fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn dashboard_rows(d: &Value, zh: bool) -> String {
    rows_with(d, "rows", |row| {
        format!(
            "<tr><td><code>{}</code></td><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td><code>{}</code></td><td>{}</td></tr>\n",
            esc(s(row, "version")),
            esc(s(row, "metric")),
            row["p50"],
            row["p95"],
            row["n"],
            esc(s(row, "host")),
            measured(row, zh)
        )
    })
}

/// The metric and the source are identifiers — a name a reader looks
/// up and a path they open — and stay as the ledger spells them in
/// both languages. Only the value column is prose, and it had been
/// English on the Chinese page since the table was first drawn:
/// `bench_support::frozen` holds the one table of Chinese sentences,
/// each spliced with the numbers the ledger's own value states.
fn frozen_rows(d: &Value, zh: bool) -> String {
    rows_with(d, "frozen", |point| {
        format!(
            "<tr><td><code>{}</code></td><td>{}</td><td><code>{}</code></td></tr>\n",
            esc(s(point, "metric")),
            esc(&frozen::value(point, zh)),
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
    // the heading names the version MEASURED and the caption names
    // the release this build is when the two differ (docs/BENCH.md):
    // these two pages are the most detailed public latency surface,
    // and they were the two of seven surfaces that said neither
    let unmeasured = unmeasured_note(d, zh);
    let caption = if unmeasured.is_empty() {
        String::new()
    } else {
        format!("<p class=\"cap\">{}</p>\n", unmeasured.trim())
    };
    format!(
        "<h2>{latency} · v{measured_version}</h2>\n<div class=\"term data-table\"><div class=\"tablewrap\"><table><thead><tr><th>{version}</th><th>{metric}</th><th>p50 ms</th><th>p95 ms</th><th>n</th><th>{host}</th><th>{measured}</th></tr></thead><tbody>\n{}</tbody></table></div></div>\n{caption}<h2>{frozen}</h2>\n<div class=\"term data-table\"><div class=\"tablewrap\"><table><thead><tr><th>{metric}</th><th>{value}</th><th>{source}</th></tr></thead><tbody>\n{}</tbody></table></div></div>\n<p class=\"cap\">{note}</p>\n",
        dashboard_rows(d, zh),
        frozen_rows(d, zh),
        measured_version = latest(d),
    )
}

/// The newest measured version's row as a Markdown table. The leading
/// column HEADS the percentiles and is prose, so it takes the page's
/// language; `p50 ms` and `p95 ms` name a metric and its unit and do
/// not.
fn latest_md(d: &Value, percentile: &str) -> String {
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
        "| {percentile} | {} |\n|---|{}|\n| p50 ms | {} |\n| p95 ms | {} |\n",
        rows.iter()
            .map(|row| format!("`{}`", s(row, "metric")))
            .collect::<Vec<_>>()
            .join(" | "),
        rows.iter().map(|_| "---:").collect::<Vec<_>>().join("|"),
        cells("p50"),
        cells("p95")
    )
}

/// The README block carries the newest measured version's latency
/// only; the frozen evaluation points render whole into docs/BENCH.md
/// and both site dashboards (plan v2.21 #30, the README trim), which
/// the block's own tail links. The heading names the version the
/// numbers came from — never "the latest", which stops being true the
/// moment a release does not join the series.
fn render_readme(d: &Value, zh: bool) -> String {
    let (head, percentile, note, bench, site) = if zh {
        (
            "延迟",
            "百分位",
            "所有值均由 `contracts/bench/bench.json` 生成；本块手改会被测试拒绝。",
            "完整回放说明与逐版本系列",
            "网站完整仪表盘",
        )
    } else {
        (
            "Latency",
            "percentile",
            "Every value is generated from `contracts/bench/bench.json`; the test rejects hand edits to this block.",
            "Full replay notes and per-version series",
            "Complete website dashboard",
        )
    };
    format!(
        "### {head} · v{version}\n\n{table}\n{note}{unmeasured}{gap}[{bench}](docs/BENCH.md) · [{site}](https://codeeraser.dev{zh_path}/bench/)\n",
        version = latest(d),
        table = latest_md(d, percentile),
        unmeasured = unmeasured_note(d, zh),
        gap = join(zh),
        zh_path = if zh { "/zh" } else { "" },
    )
}

fn frozen_point<'a>(d: &'a Value, metric: &str) -> &'a Value {
    d["frozen"]
        .as_array()
        .expect("frozen")
        .iter()
        .find(|point| s(point, "metric") == metric)
        .unwrap_or_else(|| panic!("missing frozen metric {metric}"))
}

fn render_stack_fpr(d: &Value, zh: bool) -> String {
    let fourclass = frozen_point(d, "fourclass_fpr");
    let guard = frozen_point(d, "guard_fpr_per500");
    let nums = frozen::numbers(s(guard, "detail"));
    let guard_record = format!("{}/{}", nums.last().expect("guard false count"), nums[0]);
    // the joins are punctuation, and punctuation has a language: the
    // Chinese sentence takes ：；。 with no space after them
    let (title, classifier, probe, false_label, tail, colon, semi, stop) = if zh {
        (
            "误报纪律",
            "判定层",
            "写入探针",
            "误报",
            "其余规则保持 observe。每次晋级都须写入 CHANGELOG 台账。",
            "：",
            "；",
            "。",
        )
    } else {
        (
            "False-positive discipline",
            "classifier",
            "write probe",
            "false positives",
            "Every other class stays observe. Each promotion is recorded in the CHANGELOG ledger.",
            ": ",
            "; ",
            ". ",
        )
    };
    format!(
        "<div class=\"card\"><h3>{title}</h3><p>{classifier}{colon}<code>{}</code>{semi}{probe}{colon}<code>{guard_record} {false_label}</code>{stop}{tail}</p></div>\n",
        esc(&frozen::value(fourclass, zh))
    )
}

/// The numbers each rendered LINE states, sorted. The two dashboards
/// must state one set of facts; word order inside a cell belongs to
/// the language — 「600 个样本命中 0 个」 says what `0/600 flagged`
/// says with the corpus size first — so the comparison is per line
/// and over the multiset, which still catches a number dropped,
/// invented, or moved into another row.
fn numeric_facts(block: &str) -> Vec<Vec<&str>> {
    block
        .lines()
        .map(|line| {
            let mut n = frozen::numbers(line);
            n.sort_unstable();
            n
        })
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
    assert_eq!(
        numeric_facts(&en),
        numeric_facts(&zh),
        "dashboard numeric facts drifted"
    );
    gate("site/bench/index.html", "bench", en);
    gate("site/zh/bench/index.html", "bench", zh);
}

/// Every frozen point the contract carries has a Chinese sentence
/// whose numbers ARE the ledger's. `frozen::zh_value` answers None
/// both ways it can fail — no template for that metric, or a template
/// the ledger has since been reworded past — and either way the
/// English value would go out under a Chinese heading, which is the
/// defect the table exists to close. A new frozen point is refused
/// here rather than shipping half-translated.
#[test]
fn every_frozen_point_has_a_chinese_sentence() {
    let d = doc();
    let missing: Vec<&str> = d["frozen"]
        .as_array()
        .expect("frozen")
        .iter()
        .filter(|point| frozen::zh_value(point).is_none())
        .map(|point| s(point, "metric"))
        .collect();
    assert!(
        missing.is_empty(),
        "frozen points with no Chinese sentence (bench_support::frozen): {missing:?}"
    );
}

#[test]
fn readme_dashboard_blocks_match() {
    let d = doc();
    // Byte-equal to its regeneration is not the same as true: both
    // READMEs were headed "Latest-version latency · v1.3.0" all through
    // v1.3.1, green, because only the generator was ever consulted.
    names_the_release("README.md", &render_readme(&d, false));
    names_the_release("README.zh.md", &render_readme(&d, true));
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
