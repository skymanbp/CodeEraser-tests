//! An External answer is an answer, not a miss (lane v235_licence, item
//! L2 — graphtrust's T4): a site whose ladder answers External (a
//! standard-library module, a dependency the manifest declares, Rust's
//! `std`) draws no edge, and until this lane the trust ledger counted it
//! unresolved, so ONE stdlib import made the whole language unvouched
//! (booklet 06 §2 always said External is a correct terminal answer).
//! The index now records the answer (`sites.ledger` bit 0) and the
//! ledger counts it answered: every tree below has one private orphan;
//! the trees whose every site is answered — none External, stdlib,
//! declared third-party, Rust std — read it `[vouched]` with zero
//! unresolved sites, and the tree importing a module nothing answers
//! keeps its language unvouched. The erase plan is the same in all five
//! but for the reason's word: no whole-file row is eraseable (L1), so a
//! vouched orphan is refused as `reachable_by_name` and an unvouched one
//! as `language_unresolved` — `[vouched]` is a statement about sites,
//! never a licence.
//!
//! One literal per tree: a header `name|dead,…` then `@path` lines, each
//! followed by the file's text (the dynamic_loads.rs shape).

use crate::common::core_bin;
use crate::common::trees::{dead_sets_hold, reasons};

const TREES: &str = r#"py_no_external|orphan.py
@__main__.py
import used
print(used.f(1, 2))
@used.py
def f(a, b):
    return (a + b) * 2
@orphan.py
def _g(a, b):
    return (a - b) * 3
====
py_stdlib|orphan.py
@__main__.py
import os
import used
print(used.f(1, 2), os.sep)
@used.py
def f(a, b):
    return (a + b) * 2
@orphan.py
def _g(a, b):
    return (a - b) * 3
====
py_thirdparty_declared|orphan.py
@__main__.py
import requests
import used
print(used.f(1, 2))
@used.py
def f(a, b):
    return (a + b) * 2
@orphan.py
def _g(a, b):
    return (a - b) * 3
@pyproject.toml
[project]
name = "x"
version = "0"
dependencies = ["requests"]
====
rs_std|src/orphan.rs
@Cargo.toml
[package]
name = "x"
version = "0.1.0"
edition = "2021"
@src/main.rs
use std::collections::HashMap;
mod used;
fn main() {
    let m: HashMap<i32, i32> = HashMap::new();
    println!("{} {}", used::f(1, 2), m.len());
}
@src/used.rs
pub fn f(a: i32, b: i32) -> i32 {
    a + b
}
@src/orphan.rs
fn helper(a: i32, b: i32) -> i32 {
    let total = a + b;
    total * 2
}
====
py_unknown_module|orphan.py
@__main__.py
import nowhere
import used
print(used.f(1, 2))
@used.py
def f(a, b):
    return (a + b) * 2
@orphan.py
def _g(a, b):
    return (a - b) * 3
"#;

/// Each tree's orphan is the one dead file; its confidence is vouched
/// (2) exactly when no site of the language is left unanswered — an
/// External answer counts as answered — and the plan names the reason
/// that follows from it.
#[test]
fn an_external_answer_keeps_the_language_vouched() {
    let core = core_bin();
    for tree in dead_sets_hold("external", TREES, &core) {
        let unknown = tree.name == "py_unknown_module";
        let (conf, reason) = if unknown {
            (0, "language_unresolved")
        } else {
            (2, "reachable_by_name")
        };
        for (path, got) in &tree.dead {
            assert_eq!(*got, Some(conf), "{}: {path}", tree.name);
        }
        assert_eq!(reasons(&tree.dir, &core), [reason], "{}", tree.name);
    }
}
