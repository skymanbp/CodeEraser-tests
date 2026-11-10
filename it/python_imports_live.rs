//! Python imports reach what CPython executes (lane v235_pyimport, proto
//! 9.1.0): every tree below RUNS under CPython 3.13 (each was run when
//! the lane was cut — the probes in its scratch notes; the test itself
//! does not invoke Python), so `ce deadcode` must report none of its live
//! files. The 1.9.2 reports read four of them dead [vouched] — a module
//! imported as a NAME (`from pkg import sub`, `from . import sub`) and
//! the ancestor packages a dotted import executes (`import pkg.sub`) —
//! and `ce erase` planned to delete a live module. The one dead file
//! left is the control: `pkg/sub.py` that nothing imports stays dead
//! (an attribute name, `helper`, draws no edge).
//!
//! One literal per tree: a header `name|dead,…` then `@path` lines, each
//! followed by the file's text (a tuple table rhymes under the clone
//! gate).

use crate::common::{core_bin, init_and_commit, tmp};
use codeeraser::erase;
use codeeraser::graph::deadcode;
use std::path::Path;

const TREES: &str = r#"from_pkg_import_sub|
@__main__.py
from pkg import sub
print(sub.f(1, 2))
@pkg/__init__.py
@pkg/sub.py
def f(a, b):
    return a + b
====
from_dot_import_sub|
@__main__.py
import pkg
print(pkg.VALUE)
@pkg/__init__.py
from . import sub
VALUE = sub.f(1, 2)
@pkg/sub.py
def f(a, b):
    return a + b
====
import_pkg_sub|
@__main__.py
import pkg.sub
print(pkg.sub.f(1, 2))
@pkg/__init__.py
@pkg/sub.py
def f(a, b):
    return a + b
====
from_pkg_sub_import_f|
@__main__.py
from pkg.sub import f
print(f(1, 2))
@pkg/__init__.py
@pkg/sub.py
def f(a, b):
    return a + b
====
control_attribute_name|pkg/sub.py
@__main__.py
from pkg import helper
print(helper(1, 2))
@pkg/__init__.py
def helper(a, b):
    return a + b
@pkg/sub.py
def f(a, b):
    return a + b
====
import_a_b_c|
@__main__.py
import a.b.c
print(a.b.c.f())
@a/__init__.py
@a/b/__init__.py
@a/b/c.py
def f():
    return 1
====
namespace_from_import|
@__main__.py
from ns import mod
print(mod.f())
@ns/mod.py
def f():
    return 1
====
relative_parent_package|
@__main__.py
from app.sub import mod
print(mod.f())
@app/__init__.py
@app/sub/__init__.py
@app/sub/mod.py
from ..util import g

def f():
    return g()
@app/util.py
def g():
    return 5
====
package_main_relative|
@pkg/__main__.py
from . import cli
cli.run()
@pkg/__init__.py
@pkg/cli.py
def run():
    print("ok")
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
            let owned = |s: &str| s.to_string();
            let files = body
                .split('@')
                .filter(|f| !f.is_empty())
                .map(|f| {
                    let (path, text) = f.split_once('\n').expect("path line");
                    (owned(path), owned(text))
                })
                .collect();
            let dead = dead
                .split(',')
                .filter(|d| !d.is_empty())
                .map(owned)
                .collect();
            (owned(name), dead, files)
        })
        .collect()
}

fn seeded(name: &str, files: &[(String, String)]) -> std::path::PathBuf {
    let dir = tmp(&format!("pyimports-{name}"));
    for (path, text) in files {
        let at = dir.join(path);
        std::fs::create_dir_all(at.parent().expect("parent")).expect("dirs");
        std::fs::write(at, text).expect("write");
    }
    init_and_commit(&dir, "seed");
    dir
}

fn dead_paths(dir: &Path, core: &str) -> Vec<String> {
    let report = deadcode::run(dir, None, core).expect("deadcode");
    report.dead.iter().map(|d| d.path.clone()).collect()
}

/// Every tree that runs: the dead set is exactly the expected one (empty
/// but for the control).
#[test]
fn a_python_program_that_runs_keeps_its_modules_alive() {
    let core = core_bin();
    for (name, want, files) in trees() {
        let dir = seeded(&name, &files);
        assert_eq!(dead_paths(&dir, &core), want, "{name}");
    }
}

/// `ce erase` never plans a live module: a package whose `__init__.py`
/// pulls in `hooks` relatively, the module holding only private names /
/// an empty `__all__` / side effects — the 1.9.2 plan deleted it.
#[test]
fn erase_plans_no_live_module() {
    let core = core_bin();
    let bodies = [
        "def _register(a, b):\n    return a + b\n\n_register(1, 2)\n",
        "__all__ = []\n\ndef _setup():\n    return 1\n\n_setup()\n",
        "import sys\nsys.modules.setdefault('marker', sys)\nREADY = True\n",
    ];
    for (i, body) in bodies.iter().enumerate() {
        let files = [
            (
                "__main__.py".to_string(),
                "import pkg\nprint('ok')\n".to_string(),
            ),
            (
                "pkg/__init__.py".to_string(),
                "from . import hooks\n".to_string(),
            ),
            ("pkg/hooks.py".to_string(), body.to_string()),
        ];
        let dir = seeded(&format!("erase{i}"), &files);
        assert!(dead_paths(&dir, &core).is_empty(), "variant {i}: dead set");
        let plan = erase::plan(&dir, None, &core).expect("plan");
        assert!(
            plan.rows.iter().all(|r| r.path != "pkg/hooks.py"),
            "variant {i}: the plan names the live module"
        );
    }
}
