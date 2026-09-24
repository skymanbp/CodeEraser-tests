//! The scan-metric battery surface: the parse every metric and token
//! harness starts from, one measured unit, the measuring walk, and the
//! two table runners every metric battery asserts through. Split out
//! of mod.rs whole when the literal runner gained its unit-list form
//! and mod.rs had no room left under 300 lines.

use codeeraser::scan::lang::Lang;
use std::path::Path;

/// Parse `src` with the language's tree-sitter grammar — the shared
/// head of every metric/token harness (metrics, divergence, sonar,
/// dedup_core each kept a copy before the self-ratchet flagged it).
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
}

/// Every extracted unit of `src` with all five metrics.
pub fn measure_units(lang: Lang, src: &str) -> Vec<MeasuredUnit> {
    use codeeraser::scan::{functions, metrics, spec};
    let sp = spec::spec(lang);
    let tree = parse(lang, src);
    functions::extract(tree.root_node(), src.as_bytes(), sp)
        .into_iter()
        .map(|u| {
            let cog = metrics::cognitive::measure(u.node, src.as_bytes(), sp);
            MeasuredUnit {
                name: u.name,
                lines: u.end_line - u.start_line + 1,
                params: u.params,
                cc: metrics::cyclo::measure(u.node, src.as_bytes(), sp),
                coc: cog.score,
                nesting: cog.max_nesting,
            }
        })
        .collect()
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
/// scan-metric test files.
pub fn run_metric_cases(cases: &[MetricCase]) {
    for c in cases {
        let m = measure_units(c.lang, c.src);
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
    for block in table.trim().split("\n====\n") {
        let (head, src) = block.split_once('\n').expect("a header over a source");
        let [ext, want, why]: [&str; 3] = head
            .split(" @@ ")
            .collect::<Vec<_>>()
            .try_into()
            .expect("ext @@ metrics @@ why");
        let lang = Lang::from_path(Path::new(&format!("x.{ext}"))).expect("a judged extension");
        let units = measure_units(lang, src);
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
