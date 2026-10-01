//! A frozen row's member identity and member list — the leaf the
//! binding (mod.rs) and the sample share. It lives in its own file so
//! sample.rs reads it as a child module instead of `use super::{..}`:
//! a brace group on super resolves to the parent file, and with the
//! parent's `pub mod sample` that is a two-file cycle on the check's
//! cycle axis (the fpr_flow_replay_parts/shape.rs precedent).

use serde_json::{Value, json};

/// A member's identity: its unit, or a fragment's `path:start-end`.
pub fn identity(member: &Value) -> String {
    member["unit"].as_str().map_or_else(
        || {
            let lines = &member["lines"];
            format!(
                "{}:{}-{}",
                member["path"].as_str().expect("path"),
                lines[0],
                lines[1]
            )
        },
        str::to_string,
    )
}

/// A group's members as the set freezes them, sorted by identity:
/// `at` the member's identity, `run` the lines of the run it sent (the
/// lines the core priced — a trimmed fragment's shorter than `at`'s).
pub fn members_of(group: &Value) -> Vec<Value> {
    let mut ids: Vec<(String, Value)> = group["members"]
        .as_array()
        .expect("members")
        .iter()
        .map(|m| (identity(m), m["run"].clone()))
        .collect();
    ids.sort_by(|a, b| a.0.cmp(&b.0));
    ids.into_iter()
        .map(|(at, run)| json!({"at": at, "run": run}))
        .collect()
}
