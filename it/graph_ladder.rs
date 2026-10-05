//! Resolution-ladder fixtures (design §6 2f exit row: per rung ≥1
//! fixture resolving exactly at that rung, plus ambiguity fixtures
//! that MUST stay Unresolved — an ambiguous fixture that resolves is
//! the red condition). One shared tree and one case table drive
//! every import-shaped language — per-language table pairs were the
//! dedup ratchet's first catch in this file, and that discipline is
//! why the file sits over the E01 300 soft line since the sixth
//! language (3l): splitting per language would re-create the caught
//! pattern. The Markdown rungs read document CONTENT (headings,
//! definitions), so their doc-shaped habitat lives in
//! graph_ladder_md.rs; the tsconfig `extends` chains and the Rust
//! habitats with trees of their own (inline `mod`, `#[path]`, the
//! re-export facade) live in graph_ladder_ts_config.rs and
//! graph_ladder_rs_habitats.rs — this file met the 750 hard line in
//! step 5b.

use codeeraser::graph::ladder::{Outcome, Reason};
use codeeraser::scan::lang::Lang;

// two groups on purpose: the one-line form token-chained against
// graph_ladder_md.rs's header under T2 (the census's catch)
use crate::common::{Case, run_cases, via};
use crate::common::{ext, fixture, no, ok, pkg};

