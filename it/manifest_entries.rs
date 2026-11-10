//! A manifest names what a runtime starts (lane v235_graphtrust, item
//! T2): a package.json `main` / `bin` / `exports`, a pyproject.toml
//! `[project.scripts]` / `[project.entry-points.*]` module, a Cargo
//! `[[example]] path`. Every tree with a program RAN when the lane was
//! cut (bun 1.3, CPython 3.13, cargo 1.94 — the probe in its scratch
//! notes; the test itself runs none of them). The 1.9.2 reports read the
//! TS and Python programs all-dead "for want of an entry flag" and the
//! example's helper unvouched; a manifest entry is now an entry root
//! (the declared-target role). The controls stand: a package.json that
//! names no entry changes nothing, a module no manifest names and nothing
//! imports stays dead, an `exports` pattern names only what it matches —
//! a `*` after a name prefix (`./src/feat-*.ts`) included.
//!
//! One literal per tree: a header `name|dead,…` then `@path` lines, each
//! followed by the file's text (a tuple table rhymes under the clone
//! gate).

use crate::common::core_bin;
use crate::common::trees::dead_sets_hold;

const TREES: &str = r#"ts_main|
@package.json
{"name": "app", "main": "main.ts"}
@main.ts
import * as ns from "./lib";
console.log(ns.f(1));
@lib/index.ts
export function f(x: number): number {
  return x + 1;
}
====
ts_no_manifest_entry|lib/index.ts,main.ts
@package.json
{"name": "app"}
@main.ts
import * as ns from "./lib";
console.log(ns.f(1));
@lib/index.ts
export function f(x: number): number {
  return x + 1;
}
====
ts_bin_map_js_twin|
@package.json
{"name": "app", "bin": {"app": "./src/cli.js"}}
@src/cli.ts
import { f } from "./util";
console.log(f(1));
@src/util.ts
export function f(x: number): number {
  return x + 1;
}
====
ts_exports_pattern|src/other.ts
@package.json
{"name": "app", "exports": {".": "./src/index.ts", "./feat/*": {"import": "./src/feat/*.ts"}}}
@src/index.ts
export const v = 1;
@src/feat/a.ts
export const a = 1;
@src/feat/b.ts
export const b = 2;
@src/other.ts
export const o = 3;
====
ts_exports_name_prefix|src/feat/a.ts,src/other.ts
@package.json
{"name": "app", "exports": {"./feat-*": "./src/feat-*.ts"}}
@src/feat-a.ts
export const a = 1;
@src/feat/a.ts
export const b = 2;
@src/other.ts
export const o = 3;
====
py_scripts|app/stray.py
@pyproject.toml
[project]
name = "app"
version = "0"
[project.scripts]
app = "app.cli:main"
[project.entry-points."x.plugins"]
p = "app.plug"
@app/__init__.py
@app/cli.py
from . import util

def main():
    print(util.g())
@app/util.py
def g():
    return 1
@app/plug.py
X = 1
@app/stray.py
Y = 2
====
cargo_example_path|
@Cargo.toml
[package]
name = "x"
version = "0.1.0"
edition = "2021"
[[example]]
name = "demo"
path = "samples/demo.rs"
@src/lib.rs
pub fn f() -> i32 {
    1
}
@samples/demo.rs
mod helper;
fn main() {
    println!("{}", helper::h());
}
@samples/helper.rs
pub fn h() -> i32 {
    2
}
"#;

/// Every tree: the dead set is exactly the expected one.
#[test]
fn a_manifest_entry_is_an_entry_root() {
    dead_sets_hold("entries", TREES, &core_bin());
}
