//! owed.rs unit battery (plan v2.33 wave W2a, no-core rule): a pass
//! whose core could not answer stores the sites unresolved, books
//! their files and names the state; a face that reads edges refuses by
//! name; the next pass with a core settles the debt and clears the
//! name.

use super::*;
use crate::dedup::{Params, schema};
use crate::graph::store::{self, CachedSite};
use crate::scan::lang::Lang;
use rusqlite::Connection;

fn seeded() -> Connection {
    let mut conn = Connection::open_in_memory().expect("mem db");
    conn.pragma_update(None, "foreign_keys", "ON").expect("fk");
    schema::ensure_cache_key(&conn, Params::default()).expect("schema");
    conn.execute(
        "INSERT INTO files (path, content_hash, token_count, has_tokens) VALUES ('a.py', 1, 0, 1)",
        [],
    )
    .expect("file row");
    let tx = conn.transaction().expect("tx");
    store::refresh_graph(&tx, 1, "import os\nfrom . import m\n", Lang::Python).expect("phase 1");
    tx.commit().expect("commit");
    conn
}

/// The batch answer of a sweep whose core could not answer: no rows,
/// every site's file owed.
fn absent(sites: &[CachedSite]) -> Result<Resolved> {
    Ok(Resolved {
        rows: sites.iter().map(|_| Vec::new()).collect(),
        owed: Some(Owed {
            reason: "no core named".into(),
            files: sites.iter().map(|s| s.file.clone()).collect(),
        }),
    })
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).expect(sql)
}

#[test]
fn an_absent_core_books_its_files_and_the_faces_refuse_until_a_clean_pass() {
    let mut conn = seeded();
    assert!(store::ensure_resolved(&mut conn, 7, absent).expect("sweep"));
    let owed = "SELECT COUNT(*) FROM resolve_pending";
    let named = "SELECT COUNT(*) FROM meta WHERE k = 'resolve_degraded'";
    assert_eq!(
        (count(&conn, owed), count(&conn, named)),
        (1, 1),
        "the file is owed, the state named"
    );
    let refused = refuse_if_owed(&conn)
        .expect_err("a face refuses")
        .to_string();
    assert!(
        refused.contains("resolve_unavailable") && refused.contains("no core named"),
        "{refused}"
    );
    // the next run: the key stands, so phase 1.5 pays the debt with a core
    store::resolve_refreshed(&mut conn, &BTreeSet::new(), each(|_| Vec::new())).expect("settle");
    assert_eq!(
        (count(&conn, owed), count(&conn, named)),
        (0, 0),
        "a clean pass settles both"
    );
    refuse_if_owed(&conn).expect("the faces read again");
}
