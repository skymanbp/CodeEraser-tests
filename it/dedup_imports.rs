//! The import family gives the clone stream no token (v2.35 imports
//! lane; booklet 01 §1): a header of imports is no clone material, a
//! function under different imports is a clone of exactly the function,
//! a local import inside one of two otherwise identical bodies does not
//! break the block, and every language's declarations and load
//! statements leave the stream exactly as if they were absent — while the
//! forms that stay code keep their tokens.

use crate::common;
use crate::dedup_core::toks as hashes;
use codeeraser::dedup::{analyze, pairs::Block, tokens};
use codeeraser::scan::lang::Lang;
use std::path::Path;

fn blocks(name: &str, files: &[(&str, &str)]) -> Vec<Block> {
    let dir = common::tmp(name);
    for (rel, text) in files {
        std::fs::write(dir.join(rel), text).expect("write");
    }
    analyze(&dir, None, None, None).expect("analyze").0.blocks
}

/// Twelve imports and a package line: well past the 50-token line under
/// T2, and nothing but plumbing.
fn java_header(package: &str) -> String {
    let names =
        "List Map Set Optional Objects Arrays Collections Iterator Locale Random Scanner Timer";
    let mut out = format!("package com.example.{package};\n\n");
    for n in names.split(' ') {
        out.push_str(&format!("import java.util.{n};\n"));
    }
    out
}

#[test]
fn a_header_of_imports_is_no_clone_material() {
    let (a, b) = (java_header("one"), java_header("two"));
    assert!(blocks("imports-header-only", &[("A.java", &a), ("B.java", &b)]).is_empty());
    let a = a + "\npublic class A {\n}\n";
    let b = b + "\npublic class B {\n}\n";
    assert!(blocks("imports-header-opener", &[("A.java", &a), ("B.java", &b)]).is_empty());
}

const FUNCTION: &str = "def work(items, limit):
    total = 0
    for item in items:
        if item > limit:
            total += item * 3 + 7
        else:
            total -= item // 2
    while total > 100:
        total = total - limit
    return [total, limit, len(items)]
";

#[test]
fn a_function_under_different_imports_is_a_clone_of_the_function() {
    let a = format!("import os\nimport sys\n\n{FUNCTION}");
    let b = format!(
        "from collections import deque\nimport json, re\nfrom . import util\n\n{}",
        FUNCTION.replace("work", "job")
    );
    let found = blocks("imports-function", &[("a.py", &a), ("b.py", &b)]);
    assert_eq!(found.len(), 1, "{found:?}");
    let block = &found[0];
    assert_eq!(
        block.tokens,
        hashes(Lang::Python, FUNCTION).len(),
        "the function's tokens, no import's"
    );
    let lines = FUNCTION.lines().count();
    assert_eq!(
        (block.a_start, block.a_end),
        (4, 3 + lines),
        "a: the function's lines"
    );
    assert_eq!(
        (block.b_start, block.b_end),
        (5, 4 + lines),
        "b: the function's lines"
    );
}

#[test]
fn a_local_import_inside_one_body_does_not_break_the_block() {
    let with = FUNCTION.replacen("    total = 0\n", "    total = 0\n    import json\n", 1);
    let found = blocks(
        "imports-local",
        &[("a.py", &with), ("b.py", &FUNCTION.replace("work", "job"))],
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].tokens, hashes(Lang::Python, FUNCTION).len());
    assert_eq!(
        (found[0].a_start, found[0].a_end),
        (1, with.lines().count()),
        "the run crosses the import"
    );
}

/// Each block: `ext @@ why` over a source split by a `----` line into
/// the import family (above) and the code (below): the stream of the
/// whole is the stream of the code.
const PLUMBING: &str = r#"
py @@ the three import declarations, a local one, and the load statements of the call table
from __future__ import annotations
import os, sys as system
from a.b import (c, d as e)
m = importlib.import_module("x.y")
__import__("z")
----
def f(x):
    return x + 1
