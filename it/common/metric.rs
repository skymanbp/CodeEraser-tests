//! The scan-metric battery surface: the parse every token harness
//! starts from, one measured unit, the measuring road and the two
//! table runners every metric battery asserts through. Split out of
//! mod.rs whole when the literal runner gained its unit-list form and
//! mod.rs had no room left under 300 lines.
//!
//! The measuring road is the product's (plan v2.30 step 7b ③): a
//! battery's sources are written out as ONE tree and settled through
//! the core, because the three complexity numbers are derived there
//! from each unit's event stream and no walker on this side answers
//! them any more — so a row's number is the number `ce scan` prints,
//! the recursion charge included. One settle per table rather than
//! per row: a core process per source would cost the batteries their
//! speed for nothing, and a file's units come back in extraction
//! order either way. The whitepaper register the Haskell battery
//! replays (contracts/fixtures/scan/whitepaper.ndjson) is read here
//! too, so the recursion battery can find its anchor's line.

use codeeraser::scan::lang::Lang;
use codeeraser::scan::metrics::FnMetrics;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Parse `src` with the language's tree-sitter grammar — the shared
/// head of every token harness (dedup_core and the register's
/// emitter leg start here; the metric batteries no longer parse for
/// themselves).
pub fn parse(lang: Lang, src: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&lang.grammar().expect("grammar"))
        .expect("set_language");
    parser.parse(src, None).expect("parse")
}

/// One measured unit — every scan-metric battery's assertion currency
/// (metrics / divergence-stances / sonar-whitepaper each kept its own
/// struct + measure loop until the self-ratchet flagged the trio).
pub struct MeasuredUnit {
    pub name: String,
    pub lines: usize,
    pub params: usize,
    pub cc: u32,
    pub coc: u32,
    pub nesting: u32,
}

impl MeasuredUnit {
    /// One metric by its battery key — the one reading both battery
    /// runners below assert through.
    pub fn metric(&self, key: &str) -> u32 {
        match key {
            "cc" => self.cc,
            "coc" => self.coc,
            "nesting" => self.nesting,
            "lines" => self.lines as u32,
            "params" => self.params as u32,
            other => panic!("unknown metric key {other}"),
        }
    }

    fn of(f: &FnMetrics) -> Self {
        MeasuredUnit {
            name: f.name.clone(),
            lines: f.lines,
            params: f.params,
            cc: f.cyclomatic,
            coc: f.cognitive,
            nesting: f.max_nesting,
        }
    }
}

/// The extension a battery row's language is written out under: the
/// judged code languages, one spelling each (a literal table names
/// its own extension).
pub fn ext_of(lang: Lang) -> &'static str {
    match lang {
        Lang::Python => "py",
        Lang::TypeScript => "ts",
        Lang::Tsx => "tsx",
        Lang::Rust => "rs",
        Lang::Go => "go",
        Lang::Haskell => "hs",
        Lang::C => "c",
        Lang::Cpp => "cpp",
        Lang::Java => "java",
        Lang::Lua => "lua",
        Lang::R => "R",
        other => panic!("{other:?} measures no functions"),
    }
}

/// Every `(ext, src)` settled as its own file of one fresh tree,
/// through the core; each source's units in extraction order, keyed
/// back by the file it was written as.
pub fn settle_many(files: &[(&str, &str)]) -> Vec<Vec<MeasuredUnit>> {
    static TREES: AtomicUsize = AtomicUsize::new(0);
    let dir = super::tmp(&format!("metric-{}", TREES.fetch_add(1, Ordering::Relaxed)));
    for (i, (ext, src)) in files.iter().enumerate() {
        let name = format!("u{i}.{ext}");
        Lang::judged_path(Path::new(&name)).expect("a judged extension");
        std::fs::write(dir.join(name), src).expect("write a battery source");
    }
    let settled =
        codeeraser::scan::settle(&dir, &super::core_bin()).expect("settle through the core");
    let mut by_stem: BTreeMap<String, Vec<MeasuredUnit>> = settled
        .files
        .iter()
        .map(|f| {
            let stem = Path::new(&f.path)
                .file_stem()
                .expect("a file")
                .to_string_lossy()
                .into_owned();
            (stem, f.functions.iter().map(MeasuredUnit::of).collect())
        })
        .collect();
    (0..files.len())
        .map(|i| {
            by_stem
                .remove(&format!("u{i}"))
                .expect("every source is a file of the settled tree")
        })
        .collect()
}

/// Every extracted unit of `src` with all five metrics, measured the
/// way the product measures.
pub fn measure_units(lang: Lang, src: &str) -> Vec<MeasuredUnit> {
    settle_many(&[(ext_of(lang), src)]).remove(0)
}

/// One table row of a metric battery: source, expected unit count and
/// the (unit index, metric, expected, why) checks. The why strings
/// carry the whitepaper citations / stance records — the table IS the
/// register.
pub struct MetricCase {
    pub lang: Lang,
    pub src: &'static str,
    pub fns: usize,
    pub checks: &'static [(usize, &'static str, u32, &'static str)],
}

