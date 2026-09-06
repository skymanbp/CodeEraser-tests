//! ci.yml's job mapping, read once (plan v2.29 step 12): the tag gate
//! in release.yml makes two claims about it — which checks MAY be
//! skipped on a tag commit, and which MUST be present and successful
//! before a publish — and both are answered from the same parse here,
//! so release_roster.rs holds the workflow's two lists to one model of
//! the file rather than to two scanners that could drift apart.

use std::collections::BTreeSet;

/// `  name:` — a job key two spaces under `jobs:`.
fn job_header(line: &str) -> Option<String> {
    let rest = line.strip_prefix("  ")?;
    if rest.starts_with(' ') || rest.starts_with('#') {
        return None;
    }
    rest.strip_suffix(':').map(str::to_string)
}

/// One `  <name>:` block of ci.yml's `jobs:` mapping: its job-level
/// `if:` (absent = the job runs on every event), its matrix dimensions,
/// and whether that matrix is an `include:` table — whose surfaced
/// check names this model does not try to derive.
struct Job {
    name: String,
    cond: Option<String>,
    dims: Vec<Vec<String>>,
    include: bool,
}

impl Job {
    /// The check-run names GitHub surfaces for this job: the product of
    /// its matrix dimensions in declaration order, `name (a, b)`, or the
    /// bare key when it has no matrix.
    fn checks(&self) -> Vec<String> {
        assert!(
            !self.include,
            "{}: an include: matrix names its checks by its own rows",
            self.name
        );
        let mut combos = vec![String::new()];
        for dim in &self.dims {
            combos = spread(&combos, dim);
        }
        combos
            .iter()
            .map(|c| match c.is_empty() {
                true => self.name.clone(),
                false => format!("{} ({c})", self.name),
            })
            .collect()
    }
}

/// Every combination so far, once per value of the next dimension.
fn spread(combos: &[String], dim: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for combo in combos {
        for value in dim {
            out.push(match combo.is_empty() {
                true => value.clone(),
                false => format!("{combo}, {value}"),
            });
        }
    }
    out
}

/// One line of a job's `matrix:` block: a `key: [a, b]` dimension is
/// absorbed, `include:` is noted, and anything shallower ends the block.
fn absorb(job: &mut Job, line: &str) -> bool {
    let Some(entry) = line.strip_prefix("        ") else {
        return false;
    };
    job.include |= entry.starts_with("include:");
    if let Some((_, list)) = entry.split_once(": [") {
        let values = list.trim_end_matches(']').split(',');
        job.dims
            .push(values.map(|v| v.trim().to_string()).collect());
    }
    true
}

/// ci.yml's `jobs:` mapping, one block per job.
fn jobs(ci: &str) -> Vec<Job> {
    let mut out: Vec<Job> = Vec::new();
    let mut in_matrix = false;
    for line in ci.lines().skip_while(|l| *l != "jobs:") {
        if let Some(name) = job_header(line) {
            out.push(Job {
                name,
                cond: None,
                dims: Vec::new(),
                include: false,
            });
            in_matrix = false;
            continue;
        }
        let Some(job) = out.last_mut() else { continue };
        if let Some(cond) = line.strip_prefix("    if: ") {
            job.cond = Some(cond.to_string());
        }
        if line.trim() == "matrix:" {
            in_matrix = true;
        } else if in_matrix {
            in_matrix = absorb(job, line);
        }
    }
    out
}

/// An `if:` that names schedule or dispatch and never push.
fn never_on_push(cond: &str) -> bool {
    !cond.contains("'push'")
        && (cond.contains("'schedule'") || cond.contains("'workflow_dispatch'"))
}

/// ci.yml jobs whose job-level `if:` names schedule or dispatch and
/// never push — the ones that surface SKIPPED on a tag commit.
pub fn schedule_only_jobs(ci: &str) -> BTreeSet<String> {
    jobs(ci)
        .into_iter()
        .filter(|j| j.cond.as_deref().is_some_and(never_on_push))
        .map(|j| j.name)
        .collect()
}

/// The checks ci.yml surfaces on EVERY event, the tag push included:
/// the jobs carrying no job-level `if:`, under the names GitHub gives
/// their check runs. Measured against a green main push (run
/// 34040780731 / commit 02372fa): `build (ubuntu-latest)`,
/// `build (windows-latest)`, `build-macos`.
pub fn unconditional_checks(ci: &str) -> BTreeSet<String> {
    jobs(ci)
        .iter()
        .filter(|j| j.cond.is_none())
        .flat_map(Job::checks)
        .collect()
}

/// One `KEY="a<sep>b"` line of a workflow's shell block, as a set.
/// SKIPPED_OK separates with spaces; REQUIRED cannot — its names carry
/// spaces of their own — so it separates with `|`.
pub fn quoted_list(yml: &str, key: &str, sep: char) -> BTreeSet<String> {
    let open = format!("{key}=\"");
    let line = yml
        .lines()
        .find_map(|l| l.trim().strip_prefix(&open))
        .unwrap_or_else(|| panic!("the workflow spells {key}"));
    line.trim_end_matches('"')
        .split(sep)
        .map(str::to_string)
        .collect()
}
