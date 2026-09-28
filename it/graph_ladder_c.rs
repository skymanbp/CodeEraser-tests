//! C-family ladder fixtures (plan v2.30 step 2; the `include` rung in
//! step 6): the include site's rungs the tree alone answers - the
//! including file's own directory, the declared `[graph.search_roots]
//! c` roots, the `include` directory beside the includer's own
//! directory or beside an ancestor of it, and External for a system
//! header no rung holds; the compile-database rung has its own habitat
//! and battery (graph_ladder_c_db.rs, step 5b item 14), where a file
//! the database compiles is also shown to get no `include` rung. A
//! habitat of its own beside graph_ladder.rs: that file sits at the
//! E01 hard line since the sixth language; one text per language, the
//! db habitat's shape (common::text_ladder). Ambiguity rows MUST
//! refuse: a name two declared roots hold, or two `include` directories
//! on the includer's ancestry hold, that resolves is the red condition.

use codeeraser::scan::lang::Lang;

use crate::common::{fixture, ok, run_cases, text_ladder};

#[test]
fn c_rungs_resolve_and_refuse() {
    text_ladder(Lang::C, HABITAT);
}

/// A header under the C++ extension table includes through the
/// declared `c` roots — the key is C's and is spelled so here: a
/// `@rooted` block of a C++ text would declare the directory under
/// `cpp`, a key the ladder never reads, and the `include` rung would
/// answer instead of the declared one.
#[test]
fn cpp_shares_the_c_roots() {
    let mut fx = fixture(
        "ladder-cpp-roots",
        &[("c/src/util.h", "\n"), ("c/include/lib.h", "\n")],
    );
    fx.search_roots
        .insert("c".to_string(), ["c/include".to_string()].into());
    let row = (
        Lang::Cpp,
        "include",
        "c/src/util.h",
        "lib.h",
        ok("c/include/lib.h", 2),
    );
    run_cases(&fx, vec![row]);
}

/// R1, R2, R4 and R5 on one tree. The `@cases` rows run with nothing
/// declared: the quoted form's own-directory rung (a `../` spec walks
/// up), the `include` rung for a project layout (`c/proj/include` for
/// `c/proj/src`, both forms; from `c/proj/sub/src` an `include` nearer
/// the includer that lacks the name is walked past to the one that
/// holds it, and two on the ancestry holding two files refuse), the
/// tree root's own `include`, External for a system header no rung
/// holds, out_of_scope for a quoted one, and the empty `<>`. The
/// `@rooted` rows declare `c/include` and `c/alt`: a declared root
/// answers before the `include` rung — `c/include` is also the
/// `include` beside `c/src`'s parent, and `lib.h` resolves at rung 2 —
/// and a name both declared roots hold refuses at rung 2.
const HABITAT: &str = r#"
==== c/src/main.c
#include "util.h"
==== c/src/util.h
==== c/src/sub/deep.c
==== c/include/lib.h
==== c/include/twin.h
==== c/alt/twin.h
==== c/proj/src/a.c
#include "lib/x.h"
==== c/proj/include/lib/x.h
==== c/proj/include/lib/y.h
==== c/proj/sub/src/b.c
==== c/proj/sub/include/lib/y.h
==== include/top.h
==== @cases
include @@ c/src/main.c @@ util.h @@ ok c/src/util.h 1
include @@ c/src/sub/deep.c @@ ../util.h @@ ok c/src/util.h 1
include @@ c/src/main.c @@ lib.h @@ ok c/include/lib.h 4
include @@ c/proj/src/a.c @@ lib/x.h @@ ok c/proj/include/lib/x.h 4
include @@ c/proj/src/a.c @@ <lib/x.h> @@ ok c/proj/include/lib/x.h 4
include @@ c/proj/sub/src/b.c @@ lib/x.h @@ ok c/proj/include/lib/x.h 4
include @@ c/proj/sub/src/b.c @@ lib/y.h @@ no ambiguous_root
include @@ c/src/main.c @@ top.h @@ ok include/top.h 4
include @@ c/src/main.c @@ <util.h> @@ ext 5
include @@ c/src/main.c @@ <stdio.h> @@ ext 5
include @@ c/src/main.c @@ missing.h @@ no out_of_scope
include @@ c/src/main.c @@ <> @@ no empty
==== @rooted c/include c/alt
include @@ c/src/main.c @@ lib.h @@ ok c/include/lib.h 2
include @@ c/src/main.c @@ <lib.h> @@ ok c/include/lib.h 2
include @@ c/src/main.c @@ twin.h @@ no ambiguous_root
include @@ c/src/main.c @@ top.h @@ ok include/top.h 4
"#;
