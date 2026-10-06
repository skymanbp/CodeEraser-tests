//! The W7 differential's erase leg (plan v2.33 W7; mounted inside the
//! frozen erase face): the paths, each row's provenance and each file's
//! diff (a `diff` reference keyed by the file's first row) spelled by
//! the core and by the frozen `PlanStrings`. The face sends one diff
//! slot per plan row (the file's text at its first row, null at every
//! other), as `Diffs::by_row` lays it out; the keys are rows of the
//! plan, as `render::file_diffs` makes them.

use super::super::binder::Lists;
use super::super::kit::{Leg, leg};
use super::{Diffs, PlanStrings};
use serde_json::{Value, json};

/// Each file's diff keyed by a row below `rows` (a key now and then
/// twice: the first one holds).
fn diffs(leg: &mut Leg, rows: usize) -> Vec<(usize, String)> {
    let n = if rows == 0 { 0 } else { leg.rng.below(4) };
    (0..n)
        .map(|_| (leg.rng.below(rows), leg.words(4).join("\n")))
        .collect()
}

/// The slots as the face sends them.
fn by_row(d: &[(usize, String)], rows: usize) -> Vec<Value> {
    let mut out = vec![Value::Null; rows];
    for (i, text) in d.iter().rev() {
        out[*i] = json!(text);
    }
    out
}

leg!(erase_spells_as_the_frozen_face, "erase", 0x7713, |leg| {
    let provenance = leg.words(7);
    let paths = leg.paths(0, 6);
    let rows = provenance.len();
    let d = diffs(leg, rows);
    let sent = json!({"path": paths, "provenance": provenance, "diff": by_row(&d, rows)});
    let drawn = [
        ("diff", vec![rows, rows]),
        ("provenance", vec![rows]),
        ("path", vec![paths.len()]),
    ];
    let diffs = Diffs(d);
    let lists = Lists(vec![("path", paths), ("provenance", provenance)]);
    let frozen = PlanStrings {
        lists,
        diffs: &diffs,
    };
    leg.spell_drawn("erase", &sent, &frozen, &drawn);
});
