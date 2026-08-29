use super::*;

/// Identity in, identity out (O34): the continuous rows projected to
/// the three baseline columns, the member set verbatim, the pinned
/// soft line, and the digest exactly when one is declared — absent,
/// never null.
#[test]
fn the_pinned_baseline_is_the_requests_own_facts() {
    let rows = [[7, 0, 120, 2], [9, 1, 4, 0]];
    for digest in [Some(42u64), None] {
        let doc = baseline(&rows, &[3, 5], 300, digest);
        assert_eq!(doc["continuous"], json!([[7, 0, 120], [9, 1, 4]]));
        assert_eq!(doc["discrete"], json!([3, 5]));
        assert_eq!(doc["softLine"], json!(300));
        assert_eq!(doc.get("knobsDigest").and_then(Value::as_u64), digest);
        assert_eq!(
            doc.get("knobsDigest").is_some(),
            digest.is_some(),
            "absent, never null: {doc}"
        );
    }
}
