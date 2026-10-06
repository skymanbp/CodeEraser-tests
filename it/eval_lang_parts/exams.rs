//! The exam table of the v2.30 language exams — split from mod.rs
//! (which keeps the types, the doc families and the pre-registered
//! constants) when the C family's two rows took it past the 300-line
//! soft line.

use super::Exam;
use super::Reach::{Tree, Universe};
use super::Stage::Scored;

/// The C family's rungs and the compilation-database readers they
/// consult: one ladder for both exams, since a C++ include resolves
/// exactly as a C one. Since plan v2.33 wave W2a the search runs in the
/// core (CE.Resolve.C / CE.Resolve.CIndex), and since W2-text so do the
/// database, command-line and flag readers (CE.Resolve.CompDb / Cmdline
/// / Flags); this side probes for the databases (compdb_find.rs), reads
/// the include lists (ladder/c_head.rs) and sends what it read
/// (graph/resolve/), so all of them answer the C family's docs.
const C_LADDER: &[&str] = &[
    "cli/src/graph/ladder/c*",
    "cli/src/graph/compdb*",
    "cli/src/graph/resolve/",
    "core/app/CE/Resolve.hs",
    "core/app/CE/Resolve/",
];

/// The Lua ladder: the `package.path` reader here, the search in the
/// core since plan v2.33 wave W2a (CE.Resolve.Lua), the request between
/// them.
const LUA_LADDER: &[&str] = &[
    "cli/src/graph/ladder/lua*",
    "cli/src/graph/resolve/",
    "core/app/CE/Resolve.hs",
    "core/app/CE/Resolve/",
];

/// The R ladder: in the core since plan v2.33 W2-text stage B
/// (CE.Resolve.R, the DESCRIPTION reader CE.Resolve.Description), the
/// request that carries each DESCRIPTION's text between them.
const R_LADDER: &[&str] = &[
    "cli/src/graph/resolve/",
    "core/app/CE/Resolve.hs",
    "core/app/CE/Resolve/",
];

/// The Java ladder: the header and type readers here (the walk reads
/// every Java file's head with them, ladder/java_header.rs and
/// java_types.rs), the rungs in the core since plan v2.33 W2-text stage
/// C (CE.Resolve.Java and its helpers), the request that carries each
/// header between them.
const JAVA_LADDER: &[&str] = &[
    "cli/src/graph/ladder/java*",
    "cli/src/graph/resolve/",
    "core/app/CE/Resolve.hs",
    "core/app/CE/Resolve/",
];

/// Why the C family's ladder precedes its sample (Exam::ladder_first).
const C_LADDER_FIRST: &str = "step 2 landed the C family's ladder (b7e78c7, 2026-09-24) before any \
    exam of it existed; booklet section 14 item 14 fills the C/C++ three-piece set in at step 6, \
    so the ladder precedes the sample by construction and the blindness rests on the process \
    alone - the auditors read the pinned clone and their batch, never a parse";

/// A scored exam whose truths reach its own universe and whose ladder
/// landed after its audit — the shape of every row but the ones that
/// say otherwise below it. (A row per exam spelled out in full rhymes
/// with the next at the clone gate's window; the varying columns alone
/// are what a row states.)
const fn exam(
    lang: &'static str,
    corpora: &'static [(&'static str, &'static str)],
    exts: &'static [&'static str],
    ladder: &'static [&'static str],
    generation: u32,
) -> Exam {
    Exam {
        lang,
        corpora,
        exts,
        reach: Universe,
        ladder,
        ladder_first: None,
        stage: Scored,
        generation,
    }
}