/// The shared fixture tree — every rung's habitat, all languages.
const TREE: &[(&str, &str)] = &[
    // TS R1: extension order + exact + traversal; widget.ts beats
    // widget/index.ts because the order IS the norm, not ambiguity
    ("src/app.ts", "export {};\n"),
    ("src/util.ts", "export {};\n"),
    ("src/widget.ts", "export {};\n"),
    ("src/widget/index.ts", "export {};\n"),
    ("shared/x.tsx", "export {};\n"),
    // TS R2: legacy.ts has no .js twin (rewrite fires); built.js
    // exists on disk (rewrite blocked)
    ("src/legacy.ts", "export {};\n"),
    ("src/built.ts", "export {};\n"),
    ("src/built.js", "// compiled artifact\n"),
    // TS R3: extends chain (base contributes baseUrl, child the
    // paths; JSONC comment + trailing comma exercised on purpose)
    (
        "tsconfig.json",
        "{\n  // jsonc on purpose\n  \"extends\": \"./tsconfig.base.json\",\n  \"compilerOptions\": {\n    \"paths\": {\n      \"@app/*\": [\"../src/*\"],\n      \"@dup/*\": [\"../src/dup_a/*\", \"../src/dup_b/*\"],\n    },\n  },\n}\n",
    ),
    (
        "tsconfig.base.json",
        "{\"compilerOptions\": {\"baseUrl\": \"./lib\"}}\n",
    ),
    ("lib/leaf.ts", "export {};\n"),
    ("src/dup_a/thing.ts", "export {};\n"),
    ("src/dup_b/thing.ts", "export {};\n"),
    // TS R4: one clean member, one duplicate-name pair, one member
    // whose exports conditions point at two distinct in-scope files;
    // subpath patterns (step 5b: the longest matching key wins, a
    // null target exports nothing) and a package's own node_modules
    // (a bare name it vendors is External from inside it alone)
    (
        "packages/pkga/package.json",
        "{\"name\": \"pkga\", \"exports\": {\".\": {\"source\": \"./src/index.ts\", \"default\": \"./dist/index.js\"}, \"./sub\": \"./src/sub.ts\", \"./lib/*\": \"./src/*.ts\", \"./lib/deep/*\": \"./src/nested/*.ts\", \"./lib/private\": null}}\n",
    ),
    ("packages/pkga/src/index.ts", "export {};\n"),
    ("packages/pkga/src/sub.ts", "export {};\n"),
    ("packages/pkga/src/nested/x.ts", "export {};\n"),
    ("packages/pkga/node_modules/vend/index.js", "// vendored\n"),
    ("packages/dupa/package.json", "{\"name\": \"dup-pkg\"}\n"),
    ("packages/dupb/package.json", "{\"name\": \"dup-pkg\"}\n"),
    (
        "packages/pkgb/package.json",
        "{\"name\": \"pkgb\", \"exports\": {\".\": {\"a\": \"./one.ts\", \"b\": \"./two.ts\"}}}\n",
    ),
    ("packages/pkgb/one.ts", "export {};\n"),
    ("packages/pkgb/two.ts", "export {};\n"),
    // TS R5: declared dependency vs physically vendored vs nothing
    (
        "package.json",
        "{\"name\": \"rootpkg\", \"dependencies\": {\"lodash\": \"^4\"}}\n",
    ),
    ("node_modules/leftover/index.js", "// vendored\n"),
    // Py R1-R3: package chain; R2 ambiguity needs the same dotted
    // path under two roots; pyproject declares one dependency (R4)
    ("pkg/__init__.py", "\n"),
    ("pkg/consumer.py", "\n"),
    ("pkg/mod.py", "\n"),
    ("pkg/sub/__init__.py", "\n"),
    ("pkg/sub/leaf.py", "\n"),
    ("other.py", "\n"),
    ("top.py", "\n"),
    ("tool/__init__.py", "\n"),
    ("src/tool/__init__.py", "\n"),
    (
        "pyproject.toml",
        "[project]\nname = \"fixture\"\ndependencies = [\"requests>=2.31\"]\n",
    ),
    // Rust R1-R3: lib+main double root (a walk ending AT the root
    // must refuse), a mod.rs chain, a 2018-style subdir, an E0761
    // double habitat, and a [[bin]] crate with its own tree
    (
        "Cargo.toml",
        "[package]\nname = \"rootcrate\"\n\n[dependencies]\nserde = \"1\"\nhelper-lib = { path = \"crates/helper\" }\n\n[[bin]]\nname = \"gen\"\npath = \"tools/gen.rs\"\n",
    ),
    // the lib root defines Shared and imports Deep; main.rs holds
    // neither — the crate:: tie-break's two habitats (step 8); it
    // declares `dual` (the E0761 double) but never `nest`
    (
        "src/lib.rs",
        "mod util;\nmod dual;\npub struct Shared;\npub use util::helper::Deep;\n",
    ),
    ("src/main.rs", "fn main() {}\n"),
    ("src/util.rs", "mod helper;\n"),
    ("src/util/helper.rs", "pub struct Deep;\n"),
    ("src/nest/mod.rs", "mod deep;\n"),
    ("src/nest/deep.rs", "\n"),
    ("src/dual.rs", "\n"),
    ("src/dual/mod.rs", "\n"),
    // a `use` folded over lines (step 5b): the detector's site is the
    // first line's `crate::util::`, the whole is read from the file
    (
        "src/folded.rs",
        "use crate::util::\n    helper::Deep;\nuse crate::nest::\n    deep::X;\n",
    ),
    // a local module sharing its name with a member crate and with a
    // builtin (step 8 review): the bare head reads the local module
    // first, the global `::` form never does
    ("src/shadow.rs", "mod helper_lib;\nmod test;\n"),
    ("src/shadow/helper_lib.rs", "\n"),
    ("src/shadow/test.rs", "\n"),
    ("tools/gen.rs", "mod gadget;\n"),
    ("tools/gadget.rs", "\n"),
    // the `<name>/main.rs` auto-discovery form (step 8): a root of
    // its own, so its modules mount in ITS directory and its
    // `crate::` walks anchor at it, not at the package's src roots
    ("tests/it/main.rs", "mod helper;\n"),
    ("tests/it/helper.rs", "\n"),
    ("src/bin/tool/main.rs", "mod part;\n"),
    ("src/bin/tool/part.rs", "\n"),
    // Rust R4: hyphenated member name (code says helper_lib), a
    // duplicate-name pair, registry dep declared in root Cargo.toml
    ("crates/helper/Cargo.toml", "[package]\nname='helper-lib'"),
    ("crates/helper/src/lib.rs", "\n"),
    ("crates/helper/src/sub.rs", "\n"),
    ("crates/dupx/Cargo.toml", "[package]\nname='dup-crate'"),
    ("crates/dupx/src/lib.rs", "\n"),
    ("crates/dupy/Cargo.toml", "[package]\nname='dup-crate'"),
    ("crates/dupy/src/lib.rs", "\n"),
    // Go R1-R3: nested module, a local replace target, a test-only
    // dir that refuses, and a duplicate module pair
    (
        "go.mod",
        "module x.io/root\nreplace x.io/away => ./vendored\n",
    ),
    ("cmd/main.go", "package main\n"),
    ("pkg/util/util.go", "package util\n"),
    ("pkg/util/util_test.go", "package util\n"),
    ("tstonly/only_test.go", "package tstonly\n"),
    ("vendored/v.go", "package vendored\n"),
    ("nested/go.mod", "module x.io/nested\n"),
    ("nested/lib/lib.go", "package lib\n"),
    ("dupg/a/go.mod", "module x.io/dup\n"),
    ("dupg/a/x/x.go", "package x\n"),
    ("dupg/b/go.mod", "module x.io/dup\n"),
    ("dupg/b/x/x.go", "package x\n"),
    // Hs R1-R2: one cabal with two stanzas (multi-root test-suite,
    // a comment INSIDE the build-depends block), a dual-root
    // ambiguity habitat, a two-cabal workspace tie, and a bare-ghc
    // loose pair with no cabal at all
    (
        "hs/pkg.cabal",
        "cabal-version: 3.0\nname: fixture-hs\n\nexecutable app\n    main-is:          Main.hs\n    hs-source-dirs:   app\n    build-depends:\n        base >=4.19 && <5,\n        -- a comment inside the block must not eat what follows\n        bytestring >=0.11,\n        aeson >=2.2,\n        corelib\n\ntest-suite spec\n    type:             exitcode-stdio-1.0\n    main-is:          Spec.hs\n    hs-source-dirs:   app, tst\n    build-depends:    base\n",
    ),
    ("hs/app/Main.hs", "module Main where\n"),
    ("hs/app/CE/Alpha.hs", "module CE.Alpha where\n"),
    ("hs/app/CE/Alpha/Deep.hs", "module CE.Alpha.Deep where\n"),
    ("hs/app/Dup.hs", "module Dup where\n"),
    ("hs/tst/Spec.hs", "module Main where\n"),
    ("hs/tst/Props.hs", "module Props where\n"),
    ("hs/tst/Dup.hs", "module Dup where\n"),
    // a second in-corpus package (step 5b): the executable above
    // depends on it, its library exposes Core.Api and hides Core.Hidden
    (
        "hslib/core.cabal",
        "name: corelib\nlibrary\n    hs-source-dirs: src\n    exposed-modules: Core.Api\n    other-modules: Core.Hidden\n",
    ),
    ("hslib/src/Core/Api.hs", "module Core.Api where\n"),
    ("hslib/src/Core/Hidden.hs", "module Core.Hidden where\n"),
    ("hs/wsdup/one.cabal", "library\n    hs-source-dirs: src\n"),
    ("hs/wsdup/two.cabal", "library\n    hs-source-dirs: src\n"),
    ("hs/wsdup/src/W.hs", "module W where\n"),
    ("hs/wsdup/src/Use.hs", "module Use where\nimport W\n"),
    ("loose.hs", "import Helper\n"),
    ("Helper.hs", "module Helper where\n"),
];

