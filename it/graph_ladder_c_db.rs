//! The C-family compile database completed (plan v2.30 step 5b, item
//! 14): one habitat, one database, one case table. What the rows pin —
//! the preprocessor's class order (`-iquote` answers the quoted form
//! before `-I`, the system form never asks it), `-I-` inhibiting the
//! including file's own directory, a header answered through the
//! translation units whose include closure reaches it (two chains
//! answering two files refuse; a declared root settles it first), the
//! MSVC include stack against the GNU dialect on the same tree, a
//! framework directory, a response file, a quoted argument keeping a
//! directory's space, a forced include seeding the closure, and `compile_flags.txt` covering the
//! files whose nearest database directory holds it — a nearer JSON
//! database with no entry wins over it. The database sits where clangd
//! probes, never in `Scope::configs`.

use codeeraser::graph::deadcode;
use codeeraser::scan::lang::Lang;

use crate::common::{self, compdb, fixture, text_ladder};

#[test]
fn c_compile_database_rungs_resolve_and_refuse() {
    text_ladder(Lang::C, HABITAT);
}

const HABITAT: &str = r#"
==== c/src/main.c
#include "v.h"
#include "shared.h"
==== c/src/other.c
#include "twin.h"
#include "shared.h"
==== c/src/shared.h
#include "deep.h"
#include "one.h"
==== c/src/util.h
==== c/third/vendor/v.h
==== c/alt/twin.h
==== c/include/twin.h
==== c/inc1/deep.h
==== c/inc2/deep.h
==== c/inc1/one.h
==== c/nodir/a.c
#include "b.h"
#include "only_here.h"
==== c/nodir/b.h
==== c/nodir/only_here.h
==== c/q/b.h
==== c/msvc/app.c
#include "a/one.h"
==== c/msvc/a/one.h
#include "two.h"
==== c/msvc/two.h
==== c/gnu/app.c
#include "a/one.h"
==== c/gnu/a/one.h
#include "two.h"
==== c/gnu/two.h
==== c/fw/app.c
#include <Foo/Bar.h>
==== c/fw/Frameworks/Foo.framework/Headers/Bar.h
==== c/rsp/r.c
#include "h.h"
==== c/rsp/flags.rsp
-I ../c/rsp/inc
==== c/rsp/inc/h.h
==== c/quoted/s.c
#include "q.h"
==== c/sp ace/q.h
==== c/pre/unit.c
int x;
==== c/pre/prefix.h
#include "cfg.h"
==== c/pre/loose.h
#include "cfg.h"
==== c/pre/inc/cfg.h
==== c/flags/x.c
#include "f.h"
==== c/flags/sub/y.c
#include "f.h"
==== c/flags/inc/f.h
==== c/flags/compile_flags.txt
-I
inc
==== c/flags/json/z.c
#include "f.h"
==== c/flags/json/compile_commands.json
[]
==== @compdb
c/src/main.c @@ cc -I ../c/third/vendor -I ../c/inc1 -c ../c/src/main.c
c/src/other.c @@ cc -I ../c/alt -iquote ../c/include -I ../c/inc2 -c ../c/src/other.c
c/nodir/a.c @@ cc -I ../c/q -I- -c ../c/nodir/a.c
c/msvc/app.c @@ cl /c ..\c\msvc\app.c
c/gnu/app.c @@ cc -c ../c/gnu/app.c
c/fw/app.c @@ cc -F ../c/fw/Frameworks -c ../c/fw/app.c
c/rsp/r.c @@ cc @../c/rsp/flags.rsp -c ../c/rsp/r.c
c/quoted/s.c @@ cc "-I../c/sp ace" -c ../c/quoted/s.c
c/pre/unit.c @@ cc -include ../c/pre/prefix.h -I ../c/pre/inc -c ../c/pre/unit.c
==== @cases
include @@ c/src/main.c @@ shared.h @@ ok c/src/shared.h 1
include @@ c/src/main.c @@ v.h @@ ok c/third/vendor/v.h 3
include @@ c/src/other.c @@ twin.h @@ ok c/include/twin.h 3
include @@ c/src/other.c @@ <twin.h> @@ ok c/alt/twin.h 3
include @@ c/src/shared.h @@ deep.h @@ no ambiguous_root
include @@ c/src/shared.h @@ one.h @@ ok c/inc1/one.h 3
include @@ c/src/util.h @@ deep.h @@ no out_of_scope
include @@ c/nodir/a.c @@ b.h @@ ok c/q/b.h 3
include @@ c/nodir/a.c @@ only_here.h @@ no out_of_scope
include @@ c/msvc/a/one.h @@ two.h @@ ok c/msvc/two.h 3
include @@ c/gnu/a/one.h @@ two.h @@ no out_of_scope
include @@ c/fw/app.c @@ <Foo/Bar.h> @@ ok c/fw/Frameworks/Foo.framework/Headers/Bar.h 3
include @@ c/rsp/r.c @@ h.h @@ ok c/rsp/inc/h.h 3
include @@ c/quoted/s.c @@ q.h @@ ok c/sp ace/q.h 3
include @@ c/pre/prefix.h @@ cfg.h @@ ok c/pre/inc/cfg.h 3
include @@ c/pre/loose.h @@ cfg.h @@ no out_of_scope
include @@ c/flags/x.c @@ f.h @@ ok c/flags/inc/f.h 3
include @@ c/flags/sub/y.c @@ f.h @@ ok c/flags/inc/f.h 3
include @@ c/flags/json/z.c @@ f.h @@ no out_of_scope
==== @rooted c/inc2
include @@ c/src/shared.h @@ deep.h @@ ok c/inc2/deep.h 2
"#;

/// The database in a gitignored `build/` — the walk never enters it,
/// and before this batch it answered for nobody — is found by the
/// probe; its `-include` is an arc the source text never spells, so
/// the prefix header and what it includes live. Deleting the database
/// re-fires the sweep (its bytes are a key input) and both die.
#[test]
fn a_gitignored_database_and_its_forced_include_are_read() {
    let core = common::core_bin();
    let fx = fixture(
        "ladder-c-db-deadcode",
        &[
            ("ce.toml", "[graph]\nentry_globs = [\"pre/*.c\"]\n"),
            (".gitignore", "build/\n"),
            ("pre/unit.c", "int x;\n"),
            ("pre/prefix.h", "#include \"cfg.h\"\n"),
            ("pre/inc/cfg.h", "\n"),
            ("pre/loose.h", "\n"),
        ],
    );
    compdb(
        &fx.dir,
        "pre/unit.c @@ cc -include ../pre/prefix.h -I ../pre/inc -c ../pre/unit.c",
    );
    let dead = |r: deadcode::Report| {
        let mut paths: Vec<String> = r.dead.iter().map(|d| d.path.clone()).collect();
        paths.sort();
        paths
    };
    let with = deadcode::run(&fx.dir, None, &core).expect("with the database");
    assert_eq!(dead(with), ["pre/loose.h"]);
    std::fs::remove_file(fx.dir.join("build/compile_commands.json")).expect("drop the database");
    let without = deadcode::run(&fx.dir, None, &core).expect("without the database");
    assert_eq!(
        dead(without),
        ["pre/inc/cfg.h", "pre/loose.h", "pre/prefix.h"]
    );
}
