//! The frozen pyproject reader (plan v2.33 W2-text stage A): lines 120-157
//! and 168-175 of cli/src/graph/roots.rs at 92e728b1, verbatim, the last
//! commit before the core took the key walk and the PEP 508 name over.
//! Mounted by roots.rs under cfg(test) so the frozen Python ladder
//! (ladder/oracle/py.rs) still reads `roots::pyproject`; the
//! differential gate compares it with the core's `parsed` answer.

use super::table_at;
use std::path::Path;

/// Python-side project surface: extra source roots and declared
/// dependency names, both read from the repo-root pyproject.toml
/// (its bytes sit in resolve_key, so answers cannot go stale).
pub struct PyProject {
    /// Directories that act as import roots besides the repo root
    /// and src/ ([tool.setuptools.package-dir] values and
    /// [tool.poetry.packages].from values).
    pub source_dirs: Vec<String>,
    /// [project] dependencies, reduced to bare package names.
    pub deps: Vec<String>,
}

pub fn pyproject(root: &Path) -> Option<PyProject> {
    let text = std::fs::read_to_string(root.join("pyproject.toml")).ok()?;
    // toml 1.x: Value::from_str parses a single VALUE; documents
    // parse as Table
    let doc: toml::Table = text.parse().ok()?;
    let setuptools = table_at(&doc, &["tool", "setuptools", "package-dir"])
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|map| map.values().filter_map(toml::Value::as_str));
    let poetry = table_at(&doc, &["tool", "poetry", "packages"])
        .and_then(toml::Value::as_array)
        .into_iter()
        .flat_map(|pkgs| {
            pkgs.iter()
                .filter_map(|p| p.get("from").and_then(toml::Value::as_str))
        });
    let source_dirs = setuptools.chain(poetry).map(str::to_string).collect();
    let deps = table_at(&doc, &["project", "dependencies"])
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|d| d.as_str())
        .map(dep_name)
        .collect();
    Some(PyProject { source_dirs, deps })
}

/// "requests>=2.31 ; extra" → "requests" (PEP 508 name prefix).
fn dep_name(requirement: &str) -> String {
    requirement
        .find(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_' || c == '.'))
        .map_or(requirement, |i| &requirement[..i])
        .trim()
        .to_string()
}
