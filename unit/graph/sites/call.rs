//! The sites a call opens (plan v2.30 step 4; booklet §8): Lua's
//! `require` and file loaders, R's `source` and `library` family and
//! its `pkg::name` operator — through detect() itself, so the table is
//! what the graph stores.

use super::super::detect;
use crate::testutil::blocks;

/// Each block: a header `ext @@ kind=spec@line|… @@ why` over a source.
/// One literal rather than rows of typed tuples: rows of one shape
/// repeat every dozen tokens, and the clone gate reads such a table as
/// clones of itself.
const CASES: &str = r#"
lua @@ require=a.b@1|require=c.d@2|require=e.f@3|load=h.lua@6|load=i.lua@7 @@ a module by `require` in its three call forms (a long string's content included), a file by `dofile` / `loadfile`; a computed argument holds no target in the text and a callee off an object is some other function, so neither opens a site
local a = require "a.b"
local b = require("c.d")
local c = require [[e.f]]
local d = require(name)
local e = require("g" .. x)
dofile("h.lua")
loadfile 'i.lua'
local f = x.require("no")
====
lua @@ require=a.b@1|load=c.lua@2|require=d.e@3 @@ a protected call is the call it protects: `pcall` passes what follows the function, `xpcall` what follows its message handler; a computed target, a function off an object, a wrapper with nothing to pass and a function no row names open no site
local ok, a = pcall(require, "a.b")
local ok2 = pcall(dofile, "c.lua")
xpcall(require, handler, "d.e")
pcall(require, name)
pcall(m.require, "no")
pcall(require)
pcall(print, "no")
====
R @@ library=dplyr@1|library=tidyr@2|library=stats@3|library=rlang@4|source=helpers.R@5|source=b.R@6|source=raw.R@7|library=stringr@9|library=q@11|library=base@13|source=multi.R@15 @@ R matches a named argument before a positional one, so `package =` and `file =` win wherever they sit; `library` and `require` take an unquoted name unless `character.only` is passed as anything but FALSE, while `requireNamespace` evaluates its argument (a bare name there is a variable); `pkg::name` names its package whatever it selects, and a qualified `base::source` is that `library` site alone; a raw string reads by its content, and a call spanning lines puts its site on the argument's line
library(dplyr)
library("tidyr")
require(quietly = TRUE, package = "stats")
requireNamespace("rlang", quietly = TRUE)
source("helpers.R")
source(local = TRUE, file = "b.R")
source(r"(raw.R)")
source(paste0("x", ".R"))
x <- stringr::str_detect(y, "a")
library(p, character.only = TRUE)
library(q, character.only = FALSE)
requireNamespace(pkg)
base::source("no.R")
source(
  "multi.R"
)
"#;

#[test]
fn a_call_opens_a_site_only_on_a_literal_target() {
    for (lang, [want, why], body) in blocks(CASES) {
        let got: Vec<String> = detect(body, lang)
            .iter()
            .map(|s| format!("{}={}@{}", s.kind, s.spec, s.line))
            .collect();
        assert_eq!(got.join("|"), want, "{why}\n--- source ---\n{body}");
    }
}
