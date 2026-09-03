//! The bench dashboard's three rendered faces are GENERATED from
//! contracts/bench/bench.json and byte-gated here (the docs_gate
//! pattern: CE_BLESS=1 rewrites, a plain run compares) — a
//! hand-edited surface number re-opens the D7 defect class the plan
//! forbids ("禁手填"). docs/BENCH.md is written whole; the two site
//! pages own only the block between their bench markers.

use crate::bench_support::render::{
    NoRow, VERSION_BEARING_SURFACES, doc, latest, measured, names_the_release, no_row_sentence,
    rows_with, s, series_note, unmeasured_note,
};
use crate::common;
use serde_json::Value;

/// The prose above the series: what produced these numbers, and what a
/// reader may and may not conclude from them.
const HEADER: &str = "# Benchmarks — replayed, never hand-filled\n\n\
         > Generated from [contracts/bench/bench.json](../contracts/bench/bench.json) by\n\
         > `cli/tests/it/bench_render.rs` (`CE_BLESS=1` to regenerate). Every series row was\n\
         > measured by `cli/tests/it/bench.rs` (`bench_append`, for a checkout) or by\n\
         > `cli/tests/it/bench_backfill.rs` (`bench_backfill`, per release tag — the tag’s\n\
         > submodules seated, its OWN binaries, release builds only, fresh index per cold\n\
         > run). Frozen points carry their sealed-ledger source; points that cannot\n\
         > honestly become a per-version series say why in their epoch clause.\n\
         >\n\
         > Six of the seven metrics run against this repository. `hook_probe` does\n\
         > not: it times `ce probe --hook` against a seeded two-file fixture rebuilt\n\
         > identically for every tag, so the write-time probe stays comparable across\n\
         > versions instead of tracking the tree's growth.\n\
         >\n\
         > The whole series is ONE machine — the host column repeats because it never\n\
         > varies. That is a feature for the only comparison this table makes\n\
         > (version-over-version on constant hardware) and a warning about the one it\n\
         > cannot: none of these milliseconds transfer to other hardware, and no CI\n\
         > runner replays them (PERF-BUDGET.md opens with why a shared runner cannot\n\
         > host a latency budget).\n\
         >\n\
         > One machine is not one machine-state. v1.2.0's row was first taken on\n\
         > 2026-08-26; replaying that same tag four days later — its own tree, its own\n\
         > binaries — moved every one of its seven metrics, from 11 % faster to 12 %\n\
         > slower, which is wider than most deltas a reader would try to read out of\n\
         > this table. So the series is replayed WHOLE, in one sitting, whenever a\n\
         > release joins it, and a tag whose minutes were disturbed is measured again\n\
         > alone on that same day: every row shares one measured date (a test below\n\
         > holds that line), and rows carrying different dates are not comparable.\n\
         >\n\
         > A release joins only when there is something new to measure. One that ships\n\
         > the same `cli/src` and `core/app` as its predecessor gets no row of its own:\n\
         > replaying the whole series to add a duplicate measurement would publish that\n\
         > drift under a new version number. So every surface printing these numbers\n\
         > beside a version names the version MEASURED — never \"the latest\" — and says\n\
         > so when the shipped release is a different one. Both row writers apply the\n\
         > rule: the backfill names every tag it turns away, and a checkout that\n\
         > brings nothing new is refused rather than measured a second time. So the\n\
         > table cannot gain a row the rule forbids, and a release that earns one\n\
         > says which of the two reasons it has none yet.\n";

/// The series table's own heading, split from the prose above it so the
/// derived sentence about an unmeasured release can sit between them.
const SERIES: &str = "\n## Latency series (self repository)\n\n\
         | version | metric | p50 ms | p95 ms | n | host | measured |\n|---|---|---|---|---|---|---|\n";

/// docs/BENCH.md, whole file.
fn render_md(d: &Value) -> String {
    let mut out = String::from(HEADER);
    out.push_str(&format!("\n{}\n", series_note(d)));
    out.push_str(SERIES);
    out.push_str(&rows_with(d, "rows", |r| {
        format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            s(r, "version"),
            s(r, "metric"),
            r["p50"],
            r["p95"],
            r["n"],
            s(r, "host"),
            measured(r),
        )
    }));
    out.push_str("\n## Frozen evaluation points\n\n| metric | value | source |\n|---|---|---|\n");
    out.push_str(&rows_with(d, "frozen", |f| {
        format!(
            "| {} | {} | {} |\n",
            s(f, "metric"),
            s(f, "value"),
            s(f, "source"),
        )
    }));
    out.push_str("\nPer-point detail, freeze dates and epoch clauses live in the JSON itself.\n");
    out
}

fn latest_p50(d: &Value, metric: &str) -> String {
    let v = latest(d);
    d["rows"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|r| s(r, "version") == v && s(r, "metric") == metric)
        .map(|r| r["p50"].to_string())
        .unwrap_or_else(|| "—".into())
}

/// A frozen point's value BY LOOKUP — the first draft hard-coded the
/// two frozen chips in this renderer, which is precisely the
/// hand-filled surface the whole batch exists to forbid.
fn frozen_value(d: &Value, metric: &str) -> String {
    d["frozen"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|f| s(f, "metric") == metric)
        .map(|f| s(f, "value").to_string())
        .unwrap_or_else(|| "—".into())
}

