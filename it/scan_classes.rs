//! The rulepack's scan half end to end (plan v2.13 ① P3, 3.2.0):
//! one fixture, one class, the same 60-line function twice — inside
//! the class its fn line sits at 80 and the row grades clean; on the
//! global table it warns at 50, and the finding names the line it was
//! measured against. The classed run rides the whole judged road
//! (Classes → rowClasses/gradeOverrides → the core → findings_from),
//! and analyze_judged's pinned-mirror ensure proves the local
//! per-class evaluate equal to the wire on every such run.

use crate::common;
use codeeraser::scan;

/// `depth` nested ifs: cognitive complexity 1+2+..+depth, the
/// whitepaper's nesting increment spelled as a fixture rather than
/// asserted from memory — the test reads the measured value back.
fn tangled(depth: usize) -> String {
    let pad = |n: usize| "    ".repeat(n + 1);
    let mut s = String::from("pub fn tangled(a: i32) -> i32 {\n");
    for i in 0..depth {
        s.push_str(&format!("{}if a > {i} {{\n", pad(i)));
    }
    s.push_str(&format!("{}return 1;\n", pad(depth)));
    for i in (0..depth).rev() {
        s.push_str(&format!("{}}}\n", pad(i)));
    }
    s + "    0\n}\n"
}

/// One tree, one config, the cognitive findings it produces as
/// (file, value, limit, is_fail) plus the run's fail bit.
fn coc_under(toml: &str, name: &str) -> (bool, Vec<(String, usize, usize, bool)>) {
    let fx = common::fixture(
        name,
        &[
            ("ce.toml", toml),
            ("gen/a.rs", &tangled(6)),
            ("src/b.rs", &tangled(6)),
        ],
    );
    let (_files, findings, _summary, fail, _failed) =
        scan::analyze_judged(&fx.dir, &common::gates::core_bin()).expect("judged");
    let rows = findings
        .iter()
        .filter(|f| f.rule == "cognitive")
        .map(|f| {
            (
                f.file.clone(),
                f.value,
                f.threshold,
                f.level == scan::report::Level::Fail,
            )
        })
        .collect();
    (fail, rows)
}

/// The complexity wall (plan v2.24). The load-bearing leg is the
/// NEGATIVE one: `cognitive_fail` ships 0, so the same tangled
/// function that a declared line fails must stay a plain warn under
/// the shipped table — a knob that failed without being asked to
/// would have changed every existing repo's `ce scan` exit code the
/// day it landed. The class leg then shows one tree holding two
/// walls, which a global-only reading cannot produce.
#[test]
fn the_complexity_wall_bites_only_where_a_line_was_declared() {
    let (fail, rows) = coc_under("", "coc-wall-off");
    assert!(!fail, "the shipped table has no complexity hard line");
    assert!(
        rows.iter().all(|r| !r.3) && rows.iter().all(|r| r.1 == 21 && r.2 == 15),
        "both files warn at the 15 line with a measured 21: {rows:?}"
    );

    let (fail, rows) = coc_under("[thresholds]\ncognitive_fail = 20\n", "coc-wall-on");
    assert!(fail, "a declared line fails the same tree");
    assert!(
        rows.iter().all(|r| r.3 && r.2 == 20),
        "every row now fails and names the declared line: {rows:?}"
    );

    let classed = "[thresholds]\ncognitive_fail = 30\n\n[[rules.class]]\n\
         name = \"gen\"\nglobs = [\"gen/**\"]\n[rules.class.knobs]\ncognitive_fail = 20\n";
    let (fail, rows) = coc_under(classed, "coc-wall-class");
    assert!(fail, "the class's own wall bites");
    let failed: Vec<&str> = rows.iter().filter(|r| r.3).map(|r| r.0.as_str()).collect();
    assert_eq!(
        failed,
        vec!["gen/a.rs"],
        "only the classed file crosses its 20 line; the global 30 spares its twin: {rows:?}"
    );
}

#[test]
fn a_class_moves_its_files_fn_ladder_and_the_mirror_holds() {
    let body: String = (0..58).map(|i| format!("    let v{i} = {i};\n")).collect();
    let long_fn = format!("fn long() {{\n{body}}}\n");
    let fx = common::fixture(
        "scan-classes",
        &[
            // the class's own hard line rides too: a warn at 80 over
            // the inherited fail 75 is exactly the ladder the load
            // throat refuses (C8) — a classed ladder must climb
            (
                "ce.toml",
                "[[rules.class]]\nname = \"gen\"\nglobs = [\"gen/**\"]\n[rules.class.knobs]\nfn_lines_warn = 80\nfn_lines_fail = 90\n",
            ),
            ("gen/a.rs", &long_fn),
            ("src/b.rs", &long_fn),
        ],
    );
    let core = common::gates::core_bin();
    let (files, findings, _summary, fail, _failed) =
        scan::analyze_judged(&fx.dir, &core).expect("judged");
    assert_eq!(files.len(), 2, "both files measured");
    assert!(!fail, "no hard line breached");
    let fn_lines: Vec<(&str, usize, usize)> = findings
        .iter()
        .filter(|f| f.rule == "fn-lines")
        .map(|f| (f.file.as_str(), f.value, f.threshold))
        .collect();
    assert_eq!(
        fn_lines,
        vec![("src/b.rs", 60, 50)],
        "the classed file's 60-line fn is clean under its 80 line; the global one warns at 50"
    );
}
