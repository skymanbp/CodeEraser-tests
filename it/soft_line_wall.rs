//! The relative soft line widens the comfort zone and never reaches
//! past the wall (9.0.0, size-advisory.md §A/§B). Before, S was clamped
//! to [200, 500] alone, so `file_lines_fail = 400` under a distribution
//! deriving S = 500 committed a soft line above the hard line, and a
//! 450-line file — failing `ce scan`, denied at the write — charged
//! nothing on the size axis.

use crate::common;
use codeeraser::score::Opts;

/// 300 and 450 lines: the log-space middle √(300·450) and the ratio
/// spread derive S ≈ 551, past both the hard line 400 and softMax 500.
#[test]
fn the_derived_soft_line_is_committed_under_the_hard_line() {
    let dir = common::tmp("soft-line-wall");
    std::fs::write(
        dir.join("ce.toml"),
        "[thresholds]\nfile_lines_warn = 300\nfile_lines_fail = 400\n",
    )
    .expect("ce.toml");
    for (name, lines) in [("short.py", 300), ("long.py", 450)] {
        std::fs::write(dir.join(name), "x = 1\n".repeat(lines)).expect(name);
    }
    common::build_index(&dir);
    let opts = Opts {
        db: None,
        core: common::core_bin(),
        days: None,
        floor: None,
        establish: true,
        pinned_soft: None,
        baseline: None,
    };
    let r = codeeraser::score::run(&dir, opts).expect("establish").reply;
    assert_eq!(
        r.new_baseline["softLine"],
        serde_json::json!(400),
        "S is fenced at H"
    );
    // the 450-line file past the wall charges P_max (10) — a flat P_max,
    // since S = H leaves no zone — over the two size rows: the density
    // law floor(1000 · 10 / (10 + 2))
    let size_axis = r.axes.iter().find(|[axis, _]| *axis == 0).map(|[_, c]| *c);
    assert_eq!(size_axis, Some(1000 * 10 / (10 + 2)), "axes {:?}", r.axes);
}
