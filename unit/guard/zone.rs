use super::*;
use crate::testutil::scratch;
use serde_json::json;

type Boundary = (
    usize,
    usize,
    usize,
    Option<(usize, usize)>,
    Option<(usize, &'static str)>,
);

/// The map's boundaries, under the compiled mirror and under a
/// baseline-authored pair: below S is silence, S+1 opens the zone,
/// both cut points are inclusive on the warn side, H itself is still
/// inside the zone, past H the position keeps climbing (the
/// hard-budget rule decides there, and its caller never asks this
/// map), and a degenerate zone — no hard line, H at or below S —
/// says nothing rather than inventing a position. The two derived
/// numbers booklet 11 quotes fall out of the table: at S = 300 /
/// H = 750 the warn arm opens at 413 lines and the ask arm at 638.
#[test]
fn the_map_partitions_the_zone_at_its_two_cut_points() {
    // lines, S, H, the baseline's pair, expected (permille, tier)
    const ROWS: &[Boundary] = &[
        (300, 300, 750, None, None),
        (301, 300, 750, None, Some((2, "observe"))),
        (412, 300, 750, None, Some((248, "observe"))),
        (413, 300, 750, None, Some((251, "warn"))),
        (637, 300, 750, None, Some((748, "warn"))),
        (638, 300, 750, None, Some((751, "ask"))),
        (750, 300, 750, None, Some((1000, "ask"))),
        (900, 300, 750, None, Some((1333, "ask"))),
        (400, 300, 0, None, None),
        (400, 750, 750, None, None),
        (400, 800, 750, None, None),
        (500, 300, 750, Some((100, 900)), Some((444, "warn"))),
        (700, 300, 750, Some((100, 900)), Some((888, "warn"))),
        (15, 10, 30, None, Some((250, "warn"))),
        (25, 10, 30, None, Some((750, "warn"))),
    ];
    for &(lines, soft, cap, tiers, want) in ROWS {
        let got = landing(lines, soft, cap, tiers).map(|l| (l.permille, l.tier));
        assert_eq!(got, want, "landing({lines}, {soft}, {cap}, {tiers:?})");
    }
    assert_eq!(
        DECLARED_TIERS,
        (250, 750),
        "the compiled mirror of the core"
    );
}

/// The baseline document's two zone keys, read the way the hook reads
/// them: a sound pair is taken, an unsound one falls back to the
/// compiled mirror rather than opening the zone on a nonsense cut,
/// and a zero softLine falls back the same way — a zero the core
/// would refuse must not open the zone on every file.
#[test]
fn the_baseline_envelope_refuses_an_unsound_pair_and_a_zero_soft_line() {
    let doc = |s: serde_json::Value, z: serde_json::Value| json!({"softLine": s, "zoneTiers": z});
    assert_eq!(
        envelope(&doc(json!(370), json!([250, 750]))),
        (Some(370), Some((250, 750)))
    );
    for bad in [
        json!([0, 750]),
        json!([800, 700]),
        json!(null),
        json!([250]),
    ] {
        assert_eq!(
            envelope(&doc(json!(370), bad.clone())).1,
            None,
            "an unsound pair is no pair: {bad}"
        );
    }
    assert_eq!(envelope(&doc(json!(0), json!([250, 750]))).0, None);
    assert_eq!(envelope(&json!({})), (None, None));
}

/// The file's own table since P4: a declared class owns its paths,
/// class 0 takes the global one, and a class glob the one dialect
/// refuses falls to the global table — never to a guess, and never
/// to a wider budget.
#[test]
fn the_class_table_owns_its_paths_and_a_refused_glob_falls_to_the_global_one() {
    let dir = scratch("guard-zone-table");
    let parse = |t: &str| toml::from_str::<Config>(t).expect(t);
    let classed = parse(concat!(
        "[thresholds]\nfile_lines_fail = 750\n\n",
        "[[rules.class]]\nname = \"gen\"\nglobs = [\"gen/\"]\n\n",
        "[rules.class.knobs]\nfile_lines_fail = 900\n"
    ));
    assert_eq!(table_for(&dir, &classed, "gen/a.rs").file_lines_fail, 900);
    assert_eq!(table_for(&dir, &classed, "src/a.rs").file_lines_fail, 750);
    // a leading '!' is refused by the one dialect (scan::globs), so
    // the class never compiles and never widens anything
    let refused = parse(
        "[[rules.class]]\nname = \"bad\"\nglobs = [\"!nope\"]\n\n[rules.class.knobs]\nfile_lines_fail = 9000\n",
    );
    assert_eq!(
        table_for(&dir, &refused, "nope/a.rs").file_lines_fail,
        Thresholds::default().file_lines_fail
    );
    let _ = std::fs::remove_dir_all(&dir);
}
