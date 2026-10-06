//! The frozen oracle (plan v2.33 wave W2a, coordinator instruction
//! 2026-10-03): the Python, Lua, Go and C / C++ rungs exactly as they
//! stood at a8db74a9, the last commit before resolve/1 moved their
//! search into the core, the R rungs as they stood at c96ab3f6, the
//! last commit before stage B of W2-text moved theirs, and the Java
//! rungs as they stood at 27d0d56d, before stage C. The six files
//! oracle/{py,lua,go,c,c_search,c_index}.rs are byte copies of
//! cli/src/graph/ladder/ at a8db74a9; oracle/r.rs and oracle/paths.rs are
//! byte copies of ladder/r/mod.rs and ladder/paths.rs at c96ab3f6 (r.rs's
//! one edit: its `description` module is mounted from
//! ../oracle_cfg/description.rs). paths.rs's code has not changed since
//! a8db74a9, so the earlier copies read the same steps through it.
//! oracle/{java,java_inherit,java_pick,java_sets}.rs are byte copies of
//! ladder/ at 27d0d56d (java.rs's and java_sets.rs's unit-test mounts
//! re-aimed at ../; java_pick.rs's one edit: `unannotated` reads the
//! annotation half of the header lexer from ../oracle_cfg/
//! java_annotation.rs, since the header and type readers stay in cli/src
//! and the walk still hands their output over as Scope::java).
//! oracle/hs.rs is a byte copy of ladder/hs.rs at fa83a48d, before
//! stage D; it reads the frozen cabal reader (../oracle_cfg/cabal.rs)
//! at `crate::graph::cabal` and paths.rs's `one_of`, unchanged.
//! oracle/ts.rs and oracle/ts_node.rs are byte copies of ladder/ at
//! dd0eec61, before stage E; ts.rs reads the frozen tsconfig chain
//! (../oracle_cfg/roots_ts.rs, at `crate::graph::roots_ts`) and the
//! frozen package.json reader (../oracle_cfg/ts_package.rs, at
//! `crate::graph::roots`), both mounted at their old paths.
//! oracle/{rs,rs_use,rs_bind,rs_tree,rs_reexport,rs_surface}.rs are byte
//! copies of ladder/ at 1324c927, before stage F (rs_reexport.rs's one
//! edit: its unit-test mount is dropped — the test it mounted reads the
//! live `pubuse_hash`, ladder/rs_cst.rs); ladder/mod.rs mounts rs_tree
//! and rs_reexport back at `ladder::rs_tree` / `ladder::rs_reexport`,
//! where rs_use.rs reads them, and rs_use.rs reads the frozen Cargo
//! reader (../oracle_cfg/cargo.rs) at `crate::graph::cargo`. All
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
//! vocabulary, the Scope, the workspace-member throat, the Java header
//! types); their own names shadow it.

use super::*;

#[path = "oracle/c.rs"]
mod c;
#[path = "oracle/c_index.rs"]
mod c_index;
pub(crate) use c_index::forced_wire;
#[path = "oracle/c_search.rs"]
mod c_search;
#[path = "oracle/go.rs"]
mod go;
#[path = "oracle/hs.rs"]
mod hs;
#[path = "oracle/java.rs"]
mod java;
#[path = "../oracle_cfg/java_annotation.rs"]
mod java_annotation;
#[path = "oracle/java_sets.rs"]
mod java_sets;
#[path = "oracle/lua.rs"]
mod lua;
#[path = "oracle/paths.rs"]
mod paths;
#[path = "oracle/py.rs"]
mod py;
#[path = "oracle/r.rs"]
mod r;
#[path = "oracle/ts.rs"]
mod ts;
pub(crate) use r::description;
#[path = "../oracle_cfg/r_package.rs"]
mod r_package;
pub(crate) use r_package::packages;
#[path = "oracle/rs.rs"]
mod rs;

/// The a8db74a9 dispatcher for the ladders the core now holds, R's arm
/// added at c96ab3f6, Java's at 27d0d56d, Haskell's at fa83a48d, the
/// TS / TSX one at dd0eec61 and Rust's at 1324c927: the empty specifier
/// refused before any rung.
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
        Lang::Java => java::resolve(site, scope),
        Lang::Rust => rs::resolve(site, scope),
        Lang::Haskell => hs::resolve(site.from, site.spec, scope),
        Lang::TypeScript | Lang::Tsx => ts::resolve(site.from, site.spec, scope),
        _ => Outcome::Unresolved(Reason::Unsupported),
    }
}

/// The 1324c927 workspace-member throat of the R4 rungs (ladder/mod.rs
/// there; its last live reader, the Rust rungs, moved into the core in
/// stage F, and ladder/mod.rs re-exports it for the frozen ones): the
/// in-scope configs of one basename, parsed once per sweep (the memo), and
/// filtered by name — each caller judges the hit count (1 = the
/// member, more = its own ambiguity reason).
pub(crate) fn members<T: Clone + 'static>(
    scope: &Scope,
    basename: &'static str,
    load: impl Fn(&Path, &str) -> Option<T>,
    keep: impl Fn(&T) -> bool,
) -> Vec<T> {
    scope
        .configs
        .iter()
        .filter(|c| c.rsplit('/').next() == Some(basename))
        .filter_map(|c| {
            scope
                .memo
                .cached(basename, c, || load(scope.root, c))
                .as_ref()
                .clone()
        })
        .filter(|t| keep(t))
        .collect()
}
