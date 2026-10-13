//! A parser upgrade retains history without mixing measuring regimes.

use crate::common;
use crate::trend_rebuild::{run_all, seed_two_commits};

#[test]
fn old_parser_points_are_retained_until_remeasured() {
    let dir = common::tmp("trend-parser");
    seed_two_commits(&dir);
    let core = common::core_bin();
    let first = run_all(&dir, &core);
    assert_eq!(first.rows.len(), 2);
    let db = rusqlite::Connection::open(dir.join(".ce/index.db")).expect("db");
    let stamp: String = db
        .query_row("SELECT stamp FROM trend LIMIT 1", [], |r| r.get(0))
        .expect("measuring stamp");
    let suffix = format!(" tokenizer{}", codeeraser::dedup::tokens::TOKENIZER_REV);
    assert!(stamp.ends_with(&suffix), "the parser is part of the stamp");
    let proto = format!(" proto{} ", codeeraser::corelink::PROTO);
    assert!(stamp.contains(&proto), "so is the wire: {stamp}");
    db.execute("UPDATE trend SET stamp = replace(stamp, ?1, '')", [suffix])
        .expect("legacy CLI/core-only stamp");
    let pending = codeeraser::trend::run(&dir, None, &core, 10, Some(0)).expect("pending");
    assert_eq!((pending.rows.len(), pending.pending), (0, 2));
    let retained: i64 = db
        .query_row("SELECT count(*) FROM trend", [], |r| r.get(0))
        .expect("retained rows");
    assert_eq!(retained, 2, "pending does not destroy old measurements");
    let fresh = run_all(&dir, &core);
    assert_eq!(fresh.pending, 0);
    assert_eq!(
        fresh.rows, first.rows,
        "remeasure every point under the new stamp"
    );
}

/// A window the core could not judge is refused by name before a commit
/// of it is measured (plan v2.33 step 11b item 5): the room is the
/// package's trend_rows cap, read off the core this run talks to.
#[test]
fn a_window_over_the_cores_row_cap_is_refused_before_measuring() {
    let dir = common::tmp("trend-window-cap");
    let cap = codeeraser::tables::get().limits.caps.trend_rows;
    let refused = codeeraser::trend::run(&dir, None, &common::core_bin(), cap + 1, Some(0))
        .expect_err("over the cap");
    assert!(
        refused.to_string().starts_with(&format!(
            "trend: --commits {} over the core's trend_rows cap {cap} (0 declared knob row(s)",
            cap + 1
        )),
        "{refused}"
    );
}
