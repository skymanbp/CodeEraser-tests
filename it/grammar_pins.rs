//! Grammar pins — the M5-3k spike made permanent, widened at plan
//! v2.30 step 1. tree-sitter-haskell's availability under the core
//! ABI was an UNVERIFIED assumption in the locked design; this test
//! was the spike's conclusion, and it now holds every grammar crate
//! the CLI pins: each must speak an ABI the pinned tree-sitter core
//! accepts (13..=15 for 0.27), parse its sample without an error node
//! and yield the root kind its tables anchor on — so a dependency bump
//! that breaks a grammar fails HERE by name instead of degrading scan
//! output. The six plan v2.30 grammars are pinned from step 1 on,
//! before their `Lang::grammar` arms exist (each language's own step
//! wires one): the crates are already what NOTICE inventories and
//! what the release binary links.

use tree_sitter::{Language, Parser};
use tree_sitter_language::LanguageFn;

/// One pinned grammar: the crate, its language constructor, a sample
/// it must parse cleanly, and the root kind that sample yields. The
/// constructor is stored as the crate's own `LanguageFn` value rather
/// than a closure: a row of four plain tokens keeps eight rows under
/// the clone gate's window, and `Language::from` does the conversion
/// once, in `parsed`.
const PINS: &[(&str, LanguageFn, &str, &str)] = &[
    (
        "tree-sitter-haskell",
        tree_sitter_haskell::LANGUAGE,
        "module M where\n\nf :: Int -> Int\nf x = if x > 0 then x else negate x\n",
        "haskell",
    ),
    (
        "tree-sitter-c",
        tree_sitter_c::LANGUAGE,
        "int f(int x) {\n  if (x > 0) {\n    return x;\n  }\n  return -x;\n}\n",
        "translation_unit",
    ),
    (
        "tree-sitter-cpp",
        tree_sitter_cpp::LANGUAGE,
        "namespace n {\nclass K {\n public:\n  int f(int x) const { return x > 0 ? x : -x; }\n};\n}\n",
        "translation_unit",
    ),
    (
        "tree-sitter-lua",
        tree_sitter_lua::LANGUAGE,
        "local function f(x)\n  if x > 0 then\n    return x\n  end\n  return -x\nend\n",
        "chunk",
    ),
    (
        "tree-sitter-java",
        tree_sitter_java::LANGUAGE,
        "class K {\n  int f(int x) {\n    if (x > 0) {\n      return x;\n    }\n    return -x;\n  }\n}\n",
        "program",
    ),
    (
        "tree-sitter-r",
        tree_sitter_r::LANGUAGE,
        "f <- function(x) {\n  if (x > 0) x else -x\n}\n",
        "program",
    ),
    (
        "tree-sitter-html",
        tree_sitter_html::LANGUAGE,
        "<!doctype html>\n<html><head><title>t</title></head><body><a href=\"#x\">x</a></body></html>\n",
        "document",
    ),
];

/// The parsed sample of one pin, with the grammar's ABI checked and
/// `set_language` refused by name.
fn parsed(name: &str, language: LanguageFn, sample: &str) -> tree_sitter::Tree {
    let language = Language::from(language);
    let abi = language.abi_version();
    assert!(
        (13..=15).contains(&abi),
        "{name}: ABI {abi} outside the core's 13..=15 window"
    );
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .unwrap_or_else(|e| panic!("{name}: ABI incompatible with the pinned core — {e}"));
    parser
        .parse(sample, None)
        .unwrap_or_else(|| panic!("{name}: parse returned nothing"))
}

/// Every pinned grammar speaks the core's ABI window, parses its
/// sample without an error node and yields the expected root kind.
#[test]
fn every_pinned_grammar_speaks_the_core_abi_and_parses_its_sample() {
    for &(name, language, sample, root_kind) in PINS {
        let tree = parsed(name, language, sample);
        let root = tree.root_node();
        assert_eq!(root.kind(), root_kind, "{name}: root kind");
        assert!(
            !root.has_error(),
            "{name} mis-parses its sample: {}",
            root.to_sexp()
        );
    }
}

/// The two Haskell node kinds the scan spec anchors on: the bind and
/// its conditional body must be visible, else function extraction
/// and CoC have nothing to stand on (the original spike's question).
#[test]
fn haskell_exposes_the_bind_and_the_conditional() {
    let (name, language, sample, _) = PINS[0];
    let tree = parsed(name, language, sample);
    let mut kinds = Vec::new();
    let mut stack = vec![tree.root_node()];
    while let Some(n) = stack.pop() {
        kinds.push(n.kind());
        for i in (0..n.child_count()).rev() {
            stack.push(n.child(i).unwrap());
        }
    }
    assert!(kinds.contains(&"function"), "no function node: {kinds:?}");
    assert!(
        kinds.contains(&"conditional"),
        "no conditional node: {kinds:?}"
    );
}
