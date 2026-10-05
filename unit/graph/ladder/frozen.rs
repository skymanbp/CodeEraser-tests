//! The frozen oracle (plan v2.33 wave W2a, coordinator instruction
//! 2026-10-03): the Python, Lua, Go and C / C++ rungs exactly as they
//! stood at a8db74a9, the last commit before resolve/1 moved their
//! search into the core, and the R rungs as they stood at c96ab3f6, the
//! last commit before stage B of W2-text moved theirs. The six files
//! oracle/{py,lua,go,c,c_search,c_index}.rs are byte copies of
//! cli/src/graph/ladder/ at a8db74a9; oracle/r.rs and oracle/paths.rs are
//! byte copies of ladder/r/mod.rs and ladder/paths.rs at c96ab3f6 (r.rs's
//! one edit: its `description` module is mounted from
//! ../oracle_cfg/description.rs). paths.rs's code has not changed since
//! a8db74a9, so the earlier copies read the same steps through it. All
//! are compiled for tests only, each mounted here by `#[path]` (oracle/
//! holds no mod.rs: a parent there would turn each copy's `super::` into
//! an edge back to it, a cycle); the differential gate
//! (unit/dedup/ladder_diff/) drives the same sites through them and
//! through the core over the real wire and asserts equal outcomes.
//! oracle_cfg/r_package.rs is the c96ab3f6 package-code expansion
//! (deadcode/targets.rs) the same gate holds the core's `packages` reply
//! against.
//!
//! The glob gives the copies their old parent's names (the Outcome
//! vocabulary, the Scope, the workspace-member throat); their own names
//! shadow it.

use super::*;

#[path = "oracle/c.rs"]
mod c;
#[path = "oracle/c_index.rs"]
mod c_index;
#[path = "oracle/c_search.rs"]
mod c_search;
#[path = "oracle/go.rs"]
mod go;
#[path = "oracle/lua.rs"]
mod lua;
#[path = "oracle/paths.rs"]
mod paths;
#[path = "oracle/py.rs"]
mod py;
#[path = "oracle/r.rs"]
mod r;
#[path = "../oracle_cfg/r_package.rs"]
mod r_package;

pub(crate) use c_index::forced_wire;
pub(crate) use r::description;
pub(crate) use r_package::packages;

/// The a8db74a9 dispatcher for the ladders the core now holds, R's arm
/// added at c96ab3f6: the empty specifier refused before any rung.
pub(crate) fn resolve(lang: Lang, site: &Site, scope: &Scope) -> Outcome {
    if site.spec.is_empty() {
        return Outcome::Unresolved(Reason::Empty);
    }
    match lang {
        Lang::Python => py::resolve(site.from, site.spec, scope),
        Lang::Go => go::resolve(site.from, site.spec, scope),
        Lang::C | Lang::Cpp => c::resolve(site.from, site.spec, scope),
        Lang::Lua => lua::resolve(site, scope),
        Lang::R => r::resolve(site, scope),
        _ => Outcome::Unresolved(Reason::Unsupported),
    }
}
