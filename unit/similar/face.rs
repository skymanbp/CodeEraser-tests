use super::*;

fn row(at: &str, role: Option<bool>, score: i64, widened: bool) -> Row {
    Row {
        at: at.into(),
        key: "f/1".into(),
        nth: 0,
        role,
        score,
        hits: [1, 0, 2, 0, 0, 0],
        shape_equal: false,
        widened,
    }
}

fn report(degraded: Option<&str>) -> Report {
    Report {
        label: "a.rs:1-3 g/1".into(),
        widen: true,
        terms: 4,
        rows: vec![
            row("b.rs:1-2", Some(true), 7, false),
            row("c.rs:4-9", Some(false), 3, false),
            row("d.rs:1-1", Some(true), 2, true),
        ],
        degraded: degraded.map(String::from),
    }
}

/// The document's keys, its counts, and the five scalar columns the
/// GUI hub's generic projection keeps (alphabetical): where, what,
/// and the two judged numbers — `hits` is an array and stays out.
#[test]
fn the_document_carries_the_schema_the_counts_and_hub_friendly_rows() {
    let doc = report_json(&report(None));
    assert_eq!(doc["schema"], SCHEMA_ID);
    assert_eq!(doc["similar_rev"], SIMILAR_REV);
    assert_eq!(
        doc["query"],
        serde_json::json!({"label": "a.rs:1-3 g/1", "terms": 4, "widen": true})
    );
    assert_eq!(
        doc["counts"],
        serde_json::json!({"candidates": 3, "role": 2, "widened": 1})
    );
    assert!(doc["degraded"].is_null());
    let first = doc["candidates"][0].as_object().expect("row object");
    let mut scalars: Vec<&str> = first
        .iter()
        .filter(|(_, v)| !v.is_array() && !v.is_object())
        .map(|(k, _)| k.as_str())
        .collect();
    scalars.sort_unstable();
    assert_eq!(&scalars[..5], ["at", "key", "nth", "role", "score"]);
    assert_eq!(doc["candidates"][2]["widened"], true);
}

#[test]
fn the_console_names_every_candidate_its_role_and_the_degraded_posture() {
    let lines = console(&report(None));
    assert_eq!(lines.len(), 4);
    assert!(
        lines[0]
            .contains("4 query term(s), 3 candidate(s), 2 same-role, 1 from the associative view"),
        "{}",
        lines[0]
    );
    assert_eq!(lines[1], "  b.rs:1-2 f/1  N1 P0 C2 D0 S0 L0  same-role");
    assert_eq!(lines[2], "  c.rs:4-9 f/1  N1 P0 C2 D0 S0 L0  -");
    assert!(
        lines[3].ends_with("same-role (associative)"),
        "{}",
        lines[3]
    );

    let mut unjudged = report(Some("core unavailable"));
    for r in &mut unjudged.rows {
        r.role = None;
    }
    let lines = console(&unjudged);
    assert!(
        lines[0].contains("0 same-role") && lines[0].contains("degraded: core unavailable"),
        "{}",
        lines[0]
    );
    assert!(lines[1].ends_with("  ?"), "{}", lines[1]);
    assert_eq!(report_json(&unjudged)["degraded"], "core unavailable");
    assert_eq!(report_json(&unjudged)["counts"]["role"], 0);
}
