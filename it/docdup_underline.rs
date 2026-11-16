//! A title underline is not a fence (DOCDUP_REV 9): a Python module
//! docstring written as a title, a `~~~` underline and prose keeps its
//! prose for docdup, so two modules sharing that prose pair; the same
//! prose under `Example:` inside a real `~~~` fence is code and pairs
//! with nothing (1.10.1 read the underline as a fence that never
//! closes, and shed every line after it).

use crate::common;
use crate::common::core_bin;
use serde_json::Value;

/// 64 words, past the admission floor and the verbatim hit (50 words).
const PROSE: &str = "This module gathers the small helpers the rest of the package leans on
when it reads a configuration file, checks the values a caller passed and
turns them into the settings the transport layer expects. Every helper is a
plain function without state, so a caller may use any one of them alone,
and none of them ever touches the network or the disk on its own.
";

/// The docdup document of a two-module tree; `body` writes a module's
/// docstring between its title and the code.
fn dups(name: &str, body: fn(&str) -> String) -> Value {
    let module =
        |title: &str, code: &str| format!("\"\"\"\n{title}\n{}\"\"\"\n\n{code}", body(title));
    let doc = format!(
        "--- alpha.py\n{}--- beta.py\n{}",
        module("alpha.tools", "def alpha(x):\n    return x + 1\n"),
        module("beta.parts", "class Beta:\n    size = 2\n")
    );
    let dir = common::doc_tree(name, &doc);
    let doc = codeeraser::faces::doc(codeeraser::faces::docdup(
        &dir,
        &core_bin(),
        Default::default(),
    ))
    .expect("docdup");
    doc["dups"].clone()
}

fn underline(title: &str) -> String {
    format!("{}\n\n{PROSE}", "~".repeat(title.len()))
}

fn fenced(_: &str) -> String {
    format!("\nExample:\n~~~\n{PROSE}~~~\n")
}

#[test]
fn a_title_underline_keeps_the_prose_and_a_real_fence_does_not() {
    let pair = dups("docdup-underline", underline);
    assert_eq!(pair.as_array().map(Vec::len), Some(1), "{pair}");
    for side in ["a", "b"] {
        assert!(
            pair[0][side]
                .as_str()
                .is_some_and(|s| s.ends_with("docstring")),
            "{pair}"
        );
    }
    let none = dups("docdup-underline-fence", fenced);
    assert_eq!(none.as_array().map(Vec::len), Some(0), "{none}");
}
