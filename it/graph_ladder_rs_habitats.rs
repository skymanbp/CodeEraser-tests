//! Rust habitats with trees of their own (split from graph_ladder.rs at
//! the E01 hard line, step 5b): inline `mod` bodies, `#[path]` remaps
//! and the re-export facade. graph_ladder.rs keeps the shared tree's
//! rows; these need fixtures shaped around one construct each.

use codeeraser::graph::ladder::{self, Outcome, Reason};
use codeeraser::scan::lang::Lang;

use crate::common::{fixture, no, ok, site, via};

/// Every spec from src/lib.rs rides the R4 member rung into the facade,
/// then the binder speaks or refuses — one materialized tree per
/// battery (parallel tests must not share a fixture directory).
fn facade_answers(tag: &str, rows: &[(&'static str, Outcome)]) {
    let fx = fixture(tag, REEXPORT_TREE);
    let scope = fx.scope();
    for (spec, want) in rows {
        let got = ladder::resolve(Lang::Rust, &site("use", "src/lib.rs", spec, 1), &scope);
        assert_eq!(&got, want, "{spec}");
    }
}

/// The re-export habitat (§4 R5 as amended 2026-08-18): a facade
/// whose nested `pub use` binds a remaining segment answers the
/// DEFINITION file as ResolvedVia — one hop, bind-free inside. Since
/// step 5b a glob IS followed (the one file whose pub items or pub
/// uses bear the name) and a `pub extern crate` binds its crate root,
/// alias included. Refusal rows: two matching entries never pick, a
/// name two globs' files both define or one only a private item
/// bears stays the facade, a locally-defined name never hops, and a
/// private `use` re-exports nothing.
const REEXPORT_TREE: &[(&str, &str)] = &[
    (
        "Cargo.toml",
        "[package]\nname = \"fac\"\n\n[dependencies]\nsearcher-lib = { path = \"crates/searcher\" }\n",
    ),
    ("src/lib.rs", "mod consumer;\n"),
    // the BinaryDetection shape: extern member -> lib root facade ->
    // nested group re-export -> definition file
    (
        "crates/searcher/Cargo.toml",
        "[package]\nname='searcher-lib'\n[dependencies]\ninner-lib = { path = \"../inner\" }\n",
    ),
    // the file OPENS with a bodied mod holding a same-named `uni` (step
    // 8 review): the hop reads its namespace at the pub use's own line,
    // never at line 1, so the file-level `mod uni;` answers
    (
        "crates/searcher/src/lib.rs",
        "mod shadow {\n    pub mod uni {\n        pub struct Uni;\n    }\n}\npub use crate::{searcher::{Binary, Config as Conf}, sink::*};\nuse crate::quiet::Hidden;\npub use crate::dup_a::Twice;\npub use crate::dup_b::Twice;\npub use crate::local::Local;\npub use uni::Uni;\npub use crate::glob_a::*;\npub use crate::glob_b::*;\npub extern crate inner_lib;\npub extern crate inner_lib as renamed;\nmod searcher;\nmod sink;\nmod quiet;\nmod dup_a;\nmod dup_b;\nmod local;\nmod uni;\nmod glob_a;\nmod glob_b;\npub struct Local;\n",
    ),
    ("crates/searcher/src/uni.rs", "pub struct Uni;\n"),
    (
        "crates/searcher/src/searcher.rs",
        "pub struct Binary;\npub struct Config;\n",
    ),
    ("crates/searcher/src/sink.rs", "pub struct Sink;\n"),
    ("crates/searcher/src/quiet.rs", "pub struct Hidden;\n"),
    ("crates/searcher/src/dup_a.rs", "pub struct Twice;\n"),
    ("crates/searcher/src/dup_b.rs", "pub struct Twice;\n"),
    ("crates/searcher/src/local.rs", "pub struct Local;\n"),
    // the glob habitat (step 5b): Both in two files, Secret private,
    // Shown a pub use inside a glob's file
    (
        "crates/searcher/src/glob_a.rs",
        "pub struct Both;\nstruct Secret;\npub struct OnlyA;\npub use crate::quiet::Hidden as Shown;\n",
    ),
    (
        "crates/searcher/src/glob_b.rs",
        "pub struct Both;\npub struct OnlyB;\n",
    ),
    // the `pub extern crate` habitat: a second member crate the facade
    // re-exports whole, under its name and an alias
    ("crates/inner/Cargo.toml", "[package]\nname='inner-lib'"),
    (
        "crates/inner/src/lib.rs",
        "pub struct Thing;\npub mod sub;\n",
    ),
    ("crates/inner/src/sub.rs", "pub struct Deep;\n"),
];

#[test]
fn reexport_binds_one_hop_to_the_definition() {
    let lib = "crates/searcher/src/lib.rs";
    facade_answers(
        "ladder-reexport",
        &[
            // nested group binds -> definition file, marked via
            (
                "searcher_lib::Binary",
                via("crates/searcher/src/searcher.rs", 4),
            ),
            // `as` binds the ALIAS, not the source name
            (
                "searcher_lib::Conf",
                via("crates/searcher/src/searcher.rs", 4),
            ),
            ("searcher_lib::Config", ok(lib, 4)),
            // two entries bind Twice — picking would invent a path
            ("searcher_lib::Twice", ok(lib, 4)),
            // the facade defines Local itself — definition wins
            ("searcher_lib::Local", ok(lib, 4)),
            // a private use re-exports nothing
            ("searcher_lib::Hidden", ok(lib, 4)),
            // a uniform-path facade `pub use uni::Uni` binds too (step 8,
            // ruling ④): the hop's bare head reads the facade's own
            // `mod uni;` — before it the head fell to the crate rung and
            // this row answered the facade
            ("searcher_lib::Uni", via("crates/searcher/src/uni.rs", 4)),
            // the R6 side door, guarded (step 8, O05/O15): a braced leaf
            // carrying its own path segment is cut at the brace — the
            // walk consumes the prefix alone, the facade file is the
            // edge, and neither `searcher` nor `Binary` is ever read off
            // the prefix file (the binder hops only on a prefix segment
            // the walk left unconsumed, never on a braced one)
            ("searcher_lib::{searcher::Binary, sink::Sink}", ok(lib, 4)),
            ("searcher_lib::{searcher::Binary}", ok(lib, 4)),
        ],
    );
}

/// A glob is followed (step 5b): the one glob whose file bears the name
/// pub binds it — a pub use of that file included; a name two globs'
/// files both define, or one only a private item bears, stays the
/// facade. A `pub extern crate` binds the crate root, under its alias
/// too, and the crate's own modules descend from there.
#[test]
fn globs_and_extern_crates_bind_through_the_facade() {
    let lib = "crates/searcher/src/lib.rs";
    facade_answers(
        "ladder-reexport-glob",
        &[
            ("searcher_lib::Sink", via("crates/searcher/src/sink.rs", 4)),
            (
                "searcher_lib::OnlyA",
                via("crates/searcher/src/glob_a.rs", 4),
            ),
            (
                "searcher_lib::Shown",
                via("crates/searcher/src/glob_a.rs", 4),
            ),
            ("searcher_lib::Both", ok(lib, 4)),
            ("searcher_lib::Secret", ok(lib, 4)),
            ("searcher_lib::inner_lib", via("crates/inner/src/lib.rs", 4)),
            (
                "searcher_lib::inner_lib::Thing",
                via("crates/inner/src/lib.rs", 4),
            ),
            (
                "searcher_lib::renamed::sub::Deep",
                via("crates/inner/src/sub.rs", 4),
            ),
        ],
    );
}

/// Inline `mod` bodies anchor self/super at the FILE, not its parent
/// (the audited interpolate.rs and globset rows): ups consume inline
/// depth before any file climb, self inside an inline module is a
/// self edge, and a post-climb tail may still name a FILE module.
#[test]
fn inline_mod_super_comes_home() {
    let fx = fixture(
        "ladder-inline",
        &[
            ("Cargo.toml", "[package]\nname='inl'"),
            (
                "src/lib.rs",
                "pub fn escape() {}\nmod util;\n#[cfg(test)]\nmod tests {\n    use super::escape;\n    use self::helper::x;\n    use super::util;\n    mod deep {\n        use super::super::escape;\n    }\n}\n",
            ),
            ("src/util.rs", "\n"),
        ],
    );
    let scope = fx.scope();
    let at = |spec: &'static str, line: usize| {
        ladder::resolve(Lang::Rust, &site("use", "src/lib.rs", spec, line), &scope)
    };
    assert_eq!(at("super::escape", 5), ok("src/lib.rs", 3));
    assert_eq!(at("self::helper::x", 6), ok("src/lib.rs", 3));
    assert_eq!(at("super::util", 7), ok("src/util.rs", 3));
    assert_eq!(at("super::super::escape", 9), ok("src/lib.rs", 3));
    // a bare head inside the inline module (step 8): the bodied
    // `mod deep` declared in `tests` is in scope and stays in this
    // file; the file-level `mod util;` is one namespace up, so the
    // head falls to the crate rung and nothing declares it
    assert_eq!(at("deep::x", 5), ok("src/lib.rs", 3));
    assert_eq!(at("util::x", 5), no(Reason::OutOfScope));
}

/// `#[path = "…"]` remaps answer at R1 (design §4 Rust row, R5
/// column: the literal answers at R1). Line-anchored like the inline
/// cases — the shared table drives line 1 only. Pinned here: the
/// FILE-LEVEL base is the declarer's OWN directory for every
/// declarer kind (never the convention child_dir — the one bug this
/// rung can have); the INLINE-module base is child_dir plus the
/// enclosing mod names (both rustc-reference habitats); raw-string
/// literals carry the same content node; attributes stack in either
/// order; `../` traverses while a repo escape or a missing target
/// refuses, never invents.
/// The #[path] habitat tree (the TREE convention: the fixture IS the
/// spec) — both declarer kinds, stacked attributes, a raw string,
/// traversal/escape/missing targets, and both inline-module bases.
const PATH_TREE: &[(&str, &str)] = &[
    ("Cargo.toml", "[package]\nname='pa'"),
    (
        "src/graph/md.rs",
        "#[cfg(test)]\n#[path = \"md_tests.rs\"]\nmod tests;\n#[path = \"also.rs\"]\n#[cfg(test)]\nmod also;\n#[path = \"../up.rs\"]\nmod up;\n#[path = \"../../../out.rs\"]\nmod esc;\n#[path = \"nope.rs\"]\nmod nope;\n#[path = r\"also.rs\"]\nmod raw;\nmod inline {\n    #[path = \"md_tests.rs\"]\n    mod hidden;\n    mod conv;\n    mod shallow;\n    use conv::Thing;\n}\n",
    ),
    ("src/graph/md_tests.rs", "\n"),
    ("src/graph/also.rs", "\n"),
    ("src/graph/md/inline/md_tests.rs", "\n"),
    // the convention habitat inside the inline mod (step 8 review):
    // rustc loads md/inline/conv.rs, never the shallow decoy md/conv.rs,
    // and a name with only the shallow file refuses
    ("src/graph/md/inline/conv.rs", "pub struct Thing;\n"),
    ("src/graph/md/conv.rs", "\n"),
    ("src/graph/md/shallow.rs", "\n"),
    ("src/up.rs", "\n"),
    ("src/lib.rs", "mod graph;\n"),
    (
        "src/graph/mod.rs",
        "mod md;\n#[path = \"store_tests.rs\"]\nmod tests;\nmod deep {\n    #[path = \"extra.rs\"]\n    mod e;\n}\n",
    ),
    ("src/graph/store_tests.rs", "\n"),
    ("src/graph/deep/extra.rs", "\n"),
];

#[test]
fn path_attr_remaps_resolve_at_r1() {
    let fx = fixture("ladder-path-attr", PATH_TREE);
    let scope = fx.scope();
    // (from, spec, line, want) — row order: own-dir base (never
    // child_dir), attribute stacking both ways, ../ traversal, repo
    // escape, missing target, raw string, inline habitat in both
    // declarer kinds, and the un-hijacked plain convention
    let cases: &[(&'static str, &'static str, usize, Outcome)] = &[
        (
            "src/graph/md.rs",
            "tests",
            3,
            ok("src/graph/md_tests.rs", 1),
        ),
        ("src/graph/md.rs", "also", 6, ok("src/graph/also.rs", 1)),
        ("src/graph/md.rs", "up", 8, ok("src/up.rs", 1)),
        ("src/graph/md.rs", "esc", 10, no(Reason::OutOfScope)),
        ("src/graph/md.rs", "nope", 12, no(Reason::OutOfScope)),
        ("src/graph/md.rs", "raw", 14, ok("src/graph/also.rs", 1)),
        (
            "src/graph/md.rs",
            "hidden",
            17,
            ok("src/graph/md/inline/md_tests.rs", 1),
        ),
        (
            "src/graph/mod.rs",
            "tests",
            3,
            ok("src/graph/store_tests.rs", 1),
        ),
        ("src/graph/mod.rs", "e", 6, ok("src/graph/deep/extra.rs", 1)),
        ("src/graph/mod.rs", "md", 1, ok("src/graph/md.rs", 1)),
        // the convention lookup shares the inline base (step 8 review)
        (
            "src/graph/md.rs",
            "conv",
            18,
            ok("src/graph/md/inline/conv.rs", 1),
        ),
        ("src/graph/md.rs", "shallow", 19, no(Reason::OutOfScope)),
    ];
    for (from, spec, line, want) in cases {
        let got = ladder::resolve(Lang::Rust, &site("mod_decl", from, spec, *line), &scope);
        assert_eq!(&got, want, "{from} {spec} @{line}");
    }
    // the bare head declared in the inline mod mounts through the same
    // base and descends from there
    assert_eq!(
        ladder::resolve(
            Lang::Rust,
            &site("use", "src/graph/md.rs", "conv::Thing", 20),
            &scope
        ),
        ok("src/graph/md/inline/conv.rs", 3)
    );
}
