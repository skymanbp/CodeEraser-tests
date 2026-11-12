//! The split-ROI advisory prices with the lines `ce.toml` declares —
//! the soft line (structure knob 12), the hard line (13) and, for a
//! classed file, its class's lines — and with its own P_max: since
//! `[score] size_penalty_max` retired (v2.35) no key reaches knob 14,
//! which is the core's constant like the four prices 15..18. The legs
//! read the NUMBER the user reads, not the request: the benefit is
//! pinned by its derivation on the curve with P_max 10.

use crate::common;
use crate::common::bound::{StructureReport, read_bound};
use codeeraser::structure::judge;
use std::path::Path;

/// A file well past a low declared soft line, with two top-level
/// units so a seam exists to price at all.
const WIDE: &str = "def first(values):
    total = 0
    for item in values:
        total = total + item
    return total


def second(values):
    total = 0
    for item in values:
        total = total - item
    return total
";

fn fixture_with(name: &str, (hard, body): (u32, &str)) -> std::path::PathBuf {
    let toml = format!("[thresholds]\nfile_lines_warn = 5\nfile_lines_fail = {hard}\n");
    seeded(name, &toml, body)
}

/// A fresh indexed tree holding `ce.toml` and `main.py` (both end in
/// a newline, so the document splitter writes them back byte for byte).
fn seeded(name: &str, toml: &str, body: &str) -> std::path::PathBuf {
    common::indexed_doc(name, &format!("--- ce.toml\n{toml}--- main.py\n{body}"))
}

/// The best seam's benefit, candidate or exemption.
fn best_benefit(dir: &Path) -> i64 {
    let core = common::core_bin();
    let answer = judge::run(dir, None, &core, (false, None, true)).expect("structure");
    let report: StructureReport = read_bound(answer.document, "structure").expect("read");
    assert!(report.split, "split armed");
    report
        .split_candidates
        .iter()
        .map(|c| c.benefit)
        .chain(report.size_exempt.iter().map(|e| e.benefit))
        .max()
        .expect("the fixture must price at least one seam")
}

/// The declared lines reach the advisory and P_max is its own 10: WIDE
/// (12 lines, two units ending at lines 5 and 12) on the zone (5, 40]
/// prices its one seam, a 5 | 7 split, at ⌊1000·10·(7² − 0² − 2²)/35²⌋
/// = 367 — too little against its cost, so it is an exemption row.
#[test]
fn the_declared_lines_and_the_advisorys_own_p_max_price_the_seam() {
    let total = WIDE.lines().count() as i64;
    let benefit = best_benefit(&fixture_with("structure-knob-default", (40, WIDE)));
    assert_eq!(
        (benefit, seam_milli(total, 5, (5, 40))),
        (367, 367),
        "P_max 10 on (5, 40]"
    );
}

/// 9.0.0: a file in a `[[rules.class]]` with its own lines is priced on
/// its CLASS's curve — the one the verdict's size axis charges it on —
/// not the global one. WIDE is 12 lines; the class opens at 3 and walls
/// at 20, the global table at 5 and 40, and P_max is the default 10, so
/// the two curves give the one seam two different prices.
#[test]
fn a_classed_file_prices_its_seam_on_its_class_curve() {
    let toml = "[thresholds]\nfile_lines_warn = 5\nfile_lines_fail = 40\n\n\
        [[rules.class]]\nname = \"tight\"\nglobs = [\"main.py\"]\n\
        [rules.class.knobs]\nfile_lines_warn = 3\nfile_lines_fail = 20\n";
    let dir = seeded("structure-knob-classed", toml, WIDE);
    let answer =
        judge::run(&dir, None, &common::core_bin(), (false, None, true)).expect("structure");
    let report: StructureReport = read_bound(answer.document, "structure").expect("read");
    let seam = report
        .split_candidates
        .first()
        .expect("the class curve makes the seam viable");
    let total = WIDE.lines().count() as i64;
    let end = seam.after as i64;
    assert_eq!(
        seam.benefit,
        seam_milli(total, end, (3, 20)),
        "the class curve"
    );
    assert_ne!(
        seam.benefit,
        seam_milli(total, end, (5, 40)),
        "not the global one"
    );
}

/// The benefit of a seam after line `end` of a `total`-line file under
/// the zone (s, h] with P_max 10, every piece at or under h:
/// ⌊1000·(p(total) − p(end) − p(total − end))⌋, p(x) = 10·((x − s)⁺ ⁄ (h − s))².
fn seam_milli(total: i64, end: i64, (s, h): (i64, i64)) -> i64 {
    let sq = |x: i64| (x - s).max(0).pow(2);
    10 * 1000 * (sq(total) - sq(end) - sq(total - end)) / ((h - s) * (h - s))
}

/// `file_lines_fail = 0` is legal ("no hard line") and the core
/// refuses any structure knob below 1: the advisory used to die on
/// knob 13 = 0. It now leaves code 13 out — the reading the score
/// road's ceiling code 2 already had — so the core's own hard line
/// (750) prices the seam: the same benefit a tree declaring 750
/// outright gets.
#[test]
fn no_hard_line_lets_the_core_default_price_the_advisory() {
    // two ~200-line units, so the seam is worth pricing on a 750 wall
    let body: String = "    total = total + 1\n".repeat(200);
    let long = format!(
        "def first(values):\n    total = 0\n{body}    return total\n\n\n\
         def second(values):\n    total = 0\n{body}    return total\n"
    );
    let undeclared = best_benefit(&fixture_with("structure-knob-nohard", (0, &long)));
    let explicit = best_benefit(&fixture_with("structure-knob-750", (750, &long)));
    assert!(undeclared > 0, "the advisory must answer and price a seam");
    assert_eq!(
        undeclared, explicit,
        "no hard line must price on the core default 750"
    );
}