/// Every language whose exam is frozen, in landing order; a language
/// joins with its step (booklet §13) and never leaves. A second corpus
/// joins when the first holds none of a site kind: gson has no wildcard
/// import (its style guide forbids them), jsoup brings them. Lua and R
/// take two from the start (booklet §14 item 17), a package and an
/// application apiece: a package reaches its own files by module name,
/// an application by path (`dofile`, `source`). The R ladder is a
/// directory of its own — a `r*` pathspec would hold the Rust rungs.
/// Lua's exam is at its second generation: the first was frozen before
/// the detector read a load under protection (`pcall(require, "x")`,
/// the core's CE.Lang.Lua `protected` table).
/// HTML (step 5, the document language the user upgraded to a judged
/// one) takes this repository's own pages — `site/`, the GUI page and
/// the demo scoreboards at the tip that closed step 4 (booklet §11:
/// self-feeding) — h5bp/html5-boilerplate, and mdn/learning-area (the
/// user's ruling for a third: 269 teaching pages holding the forms and
/// `srcset` lists the first two hold none of): a page
/// reaches another by `href` and its assets by `src`. Its universes
/// are the files the product's own walk reads (generate.rs walk).
/// Java, Lua and R were unscored from step 5's first commit to its
/// last: that step moved the code their answers come from (the walk's
/// build-output rule, Java's source sets and own units, Lua's own
/// directory — lang_provenance.rs holds a doc to it), so their docs
/// were generated again once, beside HTML's, after the HTML ladder
/// (step 5 commit B′).
/// C and C++ (step 6; booklet §14 item 14) are the one exam pair whose
/// ladder came first — step 2 committed the C family's rungs before any
/// exam existed — so each row says so (`ladder_first`) and the ordering
/// gate reads the reversal as a fact to check, not a rule to relax.
/// They take the crosscheck corpora of booklet §11, lua/lua for C and
/// fmtlib/fmt for C++, one apiece: `include` is the one site kind, and
/// a `.h` is C++ by the product's extension table, so the interpreter's
/// headers belong to no C exam and fmt's headers, sources and tests are
/// one universe. The C exam's truths therefore reach the pinned tree
/// (Reach::Tree, like HTML's): a `.c` includes the interpreter's `.h`,
/// a file outside the `*.c` universe, and the audit names it.
pub const EXAMS: [Exam; 6] = [
    exam(
        "java",
        &[
            ("gson", "854c8255b625cf1e13c701a83ea9ccb4caaa576a"),
            ("jsoup", "093e2f58492c531667e551e8793513a41b22443e"),
        ],
        &["java"],
        JAVA_LADDER,
        1,
    ),
    exam(
        "lua",
        &[
            ("luarocks", "2d2cc8eff2f03c23d142f8059146fb241dcf56b5"),
            ("koreader", "d9cd2788e4ec023b8fbf60b0982e831c82d15a44"),
        ],
        &["lua"],
        LUA_LADDER,
        2,
    ),
    exam(
        "r",
        &[
            ("stringr", "ae054b1d28f630fee22ddb3cb7525396e62af4fe"),
            ("covid19model", "fcc30e2b8d046ddf3ef10dfc222e42b5cd732622"),
        ],
        &["R", "r"],
        R_LADDER,
        1,
    ),
    Exam {
        reach: Tree,
        ..exam(
            "html",
            &[
                ("codeeraser", "d4b7f1f37aa50b61ecd21816204bb7f5b06d673c"),
                (
                    "html5-boilerplate",
                    "b6597338e695dc4a8165c5abcb2bfffa586c3ee5",
                ),
                ("learning-area", "dbed6bcb8284634c7549c4da596ec30b0cfc6e7e"),
            ],
            &["html", "htm"],
            &["cli/src/graph/ladder/html*"],
            1,
        )
    },
    Exam {
        reach: Tree,
        ladder_first: Some(C_LADDER_FIRST),
        ..exam(
            "c",
            &[("lua", "0b29f408433e92953cc72b1d3e06c7ac8139e439")],
            &["c"],
            C_LADDER,
            1,
        )
    },
    Exam {
        ladder_first: Some(C_LADDER_FIRST),
        ..exam(
            "cpp",
            &[("fmt", "6d71f74624be5daa548073ff8e4e0c8aa5476010")],
            &["cpp", "cc", "cxx", "hpp", "hh", "hxx", "h", "inl"],
            C_LADDER,
            1,
        )
    },
];
