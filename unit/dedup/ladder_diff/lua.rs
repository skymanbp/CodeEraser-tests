//! Lua: seeded trees with the default search directories, declared
//! roots, `package.path` templates with odd directories and suffixes
//! (a head glued onto the name, a tail after a slash), `require` names
//! split at dots and slashes (empty pieces, stdlib names) and `dofile`
//! paths (`..`, `.`, rooted, drive and colon forms).

use super::common::{Tree, World, leg};
use super::rng::Rng;
use super::tables::t;
use crate::graph::ladder::lua_path::Template;
use crate::scan::lang::Lang;
use std::path::Path;

// fixture: the rooted spellings (absolute, home-relative, drive, colon)
// the load rung must refuse — specifier text, never a path this test opens
const ROOTED: &str = "/x.lua|\u{7e}/x.lua|C:/x.lua|a:b.lua|/a/b.lua";

fn require(rng: &mut Rng) -> String {
    if rng.chance(8) {
        return rng.pick(t("lua STDLIB")).to_string();
    }
    let mut s = String::new();
    for k in 0..1 + rng.below(3) {
        if k > 0 {
            s.push(if rng.chance(70) { '.' } else { '/' });
        }
        s.push_str(rng.mostly(92, t("lua NAMES"), 11));
    }
    s
}

fn load(rng: &mut Rng) -> String {
    if rng.chance(12) {
        return rng.pick(ROOTED).to_string();
    }
    let pieces: Vec<&str> = (0..1 + rng.below(3))
        .map(|_| rng.pick(t("lua LOAD")))
        .collect();
    pieces.join("/")
}

fn tree(rng: &mut Rng, _root: &Path) -> Tree {
    let mut world = World::seeded(rng, (8, 25), t("lua DIRS"), t("lua BASES"));
    world.declare(rng, "lua", "lib|a|spec|x/lua||a/b");
    for _ in 0..rng.below(4) {
        let dir = rng.pick(t("lua T_DIRS")).to_string();
        let suffix = rng.pick(t("lua SUFFIXES")).to_string();
        world.lua.insert(Template { dir, suffix });
    }
    let keep = |f: &str| f.ends_with(".lua");
    let owned = world.sites(rng, keep, (t("lua DIRS"), "zz.lua"), |rng, from| {
        let (kind, spec) = match rng.below(100) {
            0..65 => ("require", require(rng)),
            65..95 => ("load", load(rng)),
            _ => ("import", require(rng)),
        };
        (Lang::Lua, kind, from, spec)
    });
    (world, owned)
}

#[test]
#[ignore = "instrument: needs a core (CE_CORE_BIN); run with --ignored --nocapture"]
fn lua_random_trees_agree() {
    leg("lua", 2, false, tree);
}
