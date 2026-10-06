use super::*;

/// The schema this side assembles by is the one the core judges by:
/// the golden reply that asked for the echo carries CE.Query.Schema
/// row for row (contracts/fixtures/query/golden.ndjson, id 1).
#[test]
fn the_schema_rows_are_the_cores_echo() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../contracts/fixtures/query/golden.ndjson"
    );
    let text = std::fs::read_to_string(path).expect("the query goldens");
    let echoed: Vec<Vec<i64>> = text
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .find_map(|d| {
            d["schema"]
                .as_array()
                .map(|_| serde_json::from_value(d["schema"].clone()).unwrap())
        })
        .expect("one golden reply echoes the schema");
    assert_eq!(schema_rows(), echoed);
    assert_eq!(schema_rows().len(), 27);
}

#[test]
fn every_schema_row_answers_by_name_and_by_code() {
    for (i, row) in schema_rows().iter().enumerate() {
        let p = pred_of(i as u32).expect("a code");
        assert_eq!((p.code as usize, row[1] as usize), (i, p.sorts.len()));
        assert_eq!(pred(p.name).map(|q| q.code), Some(p.code));
    }
    assert_eq!(pred("ref").map(|p| p.sorts.clone()), Some(vec![0, 0, 4, 3]));
    assert_eq!(
        pred_of(26).map(|p| (p.name, p.sorts.clone())),
        Some(("set", vec![5, 0]))
    );
    assert_eq!(
        pred(SET_PRED).map(|p| p.sorts[SET_ARG]),
        Some(sort_code("set"))
    );
    assert!(pred("nope").is_none() && pred_of(27).is_none());
    assert_eq!(IDB_FLOOR, 1000);
}

#[test]
fn the_sorts_round_trip_and_the_open_sort_reads_open() {
    for (code, name) in ["node", "dir", "unit", "int", "sym", "set"]
        .iter()
        .enumerate()
    {
        assert_eq!(sort_name(code as i64), *name);
        assert_eq!(sort_code(name), code as i64);
    }
    assert_eq!(
        (sort_name(-1), sort_name(6), sort_name(99)),
        ("open", "open", "open")
    );
}
