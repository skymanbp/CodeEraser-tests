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
