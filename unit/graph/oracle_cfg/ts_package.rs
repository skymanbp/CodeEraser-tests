//! The package.json half of cli/src/graph/roots.rs as it stood at
//! dd0eec61, the last commit before resolve/1 began carrying the
//! package.json and tsconfig texts and the core began reading them (plan
//! v2.33 W2-text stage E). Not a whole-file copy: `Package`, `package`,
//! `nearest_package` and `read_jsonc` are byte copies of that file's
//! items but for one edit: `read_jsonc` names the frozen JSONC cleaner
//! by its crate path (`crate::graph::jsonc`, mounted for tests from
//! oracle_cfg/jsonc.rs) where it read `super::jsonc` — roots.rs, where
//! this file is mounted, holds nothing but the mount under cfg(test)
//! (it/unit_mounts.rs). The path steps they call (`nearest_up`,
//! `parent_dir`) are roots.rs's own, unchanged. roots.rs mounts this
//! file for tests only and puts the four names back at their old
//! paths, where the frozen TS rungs (unit/graph/ladder/oracle/ts.rs) and
//! the frozen tsconfig chain (oracle_cfg/roots_ts.rs) read them.

use super::{nearest_up, parent_dir};
use serde_json::Value;
use std::path::Path;

/// One package.json surface, enough for the R4/R5 rungs. Clone: the
/// sweep memo hands out per-config parses once and callers keep
/// owned copies.
#[derive(Clone)]
pub struct Package {
    /// Repo-relative directory of the package ("" = repo root).
    pub dir: String,
    pub name: Option<String>,
    pub exports: Option<Value>,
    /// Union of dependencies/devDependencies/peerDependencies/
    /// optionalDependencies keys.
    pub deps: Vec<String>,
}

/// Parse one package.json at a repo-relative path.
pub fn package(root: &Path, rel: &str) -> Option<Package> {
    let doc = read_jsonc(root, rel)?;
    let deps = [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ]
    .iter()
    .filter_map(|k| doc.get(*k).and_then(Value::as_object))
    .flat_map(|m| m.keys().cloned())
    .collect();
    Some(Package {
        dir: parent_dir(rel),
        name: doc.get("name").and_then(Value::as_str).map(str::to_string),
        exports: doc.get("exports").cloned(),
        deps,
    })
}

/// Nearest package.json walking up from `from_dir`.
pub fn nearest_package(root: &Path, from_dir: &str) -> Option<Package> {
    package(root, &nearest_up(root, from_dir, "package.json")?)
}

pub(crate) fn read_jsonc(root: &Path, rel: &str) -> Option<Value> {
    let text = std::fs::read_to_string(root.join(rel)).ok()?;
    serde_json::from_str(&crate::graph::jsonc::clean(&text)).ok()
}
