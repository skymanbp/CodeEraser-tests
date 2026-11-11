//! The size classes' history walk, shared by the graded-zone ledger
//! (fpr_zone_replay) and the hard-budget ledger (fpr_budget_replay):
//! every A/M path of every first-parent commit that the hook would
//! size — `Lang::sized_path` and the PARENT commit's scope — handed to
//! the caller with the policy the hook would have judged it under.
//!
//! Scope is REPRODUCED, not approximated: the files that decide it —
//! ce.toml, ce-baseline.json, .gitmodules and every .gitignore /
//! .ceignore — are materialized into a scratch root and advanced commit
//! by commit, so `Config::load`, `walk::Scope`, `Classes` and the
//! budget fence (`guard::zone::fenced`, the hook's own) answer as the
//! product would. Nothing else is materialized; line counts come out of
//! git through the hook's own counter (`guard::zone::count_lines_in`).

use super::{blob, changed, git_lines};
use codeeraser::config::Config;
use codeeraser::guard::zone;
use codeeraser::scan::lang::Lang;
use codeeraser::scan::walk::Scope;
use codeeraser::score::baseline;
use std::path::Path;

/// The root-relative files whose content decides a write's scope and
/// its lines. Ignore files matter at every depth, so they are matched
/// on the basename instead.
const POLICY: [&str; 3] = ["ce.toml", "ce-baseline.json", ".gitmodules"];

fn is_policy(rel: &str) -> bool {
    let base = rel.rsplit('/').next().unwrap_or(rel);
    POLICY.contains(&rel) || base == ".gitignore" || base == ".ceignore"
}

/// Put one policy file into the shadow at `rev`, or take it away.
fn put(shadow: &Path, repo: &Path, rev: &str, rel: &str, present: bool) {
    let dest = shadow.join(rel);
    if !present {
        let _ = std::fs::remove_file(&dest);
        return;
    }
    std::fs::create_dir_all(dest.parent().expect("a parent")).expect("mkdir");
    std::fs::write(&dest, blob(repo, rev, rel)).expect("write policy");
}

/// The declaration the hook would have judged a commit's writes with
/// (the budget inputs fenced exactly as the hook fences them), and the
/// zone's two baseline inputs beside it.
pub struct Policy {
    pub cfg: Config,
    pub soft: Option<usize>,
    pub tiers: Option<(usize, usize)>,
}

/// `Err` = today's `ce` cannot read that commit's ce.toml, which is a
/// NAMED gap in the coverage, never a silent zero.
fn policy_of(shadow: &Path) -> Result<Policy, String> {
    let (cfg, _) = zone::fenced(shadow, Config::load(shadow)?);
    let (soft, tiers) = match baseline::document(shadow) {
        Ok(Some(doc)) => zone::envelope(&doc),
        _ => (None, None),
    };
    Ok(Policy { cfg, soft, tiers })
}

/// One write the hook would size.
pub struct Write<'a> {
    pub parent: &'a str,
    pub commit: &'a str,
    pub status: char,
    pub rel: &'a str,
    pub policy: &'a Policy,
}

/// A blob's line count, through the hook's counter.
pub fn lines_at(repo: &Path, rev: &str, rel: &str) -> usize {
    zone::count_lines_in(&blob(repo, rev, rel)[..]).expect("a slice reads")
}

/// Every sized write of the chain, oldest first, against the policy the
/// shadow holds at its parent. Returns the commits whose ce.toml today's
/// `ce` cannot read (their writes are not handed out).
pub fn each_write(repo: &Path, shadow: &Path, commits: &[String], mut f: impl FnMut(&Write)) -> usize {
    for rel in git_lines(repo, &["ls-tree", "-r", "--name-only", &commits[0]]) {
        if is_policy(&rel) {
            put(shadow, repo, &commits[0], &rel, true);
        }
    }
    let mut unreadable = 0;
    for step in commits.windows(2) {
        let (parent, commit) = (&step[0], &step[1]);
        let ch = changed(repo, parent, commit);
        match policy_of(shadow) {
            Err(_) => unreadable += 1,
            Ok(policy) => {
                let mut scope = Scope::new(shadow, &policy.cfg.exclude).expect("a scope");
                for (status, rel) in &ch {
                    let path = Path::new(rel.as_str());
                    if *status != 'D' && Lang::sized_path(path).is_some() && scope.contains(path) {
                        let (status, rel, policy) = (*status, rel.as_str(), &policy);
                        f(&Write { parent, commit, status, rel, policy });
                    }
                }
            }
        }
        for (status, rel) in &ch {
            if is_policy(rel) {
                put(shadow, repo, commit, rel, *status != 'D');
            }
        }
    }
    unreadable
}
