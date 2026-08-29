use super::allow_claim;

/// The two prior parsers' own witnesses (a why-bearing marker, a
/// bare one, an empty why) plus every seam where they disagreed
/// with each other or with this grammar: text between marker and
/// `--`, a why on the next line, tabs, `--why` with no blank, a
/// second marker carrying the why the first lacks.
#[test]
fn only_a_why_bearing_marker_claims() {
    let claims = [
        "# ce:allow(t) -- loader-invoked",
        "# ce:allow(t)-- loader-invoked",
        "# ce:allow(t)\t--\twhy",
        "# ce:allow(t)\n# ce:allow(t) -- second carries it",
    ];
    let nothing = [
        "# ce:allow(t)",
        "# ce:allow(t) -- ",
        "# ce:allow(t) -- \nwhy",
        "# ce:allow(t) --\nwhy on the next line",
        "# ce:allow(t) --why",
        "# ce:allow(t)--why",
        "# ce:allow(t) -->see below",
        "# ce:allow(t) note -- why",
        "# ce:allow(t)\tnote\t--\twhy",
        "# ce:allow(other) -- why",
    ];
    for text in claims {
        assert!(allow_claim(text, "ce:allow(t)"), "{text:?}");
    }
    for text in nothing {
        assert!(!allow_claim(text, "ce:allow(t)"), "{text:?}");
    }
}
