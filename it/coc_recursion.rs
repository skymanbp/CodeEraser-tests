//! The recursion increment (S3776 p.8 and Appendix B1, plan v2.23),
//! measured through the ONE road a settled cognitive value takes.
//! `common::measure_units` deliberately answers the PRE-cycle number —
//! the cycle is the core's judgment — so a battery built on it could
//! not see this rule at all; these legs go through `scan::settle`.
//!
//! The whitepaper scores no recursive example: its six worked examples
//! (sonar_whitepaper.rs) contain no recursive call, so the anchor here
//! is DERIVED, and derived two-sided so the derivation is checkable.
//! Its base is sumOfPrimes, whose 7 is read from the p.10 margin and
//! already asserted against that margin next door. Adding a self-call
//! adds no structural increment — a call is not one — so the pre-cycle
//! reading of the very same source must still be 7 while the settled
//! reading becomes 8. The +1 is the rule under test; the two readings
//! of one source are what pin it to that rule and nothing else.

use crate::common;
use codeeraser::scan;
use codeeraser::scan::lang::Lang;
use std::collections::BTreeMap;

/// Every function of a materialized tree with the cognitive value the
/// core settled on it. Keyed by name: each document below spells its
/// names apart, so a missing or doubled unit fails the comparison
/// rather than hiding inside a positional one.
fn settled(tag: &str, doc: &str) -> BTreeMap<String, u32> {
    let dir = common::tmp(tag);
    common::write_doc(&dir, doc);
    let s = scan::settle(&dir, &common::core_bin()).expect("settle");
    let units: Vec<_> = s.files.iter().flat_map(|f| &f.functions).collect();
    let out: BTreeMap<String, u32> = units
        .iter()
        .map(|f| (f.name.clone(), f.cognitive))
        .collect();
    assert_eq!(
        out.len(),
        units.len(),
        "the document spells its names apart"
    );
    out
}

fn expect(got: &BTreeMap<String, u32>, want: &[(&str, u32)], why: &str) {
    let want: BTreeMap<String, u32> = want.iter().map(|(n, v)| (n.to_string(), *v)).collect();
    assert_eq!(*got, want, "{why}");
}

/// The p.10 sumOfPrimes body, verbatim but for one line: a call to
/// itself whose value is discarded, which adds a cycle and no
/// structure. The margin scores this shape 7; the rule under test
/// makes it 8.
const ANCHOR: &str = "\
func sumOfPrimes(max int) int {
\ttotal := 0
OUT:
\tfor i := 1; i <= max; i++ {
\t\tfor j := 2; j < i; j++ {
\t\t\tif i%j == 0 {
\t\t\t\tcontinue OUT
\t\t\t}
\t\t}
\t\ttotal += i
\t}
\t_ = sumOfPrimes(0)
\treturn total
}
";

#[test]
fn the_anchor_reads_seven_before_the_cycle_and_eight_after() {
    let pre = common::measure_units(Lang::Go, ANCHOR);
    assert_eq!(pre.len(), 1);
    assert_eq!(
        pre[0].coc, 7,
        "p.10 margin: for +1, for +2, if +3, continue OUT +1 — a call \
         adds nothing, so the scored example's own number survives it"
    );
    let got = settled("coc-recursion-anchor", &format!("--- sum.go\n{ANCHOR}"));
    expect(
        &got,
        &[("sumOfPrimes", 8)],
        "p.8 / Appendix B1: a function in a recursion cycle pays one point",
    );
}

/// (fixture tag, the tree as a document, every unit with the value the
/// rule predicts for it, why the row is here). A table rather than a
/// test apiece: the rows differ only in their data, and four copies of
/// one settle-and-compare scaffold is duplication this repo prices —
/// the whitepaper battery next door carries its citations the same way.
type Case = (
    &'static str,
    &'static str,
    &'static [(&'static str, u32)],
    &'static str,
);

const CASES: &[Case] = &[
    (
        "coc-recursion-indirect",
        "--- lib.rs\n\
         fn alone() { alone() }\n\
         fn there() { back() }\n\
         fn back() { there() }\n",
        &[("alone", 1), ("there", 1), ("back", 1)],
        "indirect recursion costs exactly what direct recursion costs — \
         the half SonarSource's own java, python and javascript analysers \
         do not implement (their three repositories were read on \
         2026-08-31 and none mints a recursion increment at all). The \
         specification draws no line between the two shapes, so neither \
         do we",
    ),
    (
        "coc-recursion-membership",
        "--- lib.rs\n\
         fn red() { green() }\n\
         fn green() { blue() }\n\
         fn blue() { red() }\n\
         fn enters() { red() }\n\
         fn head() { middle() }\n\
         fn middle() { tail() }\n\
         fn tail() {}\n",
        &[
            ("red", 1),
            ("green", 1),
            ("blue", 1),
            ("enters", 0),
            ("head", 0),
            ("middle", 0),
            ("tail", 0),
        ],
        "membership, not reachability: `enters` calls straight into the \
         triangle and pays nothing, and a chain that never returns pays \
         nothing either. Without this row a \"+1 if you can reach a \
         cycle\" implementation would pass every other row here",
    ),
    (
        "coc-recursion-cross-file",
        "--- one.rs\n\
         fn first() { second() }\n\
         --- two.rs\n\
         fn second() { first() }\n",
        &[("first", 0), ("second", 0)],
        "a cycle spanning two files is not seen, and that is the stance: \
         ADR-008's fourth instalment says arcs are facts of ONE parse \
         unit. Cross-file arcs would have to be minted from names \
         (precision 0.576) or from symEdges (recall ~23%), and a wrong \
         +1 flows into the score and the size gate while a missing one \
         only leaves a point unpaid. Asserted so it cannot become true \
         by accident",
    ),
    (
        "coc-recursion-gocognit",
        "--- probe.go\n\
         package p\n\n\
         func fact(n int) int {\n\
         \tif n <= 1 {\n\t\treturn 1\n\t}\n\
         \treturn n * fact(n-1)\n\
         }\n\n\
         func a(n int) int { return b(n) }\n\n\
         func b(n int) int { return a(n) }\n\n\
         func plain(n int) int {\n\
         \tif n > 0 {\n\t\treturn n\n\t}\n\
         \treturn 0\n\
         }\n",
        &[("fact", 2), ("plain", 1), ("a", 1), ("b", 1)],
        "gocognit does exactly the direct half, and agrees value for \
         value. Measured 2026-08-31 on this same source: `gocognit -top \
         20 .` printed `2 p fact` and `1 p plain` and nothing else — it \
         omits CoC 0 functions, the already-registered divergence, so \
         the mutual `a`/`b` pair is its silence against our 1 apiece. \
         That pair is where the two implementations part",
    ),
];

#[test]
fn every_case_settles_to_the_value_the_rule_predicts() {
    for (tag, doc, want, why) in CASES {
        expect(&settled(tag, doc), want, why);
    }
}