/// TS rows, the in-tree rungs — relative and twin (R1 / R2), tsconfig
/// paths (R3), workspace members and their exports (R4); every site
/// from src/app.ts. Tables split by dispatch shape, not to hide length.
fn ts_cases() -> Vec<Case> {
    let (ts, im) = (Lang::TypeScript, "import");
    let app = "src/app.ts";
    vec![
        (ts, im, app, "./util", ok("src/util.ts", 1)),
        (ts, im, app, "./widget", ok("src/widget.ts", 1)),
        (ts, im, app, "./util.ts", ok("src/util.ts", 1)),
        (ts, im, app, "../shared/x", ok("shared/x.tsx", 1)),
        (ts, im, app, "./legacy.js", ok("src/legacy.ts", 2)),
        (ts, im, app, "./built.js", no(Reason::OutOfScope)),
        (ts, im, app, "@app/util", ok("src/util.ts", 3)),
        (ts, im, app, "leaf", ok("lib/leaf.ts", 3)),
        (ts, im, app, "@dup/thing", no(Reason::AmbiguousPaths)),
        (ts, im, app, "pkga", ok("packages/pkga/src/index.ts", 4)),
        (ts, im, app, "pkga/sub", ok("packages/pkga/src/sub.ts", 4)),
        (
            ts,
            im,
            app,
            "pkga/lib/sub",
            ok("packages/pkga/src/sub.ts", 4),
        ),
        (
            ts,
            im,
            app,
            "pkga/lib/deep/x",
            ok("packages/pkga/src/nested/x.ts", 4),
        ),
        (ts, im, app, "pkga/lib/private", no(Reason::OutOfScope)),
        (ts, im, app, "dup-pkg", no(Reason::AmbiguousWorkspace)),
        (ts, im, app, "pkgb", no(Reason::AmbiguousExports)),
    ]
}

