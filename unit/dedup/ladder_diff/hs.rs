//! Haskell (plan v2.33 W2-text stage D): seeded trees with Haskell files
//! under source directories, .cabal files drawn as text at some of the
//! package directories (two in one directory now and then, one listed and
//! never written) — `name:` before or after a header, the four component
//! headers in any case and named or bare, `common` stanzas pulled in by
//! `import:`, `hs-source-dirs` / `exposed-modules` / `other-modules` /
//! `build-depends` / `main-is` over continuation lines with comments
//! inside, fields at column 0, tabs and CRLF; the roots, modules and
//! mains a cabal names are mostly the ones its tree spells — and three
//! legs, one per rung: modules a source root holds (R1), modules a
//! depended package exposes, by name under PackageImports or through
//! build-depends (R2), and the global database's modules, gated or named
//! (R3); each leg also draws malformed specifiers (a lowercase segment,
//! an empty one, a quote never closed). Every tree's cabal mains and
//! cabal privacy are compared too (beside.rs).

use super::common::{Tree, World, put};
use super::rng::Rng;
use super::tables::t;
use crate::graph::roots::join_dir;
use crate::scan::lang::Lang;
use std::path::Path;

/// What a tree spells below one directory: each Haskell file split at
/// each of its directory boundaries into a source root (`.` for the
/// directory itself) and the module below it, where that is a module
/// name; the main-is path is the module's file below the root.
#[derive(Default)]
struct Spelled {
    roots: Vec<String>,
    modules: Vec<String>,
    mains: Vec<String>,
}

fn shaped(module: &str) -> bool {
    module
        .split('.')
        .all(|s| s.starts_with(|c: char| c.is_ascii_uppercase()))
}

fn spelled(world: &World, dir: &str) -> Spelled {
    let mut out = Spelled::default();
    for file in world.files.iter().filter(|f| f.ends_with(".hs")) {
        let below = if dir.is_empty() {
            file.as_str()
        } else {
            let Some(b) = file.strip_prefix(&format!("{dir}/")) else {
                continue;
            };
            b
        };
        let cuts = std::iter::once(0).chain(below.match_indices('/').map(|(i, _)| i + 1));
        for cut in cuts {
            let module = below[cut..].trim_end_matches(".hs").replace('/', ".");
            if shaped(&module) {
                let root = below[..cut].trim_end_matches('/');
                out.roots
                    .push(if root.is_empty() { "." } else { root }.to_string());
                out.modules.push(module);
                out.mains.push(below[cut..].to_string());
            }
        }
    }
    out
}

/// A value: one of the tree's own (`own`), else a table word.
fn value(rng: &mut Rng, own: &[String], table: &'static str) -> String {
    if !own.is_empty() && rng.chance(60) {
        own[rng.below(own.len())].clone()
    } else {
        rng.pick(table).to_string()
    }
}

/// One field line, its continuation lines after it.
fn field(rng: &mut Rng, name: &str, mut draw: impl FnMut(&mut Rng) -> String) -> Vec<String> {
    let indent = rng.pick(t("hs INDENTS"));
    let mut lines = vec![format!("{indent}{name}: {}", draw(rng))];
    for _ in 0..rng.below(3) {
        if rng.chance(15) {
            lines.push(format!("{}-- a comment", rng.pick(t("hs INDENTS"))));
        }
        let sep = if rng.chance(50) { ", " } else { " " };
        lines.push(format!("{}{}{sep}", rng.pick("    |\t|      "), draw(rng)));
    }
    lines
}

/// One stanza's fields.
fn fields(rng: &mut Rng, own: &Spelled, lines: &mut Vec<String>) {
    for _ in 0..rng.below(6) {
        let picked = match rng.below(10) {
            0..3 => field(rng, "hs-source-dirs", |r| {
                value(r, &own.roots, t("hs ROOTS"))
            }),
            3..5 => field(rng, "exposed-modules", |r| {
                value(r, &own.modules, t("hs MODULES"))
            }),
            5 => field(rng, "other-modules", |r| {
                value(r, &own.modules, t("hs MODULES"))
            }),
            6 => field(rng, "build-depends", |r| {
                r.pick(t("hs DEPENDS")).to_string()
            }),
            7 => field(rng, "main-is", |r| value(r, &own.mains, t("hs MAINS"))),
            8 => field(rng, "import", |r| r.pick(t("hs COMMONS")).to_string()),
            _ => {
                let name = rng.pick(t("hs FIELDS"));
                field(rng, name, |r| value(r, &own.modules, t("hs MODULES")))
            }
        };
        lines.extend(picked);
    }
}

