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
