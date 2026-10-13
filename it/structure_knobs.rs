//! The split-ROI advisory prices with the knobs `ce.toml` declares —
//! specifically P_max, whose omission made two families disagree
//! about one curve (6.1.0).
//!
//! `verdict/1` has always received `score.size_penalty_max` through
//! ceiling code 3, while `structure/1` received only the soft line
//! (12) and the hard line (13). A repo that declared the knob got the
//! declared curve in its score and the core's built-in 10 in its
//! advisory — and because both halves were internally consistent,
//! nothing anywhere disagreed out loud. The leg below is the only
//! kind of evidence that catches that class: not "the knob is in the
//! request" but "the NUMBER the user reads moves when the knob does".

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

fn fixture(name: &str, penalty_max: Option<u32>) -> std::path::PathBuf {
    fixture_with(name, (40, WIDE), penalty_max)
}

fn fixture_with(
    name: &str,
    (hard, body): (u32, &str),
    penalty_max: Option<u32>,
) -> std::path::PathBuf {
    let mut toml = format!("[thresholds]\nfile_lines_warn = 5\nfile_lines_fail = {hard}\n");
    if let Some(p) = penalty_max {
        toml.push_str(&format!("\n[score]\nsize_penalty_max = {p}\n"));
    }
    seeded(name, &toml, body)
}

/// A fresh indexed tree holding `ce.toml` and `main.py` (both end in
/// a newline, so the document splitter writes them back byte for byte).
fn seeded(name: &str, toml: &str, body: &str) -> std::path::PathBuf {
    common::indexed_doc(name, &format!("--- ce.toml\n{toml}--- main.py\n{body}"))
}

/// The best seam's benefit is linear in P_max, so declaring four
/// times the default must quadruple it — within the floor, since the
/// core prices in exact rationals and floors ONCE at the end, so
/// `floor(4y)` lands in `[4*floor(y), 4*floor(y)+3]` rather than on
/// the multiple. Comparing two runs instead of pinning one number
/// keeps the leg honest if the zone curve is ever retuned.
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

#[test]
fn a_declared_size_penalty_max_reaches_the_split_advisory() {
    let default = best_benefit(&fixture("structure-knob-default", None));
    let declared = best_benefit(&fixture("structure-knob-declared", Some(40)));
    assert!(default > 0, "the fixture must price a seam at the default");
    assert!(
        (default * 4..=default * 4 + 3).contains(&declared),
        "P_max 40 vs the core default 10: benefit is linear in P_max, so          {declared} must sit in {}..={}; outside that band the advisory          priced with a knob it never received",
        default * 4,
        default * 4 + 3
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
    let undeclared = best_benefit(&fixture_with("structure-knob-nohard", (0, &long), None));
    let explicit = best_benefit(&fixture_with("structure-knob-750", (750, &long), None));
    assert!(undeclared > 0, "the advisory must answer and price a seam");
    assert_eq!(
        undeclared, explicit,
        "no hard line must price on the core default 750"
    );
}
