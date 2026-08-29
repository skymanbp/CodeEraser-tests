use super::clean;

/// Comments and trailing commas go; string contents — including
/// a literal that LOOKS like a trailing comma or comment — stay.
#[test]
fn strips_jsonc_but_never_string_contents() {
    let dirty = "{\n  // line comment\n  \"a\": \"x, }\", /* block */\n  \"b\": \"//not-a-comment\",\n  \"c\": [1, 2,],\n}\n";
    let value: serde_json::Value = serde_json::from_str(&clean(dirty)).expect("parses");
    assert_eq!(value["a"], "x, }");
    assert_eq!(value["b"], "//not-a-comment");
    assert_eq!(value["c"], serde_json::json!([1, 2]));
}