/// Run a metric battery table — ONE assertion loop for the three
/// scan-metric test files, ONE settle for the table.
pub fn run_metric_cases(cases: &[MetricCase]) {
    let files: Vec<(&str, &str)> = cases.iter().map(|c| (ext_of(c.lang), c.src)).collect();
    for (c, m) in cases.iter().zip(settle_many(&files)) {
        assert_eq!(m.len(), c.fns, "unit count for:\n{}", c.src);
        for &(i, key, want, why) in c.checks {
            assert_eq!(m[i].metric(key), want, "{key}[{i}]: {why}");
        }
    }
}

/// Run a metric battery written as ONE literal: blocks separated by a
/// `====` line, each a header `ext @@ key=value … @@ why` over the
/// source of exactly one unit (the keys are MeasuredUnit::metric's), so
/// a row asserts only the numbers its source states — or a header
/// `ext @@ units=name/params, … @@ why` over a source of many units,
/// which pins every unit the source makes and the parameters each
/// reads, in source order. One literal rather than a slice of typed
/// rows: rows of any one tuple or struct shape repeat every dozen
/// tokens, and the clone gate reads such a table as clones of itself
/// (coc_c.rs, the first table written this way). The unit list rides
/// the same literal for the same reason: a unit-list test apiece was
/// the file skeleton the gate paired across the C and Java batteries.
pub fn assert_metric_table(table: &str) {
    let blocks: Vec<([&str; 3], &str)> = table
        .trim()
        .split("\n====\n")
        .map(|block| {
            let (head, src) = block.split_once('\n').expect("a header over a source");
            let head: [&str; 3] = head
                .split(" @@ ")
                .collect::<Vec<_>>()
                .try_into()
                .expect("ext @@ metrics @@ why");
            (head, src)
        })
        .collect();
    let files: Vec<(&str, &str)> = blocks.iter().map(|([ext, ..], src)| (*ext, *src)).collect();
    for (([_, want, why], src), units) in blocks.iter().zip(settle_many(&files)) {
        if let Some(list) = want.strip_prefix("units=") {
            let seen: Vec<_> = units
                .iter()
                .map(|u| format!("{}/{}", u.name, u.params))
                .collect();
            assert_eq!(seen.join(", "), list, "units: {why}\n--- source ---\n{src}");
            continue;
        }
        assert_eq!(units.len(), 1, "one unit for:\n{src}");
        for pair in want.split(' ') {
            let (key, value) = pair.split_once('=').expect("key=value");
            assert_eq!(
                units[0].metric(key).to_string(),
                value,
                "{key}: {why}\n--- source ---\n{src}"
            );
        }
    }
}

/// The whitepaper register (plan v2.30 step 7b ③), relative to the
/// cli/ cwd cargo runs the suite in: one JSON object per line.
pub const WHITEPAPER_REGISTER: &str = "../contracts/fixtures/scan/whitepaper.ndjson";

/// One register line: the example's language and unit name, its
/// margin values (`cc` only where the page states it), the value the
/// road ends at where it differs from the fold's (`settled`: the
/// recursion charge the core adds from the call table, which no event
/// carries — the anchor line alone), the source, the emitter's events
/// for it — and the line as parsed, for the bless that rewrites the
/// events.
pub struct RegisterLine {
    pub ext: String,
    pub name: String,
    pub why: String,
    pub coc: u32,
    pub cc: Option<u32>,
    pub settled_coc: Option<u32>,
    pub src: String,
    pub events: Vec<Vec<i64>>,
    pub json: Value,
}

/// Every line of the register, each checked to be in the canonical
/// form serde_json writes — keys sorted, no spaces — so a hand edit
/// that would drift the bytes is caught before a bless rewrites them.
pub fn whitepaper_register() -> Vec<RegisterLine> {
    let raw = std::fs::read_to_string(WHITEPAPER_REGISTER).expect("the whitepaper register");
    raw.lines()
        .enumerate()
        .map(|(n, line)| {
            let json: Value = serde_json::from_str(line)
                .unwrap_or_else(|e| panic!("register line {}: {e}", n + 1));
            assert_eq!(
                serde_json::to_string(&json).expect("json"),
                line,
                "register line {} is not in the canonical form (CE_BLESS=1 rewrites the file)",
                n + 1
            );
            let text = |k: &str| {
                json[k]
                    .as_str()
                    .unwrap_or_else(|| panic!("register line {}: `{k}` is text", n + 1))
                    .to_owned()
            };
            let count = |v: &Value| u32::try_from(v.as_u64().expect("a count")).expect("fits");
            RegisterLine {
                ext: text("ext"),
                name: text("name"),
                why: text("why"),
                coc: count(&json["want"]["coc"]),
                cc: json["want"].get("cc").map(count),
                settled_coc: json.get("settled").and_then(|s| s.get("coc")).map(count),
                src: text("src"),
                events: serde_json::from_value(json["events"].clone()).expect("event rows"),
                json,
            }
        })
        .collect()
}
