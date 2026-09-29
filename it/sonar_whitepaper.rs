//! The Sonar Cognitive Complexity whitepaper v1.7 (2023-08-29) worked
//! examples — the register contracts/fixtures/scan/whitepaper.ndjson,
//! one line per example per language (plan v2.30 step 6 widened the
//! table from the launch languages to every judged language that
//! parses; step 7b ③ made it a fixture both halves of the complexity
//! road replay). Every `want` is the margin annotation at the cited
//! page, never re-derived, and `cc` appears only where the page
//! states it; each line's `why` carries the citation and the port's
//! substitutions (a goto for the labelled continue in C, `and` / `or`
//! in Lua, an if at nesting 0 for a ternary R has not, …), and the
//! per-language headers the six Rust tables used to carry.
//!
//! This half pins two things. Each line's `events` are what the
//! emitter states for its source today — the Rust half of the
//! register, rewritten with `CE_BLESS=1` and otherwise byte-checked.
//! And each line settles through the core to its margin value — the
//! road end to end. The Haskell half (core/test/ScanEventsProps.hs)
//! folds the same events to the same margins without a parser, which
//! is the rules' own battery.

use crate::common::{self, RegisterLine, WHITEPAPER_REGISTER};
use codeeraser::scan::lang::Lang;
use codeeraser::scan::{ast, functions, metrics::events, spec};
use std::path::Path;

/// The emitter's rows for the ONE unit `line` holds, minus the row
/// column the wire adds — the register's own spelling.
fn emitted(line: &RegisterLine) -> Vec<Vec<i64>> {
    let lang =
        Lang::judged_path(Path::new(&format!("x.{}", line.ext))).expect("a judged extension");
    let sp = spec::spec(lang);
    let tree = ast::parse_lang(&line.src, lang).expect("a grammar");
    let units = functions::extract(tree.root_node(), line.src.as_bytes(), sp);
    assert_eq!(units.len(), 1, "{}: one unit per register line", line.name);
    assert_eq!(units[0].name, line.name, "a line is named after its unit");
    events::emit(units[0].node, line.src.as_bytes(), sp)
        .iter()
        .map(|e| e.row(0)[1..].to_vec())
        .collect()
}

#[test]
fn every_line_carries_the_emitters_events_for_its_source() {
    let lines = common::whitepaper_register();
    assert!(
        lines.len() >= 30,
        "{} lines: the register is the whitepaper's table",
        lines.len()
    );
    let mut drift = Vec::new();
    let mut out = Vec::new();
    for line in &lines {
        let got = emitted(line);
        if got != line.events {
            drift.push(line.name.clone());
        }
        let mut json = line.json.clone();
        json["events"] = serde_json::to_value(&got).expect("rows");
        out.push(serde_json::to_string(&json).expect("json"));
    }
    if crate::facts::blessing() {
        std::fs::write(WHITEPAPER_REGISTER, out.join("\n") + "\n").expect("rewrite the register");
        return;
    }
    assert!(
        drift.is_empty(),
        "the register's events drifted from the emitter's for {drift:?} — CE_BLESS=1 rewrites them"
    );
}

#[test]
fn every_line_settles_to_its_margin_through_the_core() {
    let lines = common::whitepaper_register();
    let files: Vec<(&str, &str)> = lines
        .iter()
        .map(|l| (l.ext.as_str(), l.src.as_str()))
        .collect();
    for (line, units) in lines.iter().zip(common::settle_many(&files)) {
        assert_eq!(units.len(), 1, "{}: one unit", line.name);
        let coc = line.settled_coc.unwrap_or(line.coc);
        assert_eq!(
            units[0].coc, coc,
            "coc of {} ({}): {}",
            line.name, line.ext, line.why
        );
        if let Some(cc) = line.cc {
            assert_eq!(
                units[0].cc, cc,
                "cc of {} ({}): {}",
                line.name, line.ext, line.why
            );
        }
    }
}
