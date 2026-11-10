//! A load whose target argument is no literal (lane v235_graphtrust,
//! item T1): every tree below RUNS under its language's own toolchain —
//! Rscript 4.6, Lua 5.4, CPython 3.13, bun 1.3 (each was run when the
//! lane was cut, the probe in its scratch notes; the test itself runs
//! none of them) — and reaches its `lib` file ONLY through a computed
//! argument (`source(file.path(…))`, `require("lib." .. name)`,
//! `dofile(dir .. "x.lua")`, `__import__("hel" + "per")`,
//! `require("./lib/" + name)`, `import("./lib/" + name)`). The 1.9.2
//! reports read that file dead with the language VOUCHING for it (no
//! site was ever opened, so nothing was unresolved), and `ce erase`
//! planned to delete the live `lib/data.lua`. Now the call opens a
//! `dynamic` site the resolve family refuses as dynamic: the file is
//! still dead (nothing names it), but unvouched — and never eraseable.
//! A literal `__import__("helper")`, `require("./lib/util")` or
//! `import("./lib/util")` is an import like any other: its file is
//! live (1.9.2 opened no site for the TS two and read `lib/util.ts`
//! dead).
//!
//! One literal per tree: a header `name|dead,…` then `@path` lines, each
//! followed by the file's text (a tuple table rhymes under the clone
//! gate).

use crate::common::{core_bin, init_and_commit, tmp};
use codeeraser::erase;
use codeeraser::graph::deadcode;

const TREES: &str = r#"r_source_filepath|lib/more.R
@app.R
source(file.path("lib", "more.R"))
cat(more(1), "\n")
@lib/more.R
more <- function(x) x + 1
====
lua_require_concat|lib/util.lua
@main.lua
local name = "util"
local u = require("lib." .. name)
print(u.f(1))
@lib/util.lua
return { f = function(x) return x + 1 end }
====
lua_dofile_concat|lib/data.lua
@main.lua
local dir = "lib/"
local t = dofile(dir .. "data.lua")
print(t.n)
@lib/data.lua
return { n = 7 }
====
py_dunder_import_concat|helper.py
@__main__.py
m = __import__("hel" + "per")
print(m._f(1))
@helper.py
def _f(x):
    return x + 1
====
py_dunder_import_literal|
@__main__.py
m = __import__("helper")
print(m._f(1))
@helper.py
def _f(x):
    return x + 1
====
ts_require_concat|lib/util.ts
@main.ts
const name = "util";
const u = require("./lib/" + name);
console.log(u.f(1));
@lib/util.ts
export function f(x: number): number {
  return x + 1;
}
@ce.toml
[graph]
entry_globs = ["main.ts"]
====
ts_import_expr|lib/util.ts
@main.ts
const name = "util";
import("./lib/" + name).then((u) => console.log(u.f(1)));
@lib/util.ts
export function f(x: number): number {
  return x + 1;
}
@ce.toml
[graph]
entry_globs = ["main.ts"]
====
ts_require_literal|
@main.ts
const u = require("./lib/util");
console.log(u.f(1));
@lib/util.ts
export function f(x: number): number {
  return x + 1;
}
@ce.toml
[graph]
entry_globs = ["main.ts"]
====
ts_import_literal|
@main.ts
import("./lib/util").then((u) => console.log(u.f(1)));
@lib/util.ts
export function f(x: number): number {
  return x + 1;
}
@ce.toml
[graph]
entry_globs = ["main.ts"]
"#;

/// A tree's files, `(path, text)`.
type Files = Vec<(String, String)>;

/// The trees: (name, dead files expected, files).
fn trees() -> Vec<(String, Vec<String>, Files)> {
    // a CRLF checkout (Windows) must read the same trees
    TREES
        .replace("\r\n", "\n")
        .split("====\n")
        .map(|block| {
            let (head, body) = block.split_once('\n').expect("header line");
            let (name, dead) = head.split_once('|').expect("name|dead");
            let files = body
                .split('@')
                .filter(|f| !f.is_empty())
                .map(|f| {
                    let (path, text) = f.split_once('\n').expect("path line");
                    (path.to_string(), text.to_string())
                })
                .collect();
            let dead = dead
                .split(',')
                .filter(|d| !d.is_empty())
                .map(str::to_string)
                .collect();
            (name.to_string(), dead, files)
        })
        .collect()
}

fn seeded(name: &str, files: &[(String, String)]) -> std::path::PathBuf {
    let dir = tmp(&format!("dynload-{name}"));
    for (path, text) in files {
        let at = dir.join(path);
        std::fs::create_dir_all(at.parent().expect("parent")).expect("dirs");
        std::fs::write(at, text).expect("write");
    }
    init_and_commit(&dir, "seed");
    dir
}

/// Every tree: the dead set is exactly the expected one, each dead row
/// unvouched (confidence 0), and the erase plan offers none of them.
#[test]
fn a_computed_load_keeps_its_language_from_vouching() {
    let core = core_bin();
    for (name, want, files) in trees() {
        let dir = seeded(&name, &files);
        let report = deadcode::run(&dir, None, &core).expect("deadcode");
        let dead: Vec<&str> = report.dead.iter().map(|d| d.path.as_str()).collect();
        assert_eq!(dead, want, "{name}: dead set");
        for row in &report.dead {
            assert_eq!(row.conf, Some(0), "{name}: {} is not unvouched", row.path);
        }
        let plan = erase::plan(&dir, None, &core).expect("plan");
        assert!(
            plan.rows.iter().all(|r| !r.eraseable),
            "{name}: the plan offers a file only a computed load reaches"
        );
    }
}

/// The private dead file (`lib/data.lua`, `helper.py`) is refused for the
/// language's unresolved site by name — the public ones meet the
/// public-surface bar first.
#[test]
fn erase_names_the_unresolved_language() {
    let core = core_bin();
    for (name, _, files) in trees() {
        if !matches!(name.as_str(), "lua_dofile_concat" | "py_dunder_import_concat") {
            continue;
        }
        let dir = seeded(&format!("{name}-erase"), &files);
        let plan = erase::plan(&dir, None, &core).expect("plan");
        let reasons: Vec<&str> = plan.rows.iter().map(|r| r.reason).collect();
        assert_eq!(reasons, ["language_unresolved"], "{name}");
    }
}
