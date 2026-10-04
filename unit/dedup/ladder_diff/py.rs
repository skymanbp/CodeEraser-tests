//! Python: seeded trees with packages, modules, nested roots, a
//! pyproject that declares (or mangles) its package directories and
//! dependencies, and specifiers with leading dots, empty segments,
//! slashes and `..` inside a segment, stdlib and dependency names.

use super::common::{Tree, World, leg, put};
use super::rng::Rng;
use super::tables::t;
use crate::scan::lang::Lang;
use std::path::Path;

fn pyproject(rng: &mut Rng) -> String {
    if rng.chance(8) {
        return "[project\nnot toml".to_string();
    }
    let mut text = String::from(
        "[project]\ndependencies = [\"requests>=2\", \"dep ; python_version<'3'\", \"A_b\"]\n",
    );
    text.push_str("[tool.setuptools.package-dir]\n");
    for k in 0..rng.below(3) {
        text.push_str(&format!("k{k} = \"{}\"\n", rng.pick(t("py ROOTS"))));
    }
    if rng.chance(50) {
        text.push_str("[tool.poetry]\npackages = [");
        for _ in 0..rng.below(3) {
            text.push_str(&format!(
                "{{ include = \"x\", from = \"{}\" }}, ",
                rng.pick(t("py ROOTS"))
            ));
        }
        text.push_str("]\n");
    }
    text
}

fn spec(rng: &mut Rng) -> String {
    let dots = [0, 0, 0, 0, 0, 0, 1, 1, 2, 3][rng.below(10)];
    let n = if dots == 0 {
        1 + rng.below(4)
    } else {
        rng.below(4)
    };
    let segs: Vec<&str> = (0..n).map(|_| rng.mostly(85, t("py SEGS"), 13)).collect();
    format!("{}{}", ".".repeat(dots), segs.join("."))
}

fn tree(rng: &mut Rng, root: &Path) -> Tree {
    let world = World::seeded(rng, (3, 18), t("py DIRS"), t("py BASES"));
    if rng.chance(70) {
        put(root, "pyproject.toml", &pyproject(rng));
    }
    let keep = |f: &str| f.ends_with(".py");
    let owned = world.sites(rng, keep, (t("py DIRS"), "z.py"), |rng, from| {
        let kind = if rng.chance(50) {
            "import"
        } else {
            "import_from"
        };
        (Lang::Python, kind, from, spec(rng))
    });
    (world, owned)
}

#[test]
#[ignore = "instrument: needs a core (CE_CORE_BIN); run with --ignored --nocapture"]
fn python_random_trees_agree() {
    leg("python", 1, false, tree);
}
