//! The relative join of cli/src/graph/roots.rs as it stood at ca057291,
//! a byte copy: the reference ladders resolve in the core since plan
//! v2.33 W2-text, and the frozen oracles (unit/graph/ladder/oracle,
//! unit/graph/oracle_cfg) were its last readers. roots.rs mounts this
//! file for tests only and puts the name back at its old path.

/// Join a relative spec onto a repo-relative dir, resolving ./ and
/// ../ segments; escaping above the repo root is None (out of tree).
pub(crate) fn join_rel(dir: &str, spec: &str) -> Option<String> {
    let mut parts: Vec<&str> = dir.split('/').filter(|p| !p.is_empty()).collect();
    for seg in spec.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            s => parts.push(s),
        }
    }
    Some(parts.join("/"))
}
