//! C-family ladder fixtures (plan v2.30 step 2): the include site's
//! four rungs — the including file's own directory, the declared
//! `[graph.search_roots] c` roots, the including file's own
//! compile_commands.json entry, and External for a system header no
//! rung holds. A habitat of its own beside graph_ladder.rs: that file
//! sits at the E01 hard line since the sixth language, and the
//! compile-database rung needs a build directory the shared tree has
//! no reason to carry. Ambiguity rows MUST refuse: a name two declared
//! roots hold that resolves is the red condition.

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

/// The compile-database rung (C R3): the including file's own entry
/// names its include directories — absolute, or relative to the
/// entry's directory, in either argument spelling — first hit in
/// invocation order; a file with no entry (a header, a unit the
/// database omits) reads nothing there.
#[test]
fn c_compile_database_rung_reads_the_including_files_entry() {
    let mut fx = fixture(
        "ladder-compdb",
        &[
            ("c/src/main.c", "#include \"v.h\"\n"),
            ("c/src/other.c", "#include \"twin.h\"\n"),
            ("c/src/util.h", "\n"),
            ("c/third/vendor/v.h", "\n"),
            ("c/alt/twin.h", "\n"),
            ("c/include/twin.h", "\n"),
        ],
    );
    let root = fx.dir.to_string_lossy().replace('\\', "/");
    let db = format!(
        "[{{\"directory\": \"{root}/build\", \"file\": \"{root}/c/src/main.c\", \
         \"arguments\": [\"cc\", \"-I{root}/c/third/vendor\", \"-c\", \"{root}/c/src/main.c\"]}},\n \
         {{\"directory\": \"{root}/build\", \"file\": \"../c/src/other.c\", \
         \"command\": \"cc -I ../c/alt -iquote ../c/include -c ../c/src/other.c\"}}]\n"
    );
    std::fs::create_dir_all(fx.dir.join("build")).expect("build dir");
    std::fs::write(fx.dir.join("build/compile_commands.json"), db).expect("compdb");
    fx.configs.push("build/compile_commands.json".to_string());
    let (c, inc, m) = (Lang::C, "include", "c/src/main.c");
    run_cases(
        &fx,
        vec![
            (c, inc, m, "v.h", ok("c/third/vendor/v.h", 3)),
            (c, inc, m, "<v.h>", ok("c/third/vendor/v.h", 3)),
            (c, inc, "c/src/other.c", "twin.h", ok("c/alt/twin.h", 3)),
            (
                Lang::Cpp,
                inc,
                "c/src/util.h",
                "v.h",
                no(Reason::OutOfScope),
            ),
        ],
    );
}
