//! The sites a call opens (plan v2.30 step 4; booklet §8): Lua's
//! `require` and file loaders, R's `source` and `library` family and
//! its `pkg::name` operator, Python's `import_module` / `__import__` and
//! TypeScript's `require` / `import(…)` — and the `dynamic` site a
//! computed target argument opens (lane v235_graphtrust) — through
//! detect() itself, so the table is what the graph stores.

use super::super::detect;
use crate::testutil::blocks;

/// Each block: a header `ext @@ kind=spec@line|… @@ why` over a source.
/// One literal rather than rows of typed tuples: rows of one shape
/// repeat every dozen tokens, and the clone gate reads such a table as
/// clones of itself.
const CASES: &str = r#"
lua @@ require=a.b@1|require=c.d@2|require=e.f@3|dynamic=name@4|dynamic="g" .. x@5|load=h.lua@6|load=i.lua@7 @@ a module by `require` in its three call forms (a long string's content included), a file by `dofile` / `loadfile`; a computed argument holds no target in the text, so it opens a `dynamic` site spelled by the argument; a callee off an object is some other function and opens none
local a = require "a.b"
local b = require("c.d")
local c = require [[e.f]]
local d = require(name)
local e = require("g" .. x)
dofile("h.lua")
loadfile 'i.lua'
local f = x.require("no")
====
lua @@ require=a.b@1|load=c.lua@2|require=d.e@3|dynamic=name@4 @@ a protected call is the call it protects: `pcall` passes what follows the function, `xpcall` what follows its message handler; a computed target is a `dynamic` site; a function off an object, a wrapper with nothing to pass and a function no row names open no site
local ok, a = pcall(require, "a.b")
local ok2 = pcall(dofile, "c.lua")
xpcall(require, handler, "d.e")
pcall(require, name)
pcall(m.require, "no")
pcall(require)
pcall(print, "no")
====
R @@ library=dplyr@1|library=tidyr@2|library=stats@3|library=rlang@4|source=helpers.R@5|source=b.R@6|source=raw.R@7|dynamic=paste0("x", ".R")@8|library=stringr@9|dynamic=p@10|library=q@11|dynamic=pkg@12|source=no.R@13|library=base@13|source=multi.R@15|source=part.R@17|library=pm@18|dynamic=qq@20|library=stats@21 @@ R matches a named argument before a positional one, so `package =` and `file =` win wherever they sit, and a name matches exactly or as the one prefix no other formal shares (`fi =` is `file`, `pack =` is `package`, `e =` opens three formals and names nothing, `ch =` is `character.only`); `library` and `require` take an unquoted name unless `character.only` is passed as anything but FALSE, while `requireNamespace` evaluates its argument (a bare name there is a variable) — each of those, and a computed `source`, opens a `dynamic` site; `pkg::name` names its package whatever it selects, and a qualified `base::source` opens its `source` site beside that `library` site — under any other package it is some other function, and so is `x$source` (a member is no package); a raw string reads by its content, and a call spanning lines puts its site on the argument's line
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
source(fi = "part.R")
library(pack = "pm")
source(e = "amb.R")
library(qq, ch = TRUE)
stats::source("np.R")
x$source("no2.R")
====
py @@ import=a.b@1|import=c@2|dynamic=name@3|dynamic=f"x{y}"@4|import=d@5|dynamic="e" + f@6 @@ `importlib.import_module` and the builtin `__import__`, bare or qualified by `importlib`, import the module a literal names, as `import a.b` does; an f-string with an interpolation or any other computed argument is a `dynamic` site; the same name off any other object is some other function
m = importlib.import_module("a.b")
n = __import__("c")
o = import_module(name)
p = __import__(f"x{y}")
q = importlib.__import__("d")
r = __import__("e" + f)
s = self.import_module("no")
t = other.__import__("no")
====
ts @@ import=./a@1|import=./b@2|dynamic=x@3|dynamic="./c/" + name@4|import=./d@5|dynamic=`./t${n}`@8 @@ CommonJS `require` and the dynamic `import(…)` import the specifier a literal names; a computed argument or a template literal is a `dynamic` site; `import d = require("./d")` is the one statement site it always was, and `require.resolve` / a `require` off another object load nothing here
const a = require("./a");
import("./b");
const c = require(x);
import("./c/" + name).then(f);
import d = require("./d");
const e = require.resolve("./no");
const g = mod.require("./no");
const h = require(`./t${n}`);
"#;

#[test]
fn a_call_opens_its_site_by_its_target_argument() {
    for (lang, [want, why], body) in blocks(CASES) {
        let got: Vec<String> = detect(body, lang)
            .iter()
            .map(|s| format!("{}={}@{}", s.kind, s.spec, s.line))
            .collect();
        assert_eq!(got.join("|"), want, "{why}\n--- source ---\n{body}");
    }
}
