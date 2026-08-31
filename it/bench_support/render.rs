//! Shared primitives for the GENERATED bench surfaces (the render
//! tests): one owner for the parsed contract, field access, the
//! latest-version read, the measured-at label and the
//! iterate-a-table shape — the dedup gate caught every one of them
//! growing a twin when the dashboard renderer landed beside the
//! chip renderer.

use serde_json::Value;

/// contracts/bench/bench.json, parsed.
pub fn doc() -> Value {
    let text = std::fs::read_to_string(super::bench_path()).expect("bench.json");
    serde_json::from_str(&text).expect("bench.json parses")
}

/// String field or empty — absent fields render blank, never panic.
pub fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}

/// The newest series version present (rows are sorted by version).
pub fn latest(d: &Value) -> &str {
    d["rows"]
        .as_array()
        .and_then(|rows| rows.last())
        .map(|row| s(row, "version"))
        .unwrap_or("")
}

/// The release this build IS, when the series holds no row for it.
///
/// The heading used to call `latest` the "latest version", which was
/// true only while every release joined the series. v1.3.1 shipped the
/// same `cli/src` and `core/app` as v1.3.0 and so did not join — a row
/// for an identical program publishes machine drift as if it were a
/// version delta, which the BENCH header warns is wider than most
/// deltas a reader would try to read. The heading then quietly went on
/// naming v1.3.0 as the latest, which it no longer was. Nothing but
/// this comparison can notice that, so every surface asks it.
pub fn release_without_a_row(d: &Value) -> Option<&'static str> {
    let release = env!("CARGO_PKG_VERSION");
    (latest(d) != release).then_some(release)
}

/// That fact as one sentence, for any surface that shows the numbers.
/// Plain text: it goes into a Markdown paragraph and an HTML caption
/// unchanged, and carries no character either of them would escape.
/// Empty when the release did join — the sentence must not appear on a
/// page whose heading already names the current version.
pub fn unmeasured_note(d: &Value, zh: bool) -> String {
    match (release_without_a_row(d), zh) {
        (None, _) => String::new(),
        // The separator between two sentences is language-specific:
        // English joins with a space, Chinese with nothing at all —
        // a space after 。 is a typographic error, not a nicety.
        (Some(v), true) => format!("当前发布 v{v} 没有自己的行。"),
        (Some(v), false) => format!(" The current release, v{v}, has no row of its own."),
    }
}

/// Every surface printing a version beside these numbers must name the
/// release this build IS — in its heading when that release joined the
/// series, in `unmeasured_note` when it did not.
///
/// v1.3.1 shipped with all four surfaces headed "v1.3.0" and every byte
/// gate green: a gate that compares a file to its own generator cannot
/// notice that the generator stopped telling the truth. This is the one
/// assertion that reads the crate version instead of the contract.
pub fn names_the_release(surface: &str, text: &str) {
    let release = env!("CARGO_PKG_VERSION");
    assert!(
        text.contains(&format!("v{release}")),
        "{surface} prints the latency numbers without naming v{release}, the \
         release this build is. Either the series gained a row for it and the \
         heading should say so, or it did not and the surface owes the reader \
         the sentence saying which version these numbers came from."
    );
}

/// The measured-at label with the dirty suffix — one spelling on
/// every surface (dates are digits and dashes, safe in md and html).
pub fn measured(row: &Value) -> String {
    format!(
        "{}{}",
        s(row, "measured_at"),
        if row["dirty"] == Value::Bool(true) {
            " (dirty)"
        } else {
            ""
        }
    )
}

/// Concatenate one rendered line per row of `d[key]` — the table
/// shape every generated surface repeats.
pub fn rows_with(d: &Value, key: &str, f: impl Fn(&Value) -> String) -> String {
    d[key].as_array().expect(key).iter().map(f).collect()
}