/// One .cabal file's text, held in `dir`.
fn cabal(rng: &mut Rng, own: &Spelled) -> String {
    let mut lines: Vec<String> = Vec::new();
    if rng.chance(70) {
        let name = rng.pick("name|Name|NAME");
        lines.push(format!("{name}: {}", rng.pick(t("hs PACKAGES"))));
    }
    lines.push("cabal-version: 2.4".into());
    for _ in 0..1 + rng.below(4) {
        let header = if rng.chance(35) {
            "library"
        } else {
            rng.pick(t("hs HEADERS"))
        };
        lines.push(header.to_string());
        fields(rng, own, &mut lines);
    }
    if rng.chance(20) {
        lines.push(format!("name: {}", rng.pick(t("hs PACKAGES"))));
    }
    let mut text = lines.join(if rng.chance(30) { "\r\n" } else { "\n" });
    if rng.chance(30) {
        text.push('\n');
    }
    text
}

/// .cabal files at some of the package directories — a second one in
/// the same directory now and then, one listed and never written.
fn cabals(rng: &mut Rng, root: &Path, world: &mut World) {
    for dir in t("hs CABAL_DIRS").split('|') {
        let own = spelled(world, dir);
        for _ in 0..usize::from(rng.chance(45)) + usize::from(rng.chance(8)) {
            let name = format!("{}.cabal", rng.pick(t("hs CABAL_NAMES")));
            let rel = join_dir(dir, &name);
            if !world.configs.contains(&rel) {
                put(root, &rel, &cabal(rng, &own));
                world.configs.push(rel);
            }
        }
    }
    if rng.chance(5) {
        world.configs.push("ghost/g.cabal".into());
    }
}

/// A specifier for the rung `rung` names: 0 and 1 a module the tree
/// spells (1 named under a package more often), 2 a global-database
/// module (named, sometimes, under its package); one in ten malformed.
fn spec(rng: &mut Rng, own: &Spelled, rung: usize) -> String {
    if rng.chance(10) {
        return rng.pick(t("hs MALFORMED")).to_string();
    }
    let module = match rung {
        0 | 1 => value(rng, &own.modules, t("hs MODULES")),
        _ => rng.pick(t("hs BOOT")).to_string(),
    };
    let (named, packages) = match rung {
        0 => (rng.chance(15), t("hs PACKAGES")),
        1 => (rng.chance(55), t("hs PACKAGES")),
        _ => (rng.chance(40), t("hs BOOT_PACKAGES")),
    };
    if !named {
        return module;
    }
    format!(
        "\"{}\"{}{module}",
        rng.pick(packages),
        rng.pick(" |  | |\t")
    )
}

/// The R2 layout planted whole: library files under `other/lib` (and
/// `x/lib`), a package `other` whose library exposes what `other/lib`
/// spells, a second one (`x/x.cabal`, named `other` or `twin`) exposing
/// the same now and then, and the package at `pkg` depending on one or
/// both — no other cabal. The modules the library exposes are the ones
/// the sites then name.
fn plant(rng: &mut Rng, root: &Path, world: &mut World) -> Vec<String> {
    for _ in 0..2 + rng.below(4) {
        world.files.insert(rng.pick(t("hs LIBRARY")).to_string());
    }
    let lib = spelled(world, "other/lib");
    let exposed = lib.modules.join(", ");
    let library = format!("library\n  hs-source-dirs: lib\n  exposed-modules: {exposed}\n");
    let mut depends = vec!["base", "other"];
    let mut write = |rel: &str, text: String| {
        put(root, rel, &text);
        world.configs.push(rel.to_string());
    };
    write("other/other.cabal", format!("name: other\n{library}"));
    if rng.chance(15) {
        let twin = rng.pick("other|twin");
        depends.push(twin);
        write("x/x.cabal", format!("name: {twin}\n{library}"));
    }
    let depends = depends.join(", ");
    write(
        "pkg/pkg.cabal",
        format!("name: pkg\nlibrary\n  hs-source-dirs: src\n  build-depends: {depends}\n"),
    );
    lib.modules
}

/// A tree: its Haskell files, its cabals, thirty sites aimed at `rung` —
/// in a planted tree, from the depending package's files.
fn tree(rng: &mut Rng, root: &Path, rung: usize) -> Tree {
    let mut world = World::seeded(rng, (8, 25), t("hs DIRS"), t("hs BASES"));
    let planted = rung == 1 && rng.chance(60);
    let mut own = Spelled::default();
    if planted {
        own.modules = plant(rng, root, &mut world);
    } else {
        cabals(rng, root, &mut world);
        own = spelled(&world, "");
        for dir in t("hs CABAL_DIRS").split('|').filter(|d| !d.is_empty()) {
            own.modules.extend(spelled(&world, dir).modules);
        }
    }
    let keep: fn(&str) -> bool = if planted {
        |f| f.starts_with("pkg/") && f.ends_with(".hs")
    } else {
        |f| f.ends_with(".hs")
    };
    let owned = world.sites(rng, keep, (t("hs DIRS"), "Zz.hs"), |rng, from| {
        (Lang::Haskell, "import", from, spec(rng, &own, rung))
    });
    (world, owned)
}

/// The three legs, one per rung (the `hs LEGS` table).
#[test]
#[ignore = "instrument: needs a core (CE_CORE_BIN); run with --ignored --nocapture"]
fn hs_random_trees_agree() {
    super::common::legs("hs", tree);
}
