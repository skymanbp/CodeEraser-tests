use super::*;
use serde_json::json;

/// Two classes of strings: `file` by index, `mount` with no integer.
struct Strings;

impl Resolve for Strings {
    fn resolve(&self, class: &str, ints: &[i128]) -> Option<String> {
        match (class, ints) {
            ("file", [i]) => ["a.rs", "{}.rs", "目录/b.rs"]
                .get(*i as usize)
                .map(|s| s.to_string()),
            ("mount", []) => Some("sub".into()),
            _ => None,
        }
    }
}

fn file(i: i64) -> serde_json::Value {
    json!({"$": ["file", i]})
}

fn reply(lines: serde_json::Value) -> serde_json::Value {
    json!({"degraded": false, "document": {}, "lines": lines, "exit": {"fail": true}})
}

#[test]
fn holes_fill_left_to_right_and_streams_and_the_veto_come_through() {
    let r = reply(json!([
        [0, "{} <-> {} ({} tokens)", file(0), file(2), {"$": ["mount"]}],
        [1, "ce：读不了 {}", file(1)],
        [0, "no hole at all"],
    ]));
    let (lines, fail) = bind_lines(&r, &Strings).expect("bound");
    assert!(fail);
    let want = [
        (Stream::Out, "a.rs <-> 目录/b.rs (sub tokens)"),
        (Stream::Err, "ce：读不了 {}.rs"),
        (Stream::Out, "no hole at all"),
    ];
    let got: Vec<(Stream, &str)> = lines.iter().map(|l| (l.stream, l.text.as_str())).collect();
    assert_eq!(got, want);
}

#[test]
fn a_string_holding_braces_never_becomes_a_hole() {
    let r = reply(json!([[0, "{} then {}", file(1), file(0)]]));
    let (lines, _) = bind_lines(&r, &Strings).expect("bound");
    assert_eq!(lines[0].text, "{}.rs then a.rs");
}

#[test]
fn every_malformed_line_is_an_error_by_name() {
    let refused = [
        (
            json!([[0, "{} and {}", file(0)]]),
            "line 0: 2 hole(s) and 1 reference(s)",
        ),
        (
            json!([[0, "{}", file(0), file(0)]]),
            "1 hole(s) and 2 reference(s)",
        ),
        (
            json!([[0, "x"], [0, "{}", file(9)]]),
            "line 1: document: no string for",
        ),
        (json!([[0, "{}", "a.rs"]]), "\"a.rs\" is not a reference"),
        (
            json!([[0, "{}", {"$": ["file", 0], "x": 1}]]),
            "is not a reference",
        ),
        (json!([[2, "x"]]), "stream 2 is not 0 or 1"),
        (json!([[0, 7]]), "text 7 is not a string"),
        (json!([[0]]), "not [stream, text, reference…]"),
    ];
    for (lines, says) in refused {
        let err = bind_lines(&reply(lines), &Strings)
            .expect_err(says)
            .to_string();
        assert!(err.contains(says), "{err}");
    }
}

#[test]
fn a_reply_without_lines_or_exit_names_the_proto_it_needs() {
    for r in [
        json!({"document": {}, "exit": {"fail": false}}),
        json!({"lines": []}),
    ] {
        let err = bind_lines(&r, &Strings)
            .expect_err("pre-7.10.0")
            .to_string();
        assert!(err.contains("pre-7.10.0 core"), "{err}");
    }
}