/// TS rows, the bare rung (R5) and the refusals: a dependency, a
/// leftover, Node's builtins (step 5b; CE.Resolve.Ts.isBuiltin since
/// v2.33 W2-text stage E: bare or `node:`-
/// prefixed, a subpath, a prefix-only name; a `node:` name Node has no
/// module for is nothing), a package vendored under the importing
/// file's own package (External from inside that package alone), an
/// unknown name, a missing relative file, and the degenerate specifier
/// refused by name before any rung reads `""` as a bare package (O60).
fn ts_bare_cases() -> Vec<Case> {
    let at = |spec: &'static str, want: Outcome| -> Case {
        (Lang::TypeScript, "import", "src/app.ts", spec, want)
    };
    vec![
        at("lodash", ext(5)),
        at("leftover", ext(5)),
        at("fs", ext(5)),
        at("node:fs", ext(5)),
        at("fs/promises", ext(5)),
        at("node:test", ext(5)),
        at("node:nope", no(Reason::OutOfScope)),
        (
            Lang::TypeScript,
            "import",
            "packages/pkga/src/index.ts",
            "vend",
            ext(5),
        ),
        at("vend", no(Reason::OutOfScope)),
        at("unknown-pkg", no(Reason::OutOfScope)),
        at("./missing", no(Reason::OutOfScope)),
        at("", no(Reason::Empty)),
    ]
}

/// Py rows — the dotted-relative, source-root, `__init__` degradation
/// and stdlib / dependency rungs, every site from pkg/consumer.py; the
/// `from __future__` site (step 8, O27) is a stdlib module the
/// public-names table omits, answered by name.
fn py_cases() -> Vec<Case> {
    let py = Lang::Python;
    let con = "pkg/consumer.py";
    vec![
        (py, "import", con, ".mod", ok("pkg/mod.py", 1)),
        (py, "import", con, ".", ok("pkg/__init__.py", 1)),
        (py, "import", con, ".sub", ok("pkg/sub/__init__.py", 1)),
        (py, "import", con, ".sub.leaf", ok("pkg/sub/leaf.py", 1)),
        (py, "import", con, "..other", ok("other.py", 1)),
        (py, "import", con, "...breaks", no(Reason::OutOfScope)),
        (py, "import", con, "top", ok("top.py", 2)),
        (py, "import", con, "pkg.mod", ok("pkg/mod.py", 2)),
        (py, "import", con, "tool", no(Reason::AmbiguousRoot)),
        (py, "import", con, "pkg.missing", ok("pkg/__init__.py", 3)),
        (
            py,
            "import",
            con,
            "pkg.sub.missing",
            ok("pkg/sub/__init__.py", 3),
        ),
        (py, "import", con, "os", ext(4)),
        (py, "import", con, "os.path", ext(4)),
        (py, "import_from", con, "__future__", ext(4)),
        (py, "import", con, "requests", ext(4)),
        (py, "import", con, "nosuch_pkg", no(Reason::OutOfScope)),
    ]
}

