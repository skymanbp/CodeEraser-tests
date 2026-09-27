//! C-family ladder fixtures (plan v2.30 step 2): the include site's
//! rungs the tree alone answers - the including file's own directory,
//! the declared `[graph.search_roots] c` roots, and External for a
//! system header no rung holds; the compile-database rung has its own
//! habitat and battery (graph_ladder_c_db.rs, step 5b item 14). A
//! habitat of its own beside graph_ladder.rs: that file sits at the
//! E01 hard line since the sixth language. Ambiguity rows MUST refuse:
//! a name two declared roots hold that resolves is the red condition.

use codeeraser::graph::ladder::Reason;
use codeeraser::scan::lang::Lang;

use crate::common::{Case, ext, fixture, no, ok, run_cases};

/// C rows — the quoted form's own-directory rung, the declared roots
/// (`search_roots` `c` = ROOTS), the system form skipping the first
/// rung, External for a system header no declared root holds,
/// out_of_scope for a quoted one, and the empty `<>`; a header
/// includes through the same roots (C++ shares the key).
fn c_cases() -> Vec<Case> {
    let (c, inc, m) = (Lang::C, "include", "c/src/main.c");
    vec![
        (c, inc, m, "util.h", ok("c/src/util.h", 1)),
        (
            c,
            inc,
            "c/src/sub/deep.c",
            "../util.h",
            ok("c/src/util.h", 1),
        ),
        (c, inc, m, "lib.h", ok("c/include/lib.h", 2)),
        (c, inc, m, "twin.h", no(Reason::AmbiguousRoot)),
        (c, inc, m, "<lib.h>", ok("c/include/lib.h", 2)),
        (c, inc, m, "<util.h>", ext(4)),
        (c, inc, m, "<stdio.h>", ext(4)),
        (c, inc, m, "missing.h", no(Reason::OutOfScope)),
        (c, inc, m, "<>", no(Reason::Empty)),
        (
            Lang::Cpp,
            inc,
            "c/src/util.h",
            "lib.h",
            ok("c/include/lib.h", 2),
        ),
    ]
}

/// R1–R2 habitat: the including file's own directory (a `../` spec
/// walks up), two declared roots, and a name both of them hold.
const TREE: [(&str, &str); 6] = [
    ("c/src/main.c", "#include \"util.h\"\n"),
    ("c/src/util.h", "\n"),
    ("c/src/sub/deep.c", "\n"),
    ("c/include/lib.h", "\n"),
    ("c/include/twin.h", "\n"),
    ("c/alt/twin.h", "\n"),
];

/// The declared `[graph.search_roots] c` of the habitat above.
const ROOTS: [&str; 2] = ["c/include", "c/alt"];

#[test]
fn c_rungs_resolve_and_refuse() {
    let mut fx = fixture("ladder-c", &TREE);
    fx.search_roots
        .insert("c".to_string(), ROOTS.map(String::from).into());
    run_cases(&fx, c_cases());
}
