use crate::dedup::{Params, index::Index, schema};
use crate::scan::lang::Lang;
use rusqlite::Connection;
use std::path::Path;

const SOURCE: &[u8] = include_bytes!("../../../src/dedup/schema.rs");
const TABLES: [&str; 13] = [
    "files",
    "fingerprints",
    "symbols",
    "sites",
    "edges",
    "unitsig",
    "docsegs",
    "bag",
    "df",
    "mentions",
    "mention_files",
    "result_cache",
    "resolve_pending",
];

fn count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .expect("count")
}

/// Populate every affected table, plus history the upgrade must retain.
fn old_index(db: &Path) -> Index {
    let mut idx = Index::open(db, Params::default()).expect("open");
    idx.refresh_file("ast.rs", SOURCE, Lang::Rust, Params::default(), false)
        .expect("source");
    idx.raw()
        .execute_batch(
            "INSERT INTO trend VALUES ('commit', 123, 987, 1000, '[[1,13]]', 'old');
         INSERT INTO mention_files VALUES (1, 'note.md', 1);
         INSERT INTO mentions VALUES (1, 42, 42);
         INSERT INTO edges SELECT id, 'ast.rs', '', 0, 0, 0, 0 FROM sites LIMIT 1;
         INSERT OR IGNORE INTO resolve_pending VALUES ('ast.rs');
         INSERT INTO result_cache VALUES (1, 1, 50, 7, '[]');
         INSERT INTO meta VALUES ('full_build', 1), ('resolve_key', 1), ('mention_rev', 1);
         UPDATE meta SET v = 2 WHERE k = 'tokenizer_rev';",
        )
        .expect("old cache");
    for table in TABLES {
        assert!(count(idx.raw(), table) > 0, "populated {table}");
    }
    idx
}

#[test]
fn parser_revision_refreshes_measurements_without_losing_history() {
    let dir = crate::testutil::scratch("parser-history");
    let db = dir.join("index.db");
    let old = old_index(&db);
    let epoch = schema::epoch(old.raw()).expect("epoch");
    let mut next = Index::open(&db, Params::default()).expect("upgrade");
    for table in TABLES {
        assert_eq!(count(next.raw(), table), 0, "invalidated {table}");
    }
    let history: String = next
        .raw()
        .query_row(
            "SELECT json_array(commit_hash, ts, score, scale, axes, stamp) FROM trend",
            [],
            |r| r.get(0),
        )
        .expect("history survived");
    assert_eq!(history, r#"["commit",123,987,1000,"[[1,13]]","old"]"#,);
    assert_ne!(schema::epoch(next.raw()).expect("new epoch"), epoch);
    assert!(!schema::full_build_done(next.raw()).expect("incomplete"));
    assert!(
        schema::mark_full_build(old.raw(), epoch).is_err(),
        "old run fenced"
    );
    assert!(
        next.refresh_file("ast.rs", SOURCE, Lang::Rust, Params::default(), false)
            .expect("unchanged bytes remeasured")
    );
    let rows = count(next.raw(), "bag");
    let reopened = Index::open(&db, Params::default()).expect("same key");
    assert_eq!(count(reopened.raw(), "bag"), rows, "same key retains rows");
    assert!(
        !next
            .refresh_file("ast.rs", SOURCE, Lang::Rust, Params::default(), false)
            .expect("warm")
    );
}

#[test]
fn a_failed_parser_invalidation_rolls_back_rows_and_revision() {
    let dir = crate::testutil::scratch("parser-rollback");
    let db = dir.join("index.db");
    let old = old_index(&db);
    old.raw()
        .execute_batch(
            "CREATE TRIGGER refuse_parser_reset BEFORE DELETE ON files
         BEGIN SELECT RAISE(ABORT, 'injected migration failure'); END;",
        )
        .expect("failure at last table");
    assert!(
        Index::open(&db, Params::default()).is_err(),
        "failure propagates"
    );
    assert!(!super::current(old.raw()).expect("old revision"));
    for table in TABLES.into_iter().chain(["trend"]) {
        assert!(count(old.raw(), table) > 0, "rollback restored {table}");
    }
    assert!(schema::full_build_done(old.raw()).expect("old stamp"));
}
