//! The TOML walk of cli/src/graph/roots.rs as it stood at 1324c927, the
//! last commit before plan v2.33 W2-text stage F moved the Cargo.toml
//! reading into the core: `table_at` is a byte copy of that file's item.
//! roots.rs mounts this file for tests only and puts the name back at
//! its old path, where the frozen Cargo reader (oracle_cfg/cargo.rs) and
//! the frozen pyproject reader (oracle_cfg/pyproject.rs) read it.

/// Walk one dotted key path into a parsed TOML document.
pub(crate) fn table_at<'a>(doc: &'a toml::Table, keys: &[&str]) -> Option<&'a toml::Value> {
    let mut cur = doc.get(keys[0])?;
    for key in &keys[1..] {
        cur = cur.get(key)?;
    }
    Some(cur)
}