/// Go rows — package granularity: pkg() targets a DIRECTORY, and a
/// test-only dir, a double-declared module, and a dotless non-std
/// head all refuse.
fn go_cases() -> Vec<Case> {
    let (go, im, gm, nlib) = (Lang::Go, "import", "cmd/main.go", "nested/lib/lib.go");
    let pu = "pkg/util";
    vec![
        (go, im, gm, "", no(Reason::Empty)),
        (go, im, gm, "x.io/root/pkg/util", pkg(pu, 1)),
        (go, im, nlib, "x.io/root/pkg/util", pkg(pu, 1)),
        (go, im, gm, "x.io/nested/lib", pkg("nested/lib", 1)),
        (go, im, gm, "x.io/root/tstonly", no(Reason::OutOfScope)),
        (go, im, gm, "x.io/root/missing", no(Reason::OutOfScope)),
        (go, im, gm, "x.io/away", pkg("vendored", 2)),
        (go, im, gm, "x.io/dup/x", no(Reason::AmbiguousWorkspace)),
        (go, im, gm, "fmt", ext(3)),
        (go, im, gm, "notreal", no(Reason::OutOfScope)),
        (go, im, gm, "net/http", ext(3)),
        (go, im, gm, "github.com/spf13/pflag", ext(3)),
    ]
}

/// Rust rows, R1 — mod_decl builds the tree: root / mod.rs /
/// 2018-subdir / [[bin]] / Cargo auto-target anchors, the E0761 double;
/// #[path] is line-anchored — its battery sits below.
fn rust_mount_cases() -> Vec<Case> {
    // the three Rust tables bind their fixture files in three different
    // shapes on purpose — identical headers read as clones
    let (rs, md, us, tbin) = (Lang::Rust, "mod_decl", "use", "tools/gen.rs");
    let (rlib, rutil, nmod) = ("src/lib.rs", "src/util.rs", "src/nest/mod.rs");
    let (deep, uh) = ("src/nest/deep.rs", "src/util/helper.rs");
    vec![
        (rs, md, rlib, "util", ok("src/util.rs", 1)),
        (rs, md, rlib, "nest", ok(nmod, 1)),
        (rs, md, rutil, "helper", ok(uh, 1)),
        (rs, md, nmod, "deep", ok(deep, 1)),
        (rs, md, tbin, "gadget", ok("tools/gadget.rs", 1)),
        (rs, md, rlib, "dual", no(Reason::AmbiguousPaths)),
        (rs, md, rlib, "pathed", no(Reason::OutOfScope)),
        // Cargo auto-targets (step 8): tests/<name>/main.rs and
        // src/bin/<name>/main.rs are roots, so their mod_decls mount
        // and a crate:: walk from inside them comes home
        (
            rs,
            md,
            "tests/it/main.rs",
            "helper",
            ok("tests/it/helper.rs", 1),
        ),
        (
            rs,
            md,
            "src/bin/tool/main.rs",
            "part",
            ok("src/bin/tool/part.rs", 1),
        ),
        (
            rs,
            us,
            "tests/it/helper.rs",
            "crate::helper",
            ok("tests/it/helper.rs", 2),
        ),
    ]
}

/// Rust rows, R2 — use walks inside the crate: crate:: paths from the
/// covering roots. A walk ending AT the root sees lib+main and refuses;
/// a folded fragment's pre-{ prefix is complete; a hand-folded mid-path
/// fragment is read whole from the `use` on its line (step 5b), and one
/// no `use` on its line opens is refused.
fn rust_walk_cases() -> Vec<Case> {
    let (rs, us) = (Lang::Rust, "use");
    let (rlib, rutil, nmod, deep) = (
        "src/lib.rs",
        "src/util.rs",
        "src/nest/mod.rs",
        "src/nest/deep.rs",
    );
    let (tbin, uh) = ("tools/gen.rs", "src/util/helper.rs");
    vec![
        (rs, us, rutil, "crate::nest::deep", ok(deep, 2)),
        (rs, us, deep, "crate::util::helper as h", ok(uh, 2)),
        (rs, us, rutil, "crate::nest::{", ok(nmod, 2)),
        (rs, us, rutil, "crate::missing", no(Reason::AmbiguousRoot)),
        // the tie-break (step 8, ruling ④): the root that defines
        // Shared answers; the root whose uniform-path `pub use`
        // imports Deep answers AND binds one hop to the definition
        (rs, us, rutil, "crate::Shared", ok(rlib, 2)),
        (rs, us, deep, "crate::Deep", via(uh, 2)),
        (rs, us, tbin, "crate::gadget", ok("tools/gadget.rs", 2)),
        (rs, us, rutil, "crate::dual::x", no(Reason::AmbiguousPaths)),
        (rs, us, rutil, "foo::", no(Reason::OutOfScope)),
        (rs, "use@1", "src/folded.rs", "crate::util::", ok(uh, 2)),
        (rs, "use@3", "src/folded.rs", "crate::nest::", ok(deep, 2)),
        (
            rs,
            "use@3",
            "src/folded.rs",
            "crate::util::",
            no(Reason::OutOfScope),
        ),
    ]
}

