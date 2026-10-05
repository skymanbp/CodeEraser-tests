//! TypeScript / TSX (plan v2.33 W2-text stage E): seeded trees with TS
//! files (`.ts`, `.tsx`, `.d.ts`, `.mts`, `.cts`, index files) under
//! source and package directories, tsconfig files whose `extends` chains
//! run through arrays, cycles, bare packages, unreadable and out-of-tree
//! targets, package.json files with names (some shared), exports maps
//! (exact keys, patterns, nested conditions, null) and dependency maps —
//! most walked, some only on disk, as a gitignored one is —
//! node_modules directories (and a file of a package's name), compiled
//! JavaScript twins beside some TS files — and five legs, one per rung:
//! relative specifiers (R1), their JavaScript spelling (R2), tsconfig
//! paths and baseUrl (R3),
//! workspace members (R4), bare names (R5: builtins, `node:` names,
//! declared and vendored packages). Each leg aims a quarter of its sites
//! at the other rungs.

use super::common::{Tree, World, put};
use super::rng::Rng;
use super::tables::t;
use super::ts_gen::{package, spec, tsconfig};
use crate::graph::roots::join_dir;
use crate::scan::lang::Lang;
use std::path::Path;

/// Configs at the table's directories: written with chance `pct`, most
/// of them walked (in `configs`), the rest on disk only.
fn configs(
    (rng, root): (&mut Rng, &Path),
    world: &mut World,
    (dirs, names): (&'static str, &'static str),
    pct: usize,
    text: fn(&mut Rng) -> String,
) {
    for dir in t(dirs).split('|') {
        for _ in 0..usize::from(rng.chance(pct)) + usize::from(rng.chance(pct / 3)) {
            let rel = join_dir(dir, rng.pick(names));
            if rng.chance(4) {
                let p = root.join(&rel);
                std::fs::create_dir_all(p.parent().expect("a parent")).expect("dirs");
                std::fs::write(p, b"{\"name\": \"\xff\xfe\"}").expect("write");
            } else {
                put(root, &rel, &text(rng));
            }
            if rng.chance(85) && !world.configs.contains(&rel) {
                world.configs.push(rel);
            }
        }
    }
}

/// node_modules entries under some directories: a package directory, a
/// scoped one, now and then a plain file of a package's name.
fn vendored(rng: &mut Rng, root: &Path) {
    for dir in t("ts NM_DIRS").split('|') {
        if !rng.chance(35) {
            continue;
        }
        for _ in 0..1 + rng.below(3) {
            let at = root
                .join(join_dir(dir, "node_modules"))
                .join(rng.pick(t("ts NM_NAMES")));
            if rng.chance(15) {
                std::fs::create_dir_all(at.parent().expect("a parent")).expect("dirs");
                // a name already made a directory in this tree stays one
                let _ = std::fs::write(&at, "");
            } else {
                let _ = std::fs::create_dir_all(&at);
            }
        }
    }
}

/// The TS extensions a site may come from.
const TS_EXTS: [&str; 4] = [".ts", ".tsx", ".mts", ".cts"];

fn is_ts(f: &str) -> bool {
    TS_EXTS.iter().any(|e| f.ends_with(e))
}

/// Compiled JavaScript twins of some walked TS files, on disk only.
fn twins(rng: &mut Rng, root: &Path, world: &World) {
    for f in &world.files {
        if let Some(stem) = TS_EXTS.iter().find_map(|e| f.strip_suffix(e))
            && rng.chance(20)
        {
            put(root, &format!("{stem}{}", rng.pick(".js|.mjs|.cjs")), "");
        }
    }
}

/// The language a TS file's sites carry: TSX for `.tsx`.
fn lang_of(from: &str) -> Lang {
    if from.ends_with(".tsx") {
        Lang::Tsx
    } else {
        Lang::TypeScript
    }
}

/// The third or fourth rung's layout planted whole (rung 1: a root
/// tsconfig whose paths and baseUrl name the planted files, its base
/// beside it; rung 2: two members whose exports name them), the planted
/// files walked; the random configs then rarer.
fn plant(rng: &mut Rng, root: &Path, world: &mut World, rung: usize) {
    let (files, docs) = if rung == 1 {
        (
            "ts PLANT_PATH_FILES",
            [
                ("tsconfig.json", "ts PLANT_PATHS"),
                ("shared/base.json", "ts PLANT_BASES"),
            ],
        )
    } else {
        (
            "ts PLANT_PKG_FILES",
            [
                ("packages/p1/package.json", ""),
                ("packages/p2/package.json", ""),
            ],
        )
    };
    for _ in 0..3 + rng.below(5) {
        world.files.insert(rng.pick(t(files)).to_string());
    }
    for (i, (rel, table)) in docs.into_iter().enumerate() {
        let text = match (table, i) {
            ("", 0) => format!(
                "{{\"name\": \"@scope/p1\", \"exports\": {}}}",
                rng.pick(t("ts PLANT_EXPORTS"))
            ),
            ("", _) => {
                let name = rng.pick("p2|p2|@scope/p1");
                let exports = rng.pick(t("ts PLANT_EXPORTS"));
                format!("{{\"name\": \"{name}\", \"exports\": {exports}}}")
            }
            _ => rng.pick(t(table)).to_string(),
        };
        world.configs.push(rel.to_string());
        put(root, rel, &text);
    }
}

/// A tree: its TS files, configs, node_modules and twins, thirty sites
/// aimed at `rung` (0 R1, 4 R2, 1 R3, 2 R4, 3 R5); the paths and
/// workspace rungs plant their layout whole three times in five.
fn tree(rng: &mut Rng, root: &Path, rung: usize) -> Tree {
    let plants = matches!(rung, 1 | 2);
    let mut world = World::seeded(rng, (8, 30), t("ts DIRS"), t("ts BASES"));
    let planted = plants && rng.chance(60);
    if planted {
        plant(rng, root, &mut world, rung);
    }
    let rare = if planted { 3 } else { 1 };
    configs(
        (rng, root),
        &mut world,
        ("ts PKG_DIRS", "package.json"),
        55 / rare,
        package,
    );
    configs(
        (rng, root),
        &mut world,
        ("ts CFG_DIRS", t("ts CFG_NAMES")),
        45 / rare,
        tsconfig,
    );
    vendored(rng, root);
    twins(rng, root, &world);
    let targets: Vec<&String> = world.files.iter().collect();
    let owned = world.sites(rng, is_ts, (t("ts DIRS"), "zz.ts"), |rng, from| {
        let s = spec(rng, rung, &targets, &from);
        (lang_of(&from), "import", from, s)
    });
    (world, owned)
}

/// The five legs, one per rung (the `ts LEGS` table).
#[test]
#[ignore = "instrument: needs a core (CE_CORE_BIN); run with --ignored --nocapture"]
fn ts_random_trees_agree() {
    super::common::legs("ts", tree);
}