====
ts @@ every import form, `import x = require`, CommonJS loads and a bare dynamic import
import x from "m";
import { a, b as c } from "./n";
import * as ns from "o";
import type { T } from "t";
import fs = require("fs");
const p = require("path");
require("effect");
import("dyn");
export * from "./a";
export * as ns from "./b";
export { a, b as d } from "./c";
export type { T } from "./d";
----
export function f(y: number): number {
  return y * 2;
}
====
tsx @@ the re-export in TSX
export * from "./a";
export { a } from "./c";
----
const e = <div />;
====
rs @@ `use` in every visibility, `pub use` included, `extern crate`, and the module mount in every visibility
extern crate alloc;
pub extern crate core as c2;
use std::collections::{BTreeMap, BTreeSet};
pub use crate::a::b;
pub(crate) use super::*;
mod a;
pub(crate) mod b;
pub mod c;
----
fn f(x: u32) -> u32 {
    x + 1
}
====
go @@ the package clause and both import forms
package main
import "fmt"
import (
	"os"
	str "strings"
)
----
func f(x int) int { return x + 1 }
====
java @@ the package and every import form
package com.example.foo;
import java.util.List;
import java.util.*;
import static java.lang.Math.max;
----
class F { int f(int x) { return x + 1; } }
====
c @@ both include forms and a macro-named one
#include <stdio.h>
#include "local.h"
#include MACRO_H
----
int f(int x) { return x + 1; }
====
cpp @@ both include forms
#include <vector>
#include "local.hpp"
----
int f(int x) { return x + 1; }
====
hs @@ every import form, PackageImports included
import Data.List (sort)
import qualified Data.Map as M
import "base" Data.Maybe
import Data.Char hiding (ord)
----
f :: Int -> Int
f x = x + 1
====
lua @@ local, multiple, protected, global and bare loads
local util = require("luarocks.util")
local fs = require "luarocks.fs"
local a, b = require("a"), require("b")
local ok, mod = pcall(require, "opt")
x = require("global")
require("side")
----
local function f(y)
  return y + 1
end
====
R @@ the library family and source at statement level
library(stringr)
library("dplyr")
require(magrittr)
requireNamespace("x", quietly = TRUE)
source("helpers.R")
----
f <- function(x) x + 1
"#;

fn lang_of(ext: &str) -> Lang {
    Lang::from_path(Path::new(&format!("x.{ext}"))).expect("a known extension")
}

#[test]
fn the_import_family_leaves_the_stream_as_if_absent() {
    for block in PLUMBING.trim().split("\n====\n") {
        let (head, body) = block.split_once('\n').expect("a header");
        let (ext, why) = head.split_once(" @@ ").expect("ext @@ why");
        let (_, code) = body.split_once("----\n").expect("a ---- line");
        let lang = lang_of(ext);
        assert_eq!(
            hashes(lang, body.replacen("----\n", "", 1).as_str()),
            hashes(lang, code),
            "{ext}: {why}"
        );
    }
}

/// Each line: `ext @@ source`: a form that stays code keeps its tokens.
const CODE: &str = r#"
java @@ @Deprecated class A {}
rs @@ mod x { fn f() {} }
rs @@ pub(crate) mod x {}
rs @@ #[path = "y.rs"] mod y;
ts @@ export const k = 1;
ts @@ export default k;
ts @@ export { a, b };
ts @@ export { k as j };
tsx @@ export default function F() { return <div />; }
ts @@ const w = require("w").default;
ts @@ let s = require("s"), u = 3;
cpp @@ using namespace std;
cpp @@ #define X 1
hs @@ foreign import ccall "sin" c_sin :: Double -> Double
lua @@ local cfg = require("x").get
R @@ x <- library(y)
R @@ suppressPackageStartupMessages(library(foo))
py @@ o = importlib.import_module("a").b
"#;

#[test]
fn the_forms_that_stay_code_keep_their_tokens() {
    for line in CODE.trim().lines() {
        let (ext, src) = line.split_once(" @@ ").expect("ext @@ source");
        assert!(
            !hashes(lang_of(ext), src).is_empty(),
            "{ext}: `{src}` lost its tokens"
        );
    }
    // Haskell's `foreign import` keyword is an unnamed node of kind
    // `import`: the named-only test keeps it a token
    let foreign = hashes(
        Lang::Haskell,
        "foreign import ccall \"sin\" c_sin :: Double -> Double\n",
    );
    assert!(
        foreign.contains(&tokens::fnv1a(b"import")),
        "the foreign keyword stays"
    );
}