/// The site's bench block (between the page's bench markers). One
/// renderer, two languages — the labels differ, the numbers cannot.
/// `v` is the newest version the series MEASURED, which is not always
/// the version the project ships; when they part, the caption says so
/// rather than leaving a reader to wonder whether the page is stale.
fn render_site(d: &Value, zh: bool) -> String {
    let v = latest(d);
    // `install metric`: the key is a metric's full name, which the sheet
    // keeps in its own case and gives one column width; the unit carries
    // its own leading space, so a unitless value ends the cell itself.
    let chip = |label: &str, val: String, unit: &str| {
        format!(
            "<div class=\"install metric\"><span class=\"k\">{label}</span><code>{val}{unit}</code></div>\n"
        )
    };
    // The corpus belongs in the label. Five of these chips measure this
    // repository; `hook_probe` measures the two-file fixture bench.rs
    // rebuilds per tag, and a chip that omits that reads as a whole-tree
    // figure — the same omission the BENCH header and both READMEs were
    // corrected for, which has to reach every surface quoting the number.
    // So does the metric's own name where the label does not spell it:
    // `dedup_warm` is the dashboard's row, and "incremental index" alone
    // gave a reader nothing to find that row by.
    let (h, note, hook, scan, dedup, fpr, prec) = if zh {
        (
            "实测",
            "每个数字由回放产生（cli/tests/it/bench.rs 与 bench_backfill.rs），单源 contracts/bench/bench.json——绝不手填。",
            "hook 探针 p50（两文件夹具）",
            "全仓扫描 p50",
            "增量索引 p50（暖跑，dedup_warm）",
            "守卫误报（人裁 630 事件）",
            "docdup 判准（冻结黄金集）",
        )
    } else {
        (
            "Measured",
            "Every number is produced by replay (cli/tests/it/bench.rs and bench_backfill.rs) from one source, contracts/bench/bench.json — never hand-filled.",
            "hook probe p50 (two-file fixture)",
            "full scan p50",
            "incremental index p50 (warm, dedup_warm)",
            "guard false positives (630 arbitrated events)",
            "docdup precision (frozen golden set)",
        )
    };
    format!(
        "<h2>{h} · v{v}</h2>\n<div class=\"installs\">\n{}{}{}{}{}</div>\n<p class=\"cap\">{note}{}</p>\n",
        chip(hook, latest_p50(d, "hook_probe"), " ms"),
        chip(scan, latest_p50(d, "scan"), " ms"),
        chip(dedup, latest_p50(d, "dedup_warm"), " ms"),
        chip(fpr, frozen_value(d, "guard_fpr_per500"), ""),
        chip(prec, frozen_value(d, "docdup_d3_precision"), ""),
        unmeasured_note(d, zh),
    )
}

/// One page's bench block through the shared splicer (facts::block).
fn gate_site(rel: &str, zh: bool) {
    crate::facts::block::assert_current(rel, "bench", &render_site(&doc(), zh));
}

/// Every surface that prints these numbers beside a version names the
/// release this build IS — asked of the COMMITTED files, not of the
/// generators, because generator coverage was five of the seven for a
/// whole release: the two dashboard pages, the most detailed public
/// latency surface, showed 1.4.0 as the newest row while the site
/// shipped 1.4.1 and said nothing about the gap.
///
/// It reads what the blessing tests WRITE, and those run beside it, so
/// under `CE_BLESS=1` it would race them and fail on a file that is
/// about to become correct. A bless is a local write mode — CI never
/// blesses (`facts::blessing` refuses it there) — so this asks its
/// question on the plain run that follows, where nothing is writing.
#[test]
fn every_version_bearing_surface_names_the_release() {
    if crate::facts::blessing() {
        return;
    }
    let release = env!("CARGO_PKG_VERSION");
    for rel in VERSION_BEARING_SURFACES {
        let text = std::fs::read_to_string(common::repo_root().join(rel)).expect(rel);
        assert!(
            text.contains(&format!("v{release}")),
            "{rel} prints latency numbers beside a version and never names \
             v{release}, the release this build is"
        );
    }
}

/// None of the derived sentences reads like a broken build. Each is a
/// line-continued literal, and a lost `\` leaves the source
/// indentation inside the string — two of these four shipped that way
/// while every byte gate stayed green, because each of the seven
/// surfaces is compared with the one generator carrying the same gap.
#[test]
fn no_generated_sentence_carries_a_lost_continuation() {
    let d = doc();
    let mut said = vec![series_note(&d)];
    for why in [NoRow::NothingNew, NoRow::ReplayOwed] {
        for zh in [false, true] {
            said.push(no_row_sentence(&why, "9.9.9", zh));
        }
    }
    // the chips too: a unitless value used to end its cell in a space
    said.extend([render_site(&d, false), render_site(&d, true)]);
    for line in &said {
        assert!(
            !line.contains("  ") && !line.contains(" </code>"),
            "a run of spaces, or a space before a closing tag, in text this build publishes: {line:?}"
        );
    }
}

#[test]
fn bench_md_matches_its_regeneration() {
    common::assert_matches_golden(&render_md(&doc()), std::path::Path::new("../docs/BENCH.md"));
}

#[test]
fn site_bench_blocks_match() {
    gate_site("site/index.html", false);
    gate_site("site/zh/index.html", true);
}

/// The byte gates above prove each surface equals its regeneration —
/// they cannot notice that the regeneration itself went stale.
#[test]
fn the_generated_bench_surfaces_name_the_shipped_release() {
    let d = doc();
    names_the_release("docs/BENCH.md", &render_md(&d));
    names_the_release("site/index.html", &render_site(&d, false));
    names_the_release("site/zh/index.html", &render_site(&d, true));
}

/// HEADER promises one measured date for every row. A sitting that
/// crosses UTC midnight breaks that silently — 28 rows once shipped so.
#[test]
fn every_row_shares_one_measured_date() {
    let d = doc();
    let dates: std::collections::BTreeSet<&str> = d["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|r| s(r, "measured_at"))
        .collect();
    assert_eq!(
        dates.len(),
        1,
        "bench.json rows carry {dates:?}: measure the odd tags again"
    );
}
