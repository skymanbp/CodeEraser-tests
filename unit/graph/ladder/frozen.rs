//! The frozen oracle (plan v2.33 wave W2a, coordinator instruction
//! 2026-10-03): the Python, Lua, Go and C / C++ rungs exactly as they
//! stood at a8db74a9, the last commit before resolve/1 moved their
//! search into the core. The six files in oracle/ are byte copies of
//! cli/src/graph/ladder/{py,lua,go,c,c_search,c_index}.rs at that
//! commit, compiled for tests only, each mounted here by `#[path]`
//! (oracle/ holds no mod.rs: a parent there would turn each copy's
//! `super::` into an edge back to it, a cycle); the differential gate
//! (unit/dedup/ladder_diff/) drives the same sites through them and
//! through the core over the real wire and asserts equal outcomes.
//!
//! The glob gives the copies their old parent's names (the Outcome
//! vocabulary, the Scope, the workspace-member throat, paths.rs, whose
//! code has not changed since a8db74a9); their own names shadow it.

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
#[path = "oracle/py.rs"]
mod py;

pub(crate) use c_index::forced_wire;

/// The a8db74a9 dispatcher for the four ladders the core now holds:
/// the empty specifier refused before any rung.
pub(crate) fn resolve(lang: Lang, site: &Site, scope: &Scope) -> Outcome {
    if site.spec.is_empty() {
        return Outcome::Unresolved(Reason::Empty);
    }
    match lang {
        Lang::Python => py::resolve(site.from, site.spec, scope),
        Lang::Go => go::resolve(site.from, site.spec, scope),
        Lang::C | Lang::Cpp => c::resolve(site.from, site.spec, scope),
        Lang::Lua => lua::resolve(site, scope),
        _ => Outcome::Unresolved(Reason::Unsupported),
    }
}
