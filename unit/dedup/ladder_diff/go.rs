//! Go: seeded trees with nested go.mod files (empty and slashed module
//! paths, a file with no module line), replace directives to directories
//! (`./`, `../`, `..` inside) and to module paths, package directories
//! with and without non-test `.go` files, and import paths built from
//! module prefixes, stdlib names, dotted heads and odd slashes.

use super::common::{Tree, World, leg};
use super::rng::Rng;
use super::tables::t;
use crate::scan::lang::Lang;
use std::path::Path;

/// One replace directive's text: versions on either side or none.
fn replace(rng: &mut Rng) -> String {
    let (old, new) = (rng.pick(t("go OLDS")), rng.pick(t("go NEWS")));
    if rng.chance(30) {
        format!("{old} v1.2.3 => {new} v1.0.0")
    } else {
        format!("{old} => {new}")
    }
}

fn go_mod(rng: &mut Rng) -> String {
    let mut text = String::new();
    if rng.chance(90) {
        text.push_str(&format!(
            "module {}\n\ngo 1.22\n",
            rng.pick(t("go MODULES"))
        ));
    }
    let lines: Vec<String> = (0..rng.below(4)).map(|_| replace(rng)).collect();
    let block = rng.chance(30);
    if block && !lines.is_empty() {
        text.push_str(&format!("replace (\n\t{}\n)\n", lines.join("\n\t")));
    } else {
        lines
            .iter()
            .for_each(|l| text.push_str(&format!("replace {l}\n")));
    }
    text
}

fn spec(rng: &mut Rng) -> String {
    let mut s = rng.pick(t("go HEADS")).to_string();
    for _ in 0..rng.below(3) {
        s.push('/');
        s.push_str(rng.pick(t("go RESTS")));
    }
    if rng.chance(5) {
        s.push('/');
    }
    s
}

fn tree(rng: &mut Rng, root: &Path) -> Tree {
    let mut world = World::seeded(rng, (10, 30), t("go DIRS"), t("go BASES"));
    world.manifests((rng, root), (t("go MOD_DIRS"), "go.mod"), 55, go_mod);
    let keep = |f: &str| f.ends_with(".go");
    let owned = world.sites(rng, keep, (t("go DIRS"), "zz.go"), |rng, from| {
        (Lang::Go, "import", from, spec(rng))
    });
    (world, owned)
}

#[test]
#[ignore = "instrument: needs a core (CE_CORE_BIN); run with --ignored --nocapture"]
fn go_random_trees_agree() {
    leg("go", 3, false, tree);
}