/// Rust rows, R3 — the bare, self and super heads (step 8, ruling ④):
/// a module DECLARED in the site's own namespace is read before any
/// crate name — a declaration mounts and descends, the E0761 double
/// still refuses, and a head nothing declares there is a crate name
/// (deep.rs declares no `mod util`; lib.rs declares no `mod nest`
/// though nest/mod.rs sits on disk — a file is not a module until a
/// declaration mounts it); the island red condition — an intra-file
/// self:: reference must come home to its own file, never dangle.
fn rust_head_cases() -> Vec<Case> {
    let at = |from: &'static str, spec: &'static str, want: Outcome| -> Case {
        (Lang::Rust, "use", from, spec, want)
    };
    let (rlib, rutil) = ("src/lib.rs", "src/util.rs");
    let (nmod, deep, uh) = ("src/nest/mod.rs", "src/nest/deep.rs", "src/util/helper.rs");
    vec![
        at(rlib, "util::helper::x", ok(uh, 3)),
        at(rutil, "helper::thing", ok(uh, 3)),
        at(nmod, "deep::x", ok(deep, 3)),
        at(rlib, "dual::x", no(Reason::AmbiguousPaths)),
        at(deep, "util::x", no(Reason::OutOfScope)),
        at(rlib, "nest::deep::x", no(Reason::OutOfScope)),
        at(deep, "self::helpers", ok(deep, 3)),
        at(nmod, "self::deep", ok(deep, 3)),
        at(deep, "super::x", ok(nmod, 3)),
        at(rutil, "super::super::x", no(Reason::OutOfScope)),
        at(deep, "super::super::util", ok(rutil, 3)),
    ]
}

/// Rust rows, R4 — use walks that leave the crate: normalized member
/// name, member-tree descent (the audit records the definition file,
/// not the crate façade), duplicate pair, registry dep, nothing,
/// builtin (order breaks table-tail cloning); then the shadow file
/// (step 8 review): a declared local module wins a bare head over the
/// member crate AND over a builtin name (`test` is no extern-prelude
/// crate), the global `::` form reads neither.
fn rust_member_cases() -> Vec<Case> {
    let rs = Lang::Rust;
    let (us, rlib, rutil, hl, sh) = (
        "use",
        "src/lib.rs",
        "src/util.rs",
        "crates/helper/src/lib.rs",
        "src/shadow.rs",
    );
    vec![
        (
            rs,
            us,
            sh,
            "helper_lib::x",
            ok("src/shadow/helper_lib.rs", 3),
        ),
        (rs, us, sh, "::helper_lib::x", ok(hl, 4)),
        (rs, us, sh, "test::Helper", ok("src/shadow/test.rs", 3)),
        (rs, us, sh, "::std::fs", ext(4)),
        (rs, us, rutil, "helper_lib::x", ok(hl, 4)),
        (
            rs,
            us,
            rutil,
            "helper_lib::sub::y",
            ok("crates/helper/src/sub.rs", 4),
        ),
        (rs, us, rlib, "dup_crate", no(Reason::AmbiguousWorkspace)),
        (rs, us, rutil, "serde::Serialize", ext(4)),
        (rs, us, rutil, "nowhere::x", no(Reason::OutOfScope)),
        (rs, us, rutil, "std::fs", ext(4)),
    ]
}

