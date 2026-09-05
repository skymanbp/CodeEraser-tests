use super::*;

/// A revision-skewed index is REPORTED stale and left intact —
/// twice, because the defect this pins was a diagnostic that
/// repaired what it measured: the same read through `Index::open`
/// wipes the row and then answers "0 files, current".
#[test]
fn peek_reports_a_stale_key_and_wipes_nothing() {
    let dir = crate::testutil::scratch("peek");
    let db = dir.join(".ce/index.db");
    let p = Params::default();
    drop(Index::open(&db, p).expect("open"));
    let raw = Connection::open(&db).expect("raw");
    raw.execute(
        "INSERT INTO files (path, content_hash, token_count, has_tokens, owner)
             VALUES ('a.rs', 1, 0, 1, 0)",
        [],
    )
    .expect("row");
    // exactly what a sibling binary of another revision presents
    raw.execute("UPDATE meta SET v = v + 1 WHERE k = 'tokenizer_rev'", [])
        .expect("skew");
    drop(raw);
    assert_eq!(peek(&db, p).expect("peek"), (1, false), "stale, intact");
    assert_eq!(peek(&db, p).expect("peek"), (1, false), "still intact");
    std::fs::remove_dir_all(&dir).ok();
}

/// Every algorithm revision is a cache-key row: skewing any ONE of
/// them makes the index stale (the negative probe for a revision that
/// a module bumps without the key ever hearing of it — the bag tables
/// would then serve terms hashed under the old road).
#[test]
fn every_revision_row_is_part_of_the_cache_key() {
    let p = Params::default();
    for key in [
        "tokenizer_rev",
        "graph_rev",
        "struct_rev",
        "docdup_rev",
        "similar_rev",
    ] {
        let dir = crate::testutil::scratch(&format!("key-{key}"));
        let db = dir.join(".ce/index.db");
        drop(Index::open(&db, p).expect("open"));
        assert_eq!(peek(&db, p).expect("peek"), (0, true), "{key}: fresh");
        let raw = Connection::open(&db).expect("raw");
        let moved = raw
            .execute("UPDATE meta SET v = v + 1 WHERE k = ?1", (key,))
            .expect("skew");
        assert_eq!(moved, 1, "{key}: not a meta row");
        drop(raw);
        assert_eq!(peek(&db, p).expect("peek"), (0, false), "{key}: stale");
        std::fs::remove_dir_all(&dir).ok();
    }
}
