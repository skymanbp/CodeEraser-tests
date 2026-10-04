//! C / C++: seeded trees with compile databases at the three probe
//! places (JSON at a directory and under its `build/`, flags files),
//! GNU and MSVC drivers, every include-path class (`-iquote`, `-I`,
//! `-isystem`, frameworks, `-I-`, `/I`, `/external:I`), forced includes
//! (relative, absolute in and outside the tree), entry directories in
//! and outside the tree, declared roots, include lists that build a
//! closure (and an MSVC include stack), and include specifiers in both
//! forms with `..`, `./`, empty and framework spellings. The forced-
//! include arcs are compared per tree too.

use super::common::{Tree, World, leg, put};
use super::rng::Rng;
use super::tables::t;
use crate::graph::compdb::root_text;
use crate::graph::compdb_find::{is_c, is_unit};
use crate::graph::roots::join_dir;
use crate::scan::lang::Lang;
use serde_json::{Value, json};
use std::path::Path;

fn spec(rng: &mut Rng) -> String {
    let name = rng.mostly(95, t("c NAMES"), 14);
    if rng.chance(35) {
        format!("<{name}>")
    } else {
        name.to_string()
    }
}

/// A directory operand: in-tree relative, absolute in the tree, or out.
fn place(rng: &mut Rng, abs: &str) -> String {
    let p = rng.pick(t("c PLACES"));
    if rng.chance(25) {
        format!("{abs}/{p}")
    } else {
        p.to_string()
    }
}

fn forced(rng: &mut Rng, abs: &str) -> String {
    match rng.below(10) {
        0 => format!("{abs}/{}", rng.pick(t("c FORCED"))),
        1 => "/nowhere/x.h".to_string(),
        _ => rng.pick(t("c FORCED")).to_string(),
    }
}

/// Up to five options, each one class of the drivers' grammar.
fn flags(rng: &mut Rng, abs: &str, msvc: bool) -> Vec<String> {
    let mut out = Vec::new();
    for _ in 0..rng.below(6) {
        let d = place(rng, abs);
        let class = rng.below(if msvc { 13 } else { 10 });
        out.extend(option(rng, abs, class, d));
    }
    out
}

/// One option of class `class` (GNU 0-9, MSVC 10-12) on directory `d`.
fn option(rng: &mut Rng, abs: &str, class: usize, d: String) -> Vec<String> {
    match class {
        0 => vec![format!("-I{d}")],
        1 => vec!["-I".into(), d],
        2 => vec!["-iquote".into(), d],
        3 => vec!["-isystem".into(), d],
        4 => vec![format!("-F{d}")],
        5 => vec!["-iframework".into(), d],
        6 => vec!["-I-".into()],
        7 => vec!["-include".into(), forced(rng, abs)],
        8 => vec!["-imacros".into(), forced(rng, abs)],
        9 => vec!["-idirafter".into(), d],
        10 => vec![format!("/I{d}")],
        11 => vec![format!("/FI{}", forced(rng, abs))],
        _ => vec![format!("/external:I{d}")],
    }
}

fn entry(rng: &mut Rng, abs: &str, units: &[&String]) -> Value {
    let unit = if units.is_empty() || rng.chance(8) {
        "ghost.c".to_string()
    } else {
        units[rng.below(units.len())].clone()
    };
    let dir = match rng.below(6) {
        0 => "/elsewhere/out".to_string(),
        1 => format!("{abs}/src"),
        2 => format!("{abs}/a"),
        _ => abs.to_string(),
    };
    let driver = rng.pick(t("c DRIVERS"));
    let mut argv = vec![driver.to_string()];
    argv.extend(flags(rng, abs, driver.contains("cl")));
    argv.extend(["-c".to_string(), format!("{abs}/{unit}")]);
    let file = format!("{abs}/{unit}");
    if rng.chance(30) {
        json!({ "directory": dir, "file": file, "command": argv.join(" ") })
    } else {
        json!({ "directory": dir, "file": file, "arguments": argv })
    }
}

fn databases(rng: &mut Rng, root: &Path, world: &World) {
    let abs = root_text(root);
    let units: Vec<&String> = world.files.iter().filter(|f| is_unit(f)).collect();
    for dir in ["", "src", "a", "x"] {
        if rng.chance(55) {
            let rows: Vec<Value> = (0..1 + rng.below(4))
                .map(|_| entry(rng, &abs, &units))
                .collect();
            let name = if rng.chance(50) {
                "compile_commands.json"
            } else {
                "build/compile_commands.json"
            };
            put(root, &join_dir(dir, name), &Value::Array(rows).to_string());
        }
        if rng.chance(30) {
            put(
                root,
                &join_dir(dir, "compile_flags.txt"),
                &flags(rng, &abs, false).join("\n"),
            );
        }
    }
}

fn tree(rng: &mut Rng, root: &Path) -> Tree {
    let mut world = World::seeded(rng, (8, 28), t("c DIRS"), t("c BASES"));
    let cs: Vec<String> = world.files.iter().filter(|f| is_c(f)).cloned().collect();
    for f in cs {
        if rng.chance(70) {
            let list = (0..rng.below(5)).map(|_| spec(rng)).collect();
            world.includes.insert(f, list);
        }
    }
    world.declare(rng, "c", "include|lib|a|inc|src");
    databases(rng, root, &world);
    let owned = world.sites(rng, is_c, (t("c DIRS"), "zz.c"), |rng, from| {
        let lang = if from.ends_with(".c") {
            Lang::C
        } else {
            Lang::Cpp
        };
        (lang, "include", from, spec(rng))
    });
    (world, owned)
}

#[test]
#[ignore = "instrument: needs a core (CE_CORE_BIN); run with --ignored --nocapture"]
fn c_random_trees_agree() {
    leg("c", 4, true, tree);
}