/// Haskell rows — cabal-anchored roots. Pinned stances: a file two
/// stanzas both hold walks the UNION of their roots (which component
/// compiles it is unknowable, so cross-component disagreement
/// refuses); the external table answers only through the owner's
/// build-depends (Data.Map refuses while containers is undeclared),
/// and with no cabal at all the whole global db is default-visible
/// (bare-ghc semantics); a declared dep OUTSIDE the global db
/// (aeson, store-installed) refuses — module→package needs evidence,
/// never a guess. Since step 5b a module another in-corpus package
/// exposes answers at R2 (hs_package_cases), so the external table
/// sits at R3.
fn hs_cases() -> Vec<Case> {
    let (hs, im) = (Lang::Haskell, "import");
    let (hm, hsp, lo) = ("hs/app/Main.hs", "hs/tst/Spec.hs", "loose.hs");
    vec![
        (hs, im, hm, "CE.Alpha", ok("hs/app/CE/Alpha.hs", 1)),
        (
            hs,
            im,
            hm,
            "CE.Alpha.Deep",
            ok("hs/app/CE/Alpha/Deep.hs", 1),
        ),
        (hs, im, hsp, "Props", ok("hs/tst/Props.hs", 1)),
        (hs, im, hsp, "CE.Alpha", ok("hs/app/CE/Alpha.hs", 1)),
        (hs, im, hsp, "Dup", no(Reason::AmbiguousRoot)),
        (hs, im, hm, "Dup", no(Reason::AmbiguousRoot)),
        (hs, im, lo, "Helper", ok("Helper.hs", 1)),
        (hs, im, hm, "Data.List", ext(3)),
        (hs, im, hm, "Data.ByteString.Lazy", ext(3)),
        (hs, im, hm, "Data.Map", no(Reason::OutOfScope)),
        (hs, im, lo, "Data.Map", ext(3)),
        (hs, im, hm, "Data.Aeson", no(Reason::OutOfScope)),
        (hs, im, hm, "CE.Missing", no(Reason::OutOfScope)),
        (hs, im, hm, "lowercase.name", no(Reason::OutOfScope)),
        (
            hs,
            im,
            "hs/wsdup/src/Use.hs",
            "W",
            no(Reason::AmbiguousWorkspace),
        ),
    ]
}

/// Haskell rows, R2 (step 5b): a module another in-corpus package
/// exposes, through the owner's build-depends or named; a hidden module
/// and an undeclared named package are out of scope, a PackageImports
/// spec naming the owner's own package walks R1, a loose file (no
/// owner) reaches only what it names.
fn hs_package_cases() -> Vec<Case> {
    let (hs, im, hm, lo, api) = (
        Lang::Haskell,
        "import",
        "hs/app/Main.hs",
        "loose.hs",
        "hslib/src/Core/Api.hs",
    );
    vec![
        (hs, im, hm, "Core.Api", ok(api, 2)),
        (hs, im, hm, "Core.Hidden", no(Reason::OutOfScope)),
        (hs, im, hm, "\"corelib\" Core.Api", ok(api, 2)),
        (
            hs,
            im,
            hm,
            "\"fixture-hs\" CE.Alpha",
            ok("hs/app/CE/Alpha.hs", 1),
        ),
        (hs, im, hm, "\"base\" Data.List", ext(3)),
        (
            hs,
            im,
            hm,
            "\"containers\" Data.Map",
            no(Reason::OutOfScope),
        ),
        (hs, im, lo, "Core.Api", no(Reason::OutOfScope)),
        (hs, im, lo, "\"corelib\" Core.Api", ok(api, 2)),
    ]
}

#[test]
fn rungs_resolve_and_refuse() {
    let fx = fixture("ladder-rungs", TREE);
    let all = ts_cases()
        .into_iter()
        .chain(ts_bare_cases())
        .chain(py_cases())
        .chain(rust_mount_cases())
        .chain(rust_walk_cases())
        .chain(rust_head_cases())
        .chain(rust_member_cases())
        .chain(go_cases())
        .chain(hs_cases())
        .chain(hs_package_cases());
    run_cases(&fx, all.collect());
}
