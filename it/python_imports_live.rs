//! Python imports reach what CPython executes (lane v235_pyimport, proto
//! 9.1.0): every tree below RUNS under CPython 3.13 (each was run when
//! the lane was cut — the probes in its scratch notes; the test itself
//! does not invoke Python), so `ce deadcode` must report none of its live
//! files. The 1.9.2 reports read four of them dead [vouched] — a module
//! imported as a NAME (`from pkg import sub`, `from . import sub`) and
//! the ancestor packages a dotted import executes (`import pkg.sub`) —
//! and `ce erase` planned to delete a live module. The one dead file
//! left is the control: `pkg/sub.py` that nothing imports stays dead
//! (an attribute name, `helper`, draws no edge). A star import (lane
//! v235_graphtrust, T3) imports the submodules its package's literal
//! `__all__` names — absolute and relative; an attribute-only `__all__`
//! is the second control — and an `__all__` no literal reads leaves its
//! package's unnamed module dead but unvouched, never eraseable; a
//! literal list augmented by what no literal reads keeps the literal
//! names' modules live and unvouches the rest.
//!
//! One literal per tree: a header `name|dead,…` then `@path` lines, each
//! followed by the file's text (a tuple table rhymes under the clone
//! gate).

use crate::common::core_bin;
use crate::common::trees::{dead_sets_hold, judged, reasons, seeded};
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
====
star_literal_all|
@__main__.py
from pkg import *
print(sub.VALUE)
@pkg/__init__.py
__all__ = ['sub']
@pkg/sub.py
VALUE = 1
====
star_relative_literal_all|
@__main__.py
import pkg.inner
print(pkg.inner.sub.VALUE)
@pkg/__init__.py
__all__ = ('sub', 'helper')
helper = 1
@pkg/inner.py
from . import *
@pkg/sub.py
VALUE = 1
====
star_attribute_only|pkg/sub.py
@__main__.py
from pkg import *
print(helper)
@pkg/__init__.py
__all__ = ['helper']
helper = 1
@pkg/sub.py
VALUE = 1
"#;

/// A star import whose package's `__all__` no literal reads: the names
/// it imports are unknown, so the module it may import is dead but
/// unvouched, and the plan refuses it for the language's unresolved row.
/// A literal `__all__` augmented by what no literal reads keeps its
/// literal names' edges (`pkg/sub.py` is live) and still unvouches the
/// rest (`pkg/other.py`, which the comprehension imports).
const OPAQUE: &str = r#"star_opaque_all|pkg/sub.py
@__main__.py
from pkg import *
print(sub.VALUE)
@pkg/__init__.py
__all__ = ['s' + 'ub']
@pkg/sub.py
VALUE = 1
====
star_augmented_all|pkg/other.py
@__main__.py
from pkg import *
print(sub.VALUE, other.VALUE)
@pkg/__init__.py
__all__ = ['sub']
__all__ += [n for n in ('other',)]
@pkg/sub.py
VALUE = 1
@pkg/other.py
VALUE = 2
"#;

fn dead_paths(dir: &Path, core: &str) -> Vec<String> {
    let report = deadcode::run(dir, None, core).expect("deadcode");
    report.dead.iter().map(|d| d.path.clone()).collect()
}

/// Every tree that runs: the dead set is exactly the expected one (empty
/// but for the control).
#[test]
fn a_python_program_that_runs_keeps_its_modules_alive() {
    dead_sets_hold("pyimports", TREES, &core_bin());
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
        let dir = seeded(&format!("pyimports-erase{i}"), &files);
        assert!(dead_paths(&dir, &core).is_empty(), "variant {i}: dead set");
        let plan = erase::plan(&dir, None, &core).expect("plan");
        assert!(
            plan.rows.iter().all(|r| r.path != "pkg/hooks.py"),
            "variant {i}: the plan names the live module"
        );
    }
}

/// The opaque `__all__`: dead, unvouched, refused by name.
#[test]
fn an_opaque_all_unvouches_its_star_import() {
    let core = core_bin();
    for tree in judged("pyimports", OPAQUE, &core) {
        assert_eq!(
            tree.dead,
            [(tree.want[0].clone(), Some(0))],
            "{}",
            tree.name
        );
        assert_eq!(
            reasons(&tree.dir, &core),
            ["language_unresolved"],
            "{}",
            tree.name
        );
    }
}
