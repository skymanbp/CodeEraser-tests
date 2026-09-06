use super::*;
use crate::testutil::{scratch, write_tree};

/// One record as apply.rs's append_log spells it (json! sorts keys).
fn line(class: &str, path: &str, span: Option<(i64, i64)>) -> String {
    serde_json::json!({
        "schema": LOG_SCHEMA,
        "ts_ms": 1_757_088_000_123u64,
        "class": class,
        "path": path,
        "span": span,
        "provenance": "t1 twin of a.py",
        "hash": "00000000deadbeef",
        "plan": "0123456789abcdef",
    })
    .to_string()
}

/// The console stamp against a second oracle (python's datetime, UTC),
/// on the epoch, two live values, and a leap day at a century boundary
/// — the places civil-date arithmetic breaks.
#[test]
fn the_stamp_is_utc_to_the_second() {
    let got = [0u64, 1_757_000_000_000, 1_757_088_000_123, 951_782_400_000].map(utc_stamp);
    assert_eq!(
        got,
        [
            "1970-01-01T00:00:00Z",
            "2025-09-04T15:33:20Z",
            "2025-09-05T16:00:00Z",
            "2000-02-29T00:00:00Z"
        ]
    );
}

/// Every line is a record or a NAMED refusal, never a silent skip: a
/// foreign schema, a line that is not JSON and a record carrying a
/// field the writer never writes are the three refusals, each by its
/// 1-based line, and the document counts both halves.
#[test]
fn the_reader_counts_what_it_cannot_read_by_line() {
    let dir = scratch("erase-log-read");
    let foreign = line("dead_file", "x.md", None).replace(LOG_SCHEMA, "ce.erase-log/9.0.0");
    let extra = line("dead_file", "y.md", None).replacen('{', "{\"note\":1,", 1);
    let text = [
        line("t1_twin", "a.py", Some((3, 9))),
        foreign,
        "not json".to_string(),
        extra,
        line("dead_file", "b.md", None),
    ]
    .join("\n")
        + "\n";
    write_tree(&dir, &[(".ce/erase-log.ndjson", text.as_str())]);
    let l = read(&dir).expect("read");
    assert!(l.present);
    assert_eq!(l.rows.len(), 2);
    assert_eq!(
        (l.rows[0].path.as_str(), l.rows[0].span),
        ("a.py", Some((3, 9)))
    );
    let lines: Vec<usize> = l.unreadable.iter().map(|(n, _)| *n).collect();
    assert_eq!(lines, [2, 3, 4]);
    assert!(
        l.unreadable[0].1.contains("ce.erase-log/9.0.0"),
        "{}",
        l.unreadable[0].1
    );
    let doc = report_json(&l);
    assert_eq!(
        (
            doc["schema"].as_str(),
            doc["log"].as_str(),
            doc["unreadable"][1]["line"].as_u64()
        ),
        (Some(REPORT_SCHEMA), Some(LOG_REL), Some(3))
    );
    assert_eq!(
        doc["counts"],
        serde_json::json!({
            "rows": 2,
            "unreadable": 3,
            "by_class": {"dead_file": 1, "t1_twin": 1}
        })
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// No file means "nothing was applied here" — not an error, and not
/// the same fact as a trail that exists and is empty; the document
/// says which with `present`.
#[test]
fn an_absent_trail_is_absent_not_empty() {
    let dir = scratch("erase-log-absent");
    let l = read(&dir).expect("absent is not an error");
    assert!(
        !l.present && l.rows.is_empty() && l.unreadable.is_empty(),
        "{l:?}"
    );
    assert_eq!(report_json(&l)["present"], false);
    write_tree(&dir, &[(".ce/erase-log.ndjson", "")]);
    let l = read(&dir).expect("an empty file reads");
    assert!(
        l.present && l.rows.is_empty() && l.unreadable.is_empty(),
        "{l:?}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
