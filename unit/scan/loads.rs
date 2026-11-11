//! Load statements read off the package's `[scan] loads` (v2.35 imports
//! lane): every statement of every statement list in a source, and which
//! of them the shapes admit — the admitted forms of each language beside
//! the forms that stay code. Plus the table against its grammar: every
//! import kind, statement list and shape step names something the pinned
//! grammar has.

use super::*;
use crate::testutil::blocks;

/// The 1-based lines of the statements `is_load` admits, over every
/// statement list of the tree.
fn admitted(body: &str, lang: Lang) -> String {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&lang.grammar().expect("a grammar"))
        .expect("set_language");
    let tree = parser.parse(body, None).expect("parse");
    let spec = crate::scan::spec::spec(lang);
    let mut lines = Vec::new();
    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        let lists = spec.loads.statement_lists.contains(&node.kind());
        for child in ast::children(node) {
            if lists && is_load(child, body.as_bytes(), lang, spec) {
                lines.push(child.start_position().row + 1);
            }
            stack.push(child);
        }
    }
    lines.sort_unstable();
    lines
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

/// Each block: `ext @@ admitted lines @@ why` over a source.
const CASES: &str = r#"
lua @@ 1,2,3,4,5,6,9,11 @@ a `local` or global assignment every value of which is a load (a protected one included) and a bare load call are load statements, in a chunk or a function body; the target side is not inspected (`M.x`); a member of a load, a value that is no load and a table stay code
local a = require("m")
local b = require "m"
local c, d = require("c"), require("d")
local ok, m = pcall(require, "opt")
x = require("g")
require("side")
local cfg = require("x").get
local p, q = require("p"), 3
M.x = require("x")
local function f()
  local dir = require("luarocks.dir")
  return dir
end
local t = {}
====
R @@ 1,2,3,4,5,6,11 @@ a bare `library` / `require` / `requireNamespace` / `source` call at statement level — a qualified `base::source` too, a braced body too — is a load statement; an assignment of one, a condition holding one, a wrapped one and a call no row names stay code
library(stringr)
library("dplyr")
require(magrittr)
requireNamespace("x", quietly = TRUE)
source("helpers.R")
base::source("b.R")
x <- library(y)
if (!requireNamespace("z")) stop("no")
suppressPackageStartupMessages(library(foo))
f <- function() {
  library(g)
  1
}
print("no")
====
ts @@ 1,2,3,4,5,10 @@ a CommonJS declaration whose every declarator holds a load, a bare `require` or `import(…)`, in a module or a function body, are load statements; a declarator that holds no load, a member of a load, an awaited import and a call no row names stay code
const p = require("path");
const { q, r } = require("qr");
var v = require("v");
require("effect");
import("dyn");
let s = require("s"), u = 3;
const w = require("w").default;
const z = await import("z");
function f() {
  const inner = require("inner");
  return inner;
}
const k = foo("x");
====
tsx @@ 1 @@ TSX reads TypeScript's table
const p = require("path");
const e = <div />;
====
py @@ 1,2,3,7 @@ an assignment of an `importlib.import_module` / `__import__` call and a bare one are load statements, in a module or a function body; a member of a load and a call no row names stay code
m = importlib.import_module("x.y")
n = __import__("z")
__import__("w")
o = importlib.import_module("a").b
p = load("x")
def g():
    q = importlib.import_module("q")
    return q
====
rs @@  @@ Rust states no load shape: its imports are declarations
use std::io;
fn f() {}
"#;

#[test]
fn the_shapes_admit_load_statements_and_nothing_else() {
    for (lang, [want, why], body) in blocks(CASES) {
        assert_eq!(
            admitted(body, lang),
            want.trim(),
            "{lang:?}: {why}\n--- source ---\n{body}"
        );
    }
}

/// A step is a field of the grammar or a named kind of it.
fn known(grammar: &tree_sitter::Language, step: &str) -> bool {
    grammar.field_id_for_name(step).is_some() || grammar.id_for_node_kind(step, true) != 0
}

#[test]
fn every_import_kind_and_shape_step_is_in_the_grammar() {
    for lang in Lang::with_grammar().filter(|l| l.fingerprints()) {
        let grammar = lang.grammar().expect("a grammar");
        let spec = crate::scan::spec::spec(lang);
        let named = spec
            .import_kinds
            .iter()
            .chain(spec.loads.statement_lists)
            .copied()
            .chain(spec.loads.shapes.iter().filter_map(|s| s.split(' ').next()));
        for kind in named {
            let at = "is no named kind of its grammar";
            assert!(
                grammar.id_for_node_kind(kind, true) != 0,
                "{lang:?}: `{kind}` {at}"
            );
        }
        for step in spec.loads.shapes.iter().flat_map(|s| s.split(' ').skip(1)) {
            assert!(
                known(&grammar, step),
                "{lang:?}: step `{step}` is not in its grammar"
            );
        }
    }
}
