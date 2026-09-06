//! FPR replay for the graded-zone tier map over a git history — the
//! ledger plan §4.2 demands before `[guard] zone_tiers` may change
//! its default (plan v2.29 step 10, C-zone_tiers; the ledger itself
//! is the graded-zone section of docs/FPR-REPLAY.md). The
//! duplicate-write class has fpr_replay and the tombstone class has
//! tombstone_replay; the zone had a feed event and no record at all,
//! which is exactly why it has never enforced.
//!
//! Every changed in-scope file of every first-parent commit is one
//! Write event: the child blob's line count judged against the two
//! lines the hook would have read AT THAT COMMIT — S from the PARENT
//! commit's committed baseline (`softLine`), falling back to the
//! file's warn line, and H off the file's own class table — through
//! `guard::zone::landing`, the same function `zone_assess` calls.
//! There is no second copy of the 25/75 cut anywhere in this file.
//!
//! Scope is REPRODUCED, not approximated: the files that decide it —
//! ce.toml, ce-baseline.json, .gitmodules and every .gitignore /
//! .ceignore — are materialized into a scratch root and advanced
//! commit by commit, so `Config::load`, `walk::Scope` and `Classes`
//! answer as the product would. Nothing else is materialized; line
//! counts come out of git.
//!
//!   cargo test --release -j 8 --test it -- --ignored fpr_zone_replay --nocapture
//!
//! CE_FPR_REPO / CE_FPR_TIP / CE_FPR_LIMIT select the corpus exactly
//! as they do for fpr_replay, so the two ledgers share a denominator;
//! CE_FPR_CORPUS names the row (default `self`). CE_BLESS=1 merges
//! the measured row into contracts/eval/fpr-zone-v1.json, replacing
//! the row of the same name and leaving the other corpus alone.

use crate::common::{blob, chain, changed, git_lines, repo_root, tmp};
use crate::fpr_zone_replay_parts::{Corpus, DOC, GATE_PPM, Landed, SCHEMA, table};
use codeeraser::config::Config;
use codeeraser::guard::zone;
use codeeraser::scan::lang::Lang;
use codeeraser::scan::walk::Scope;
use codeeraser::score::baseline;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The root-relative files whose content decides a write's scope and
/// its two lines. Ignore files matter at every depth, so they are
/// matched on the basename instead.
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

/// The shadow at the chain's first commit.
fn seed_policy(repo: &Path, shadow: &Path, first: &str) {
    for rel in git_lines(repo, &["ls-tree", "-r", "--name-only", first]) {
        if is_policy(&rel) {
            put(shadow, repo, first, &rel, true);
        }
    }
}

/// The declaration the hook would have judged this commit's writes
/// with, and the zone's two baseline inputs beside it.
struct Policy {
    cfg: Config,
    soft: Option<usize>,
    tiers: Option<(usize, usize)>,
}

/// `Err` = today's `ce` cannot read that commit's ce.toml, which is a
/// NAMED gap in the coverage, never a silent zero.
fn policy_of(shadow: &Path) -> Result<Policy, String> {
    let cfg = Config::load(shadow)?;
    let (soft, tiers) = match baseline::document(shadow) {
        Ok(Some(doc)) => zone::envelope(&doc),
        _ => (None, None),
    };
    Ok(Policy { cfg, soft, tiers })
}

/// The replay's fixed context.
struct Walk<'a> {
    repo: &'a Path,
    shadow: &'a Path,
}

/// What the walk accumulates.
#[derive(Default)]
struct Tally {
    events: usize,
    shared: usize,
    unreadable: usize,
    softs: BTreeMap<usize, usize>,
    rows: Vec<Landed>,
}

/// One event: the child blob's line count against the two lines the
/// hook would have read, through the product's own `zone::landing`.
fn measure(w: &Walk, commit: &str, rel: &str, p: &Policy, t: &mut Tally) {
    let table = zone::table_for(w.shadow, &p.cfg, rel);
    let (soft, hard) = (
        p.soft.unwrap_or(table.file_lines_warn),
        table.file_lines_fail,
    );
    let lines = String::from_utf8_lossy(&blob(w.repo, commit, rel))
        .lines()
        .count();
    t.events += 1;
    *t.softs.entry(soft).or_default() += 1;
    if Lang::judged_path(Path::new(rel)).is_some_and(|l| l.grammar().is_some()) {
        t.shared += 1;
    }
    let Some(l) = zone::landing(lines, soft, hard, p.tiers) else {
        return;
    };
    t.rows.push(Landed {
        sha: commit[..8].to_string(),
        rel: rel.to_string(),
        lines,
        soft,
        hard,
        permille: l.permille,
        tier: l.tier,
        shadowed: hard != 0 && lines > hard,
    });
}

/// Every changed in-scope path of one commit, measured against the
/// policy the shadow currently holds (the PARENT's).
fn judge(w: &Walk, commit: &str, ch: &[(char, String)], t: &mut Tally) {
    let Ok(p) = policy_of(w.shadow) else {
        t.unreadable += 1;
        return;
    };
    let mut scope = Scope::new(w.shadow, &p.cfg.exclude).expect("a scope");
    for (status, rel) in ch {
        let path = Path::new(rel.as_str());
        if *status != 'D' && Lang::from_path(path).is_some() && scope.contains(path) {
            measure(w, commit, rel, &p, t);
        }
    }
}

/// The shadow moves to this commit's state, ready for the next one.
fn advance(w: &Walk, commit: &str, ch: &[(char, String)]) {
    for (status, rel) in ch {
        if is_policy(rel) {
            put(w.shadow, w.repo, commit, rel, *status != 'D');
        }
    }
}

/// One corpus replayed end to end.
pub fn replay(repo: &Path, shadow: &Path, name: &str) -> Corpus {
    let commits = chain(repo);
    seed_policy(repo, shadow, &commits[0]);
    let w = Walk { repo, shadow };
    let mut t = Tally::default();
    for step in commits.windows(2) {
        let ch = changed(repo, &step[0], &step[1]);
        judge(&w, &step[1], &ch, &mut t);
        advance(&w, &step[1], &ch);
    }
    let tip = commits.last().expect("a tip");
    Corpus {
        name: name.to_string(),
        tip: tip[..8].to_string(),
        commits: commits.len() - 1,
        events: t.events,
        events_shared: t.shared,
        unreadable: t.unreadable,
        softs: t.softs,
        rows: t.rows,
    }
}

/// The measured row into the frozen doc: the row of the same name is
/// replaced and every other row stays as its own run left it, because
/// the two corpora are two runs.
fn merge(root: &Path, row: Value) {
    let path = root.join(DOC);
    let mut doc: Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| json!({ "corpora": [] }));
    doc["schema"] = json!(SCHEMA);
    doc["generated_from"] = json!("cli/tests/it/fpr_zone_replay.rs");
    doc["gate_ppm"] = json!(GATE_PPM);
    let corpora = doc["corpora"].as_array_mut().expect("a corpora array");
    corpora.retain(|c| c["name"] != row["name"]);
    corpora.push(row);
    corpora.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    std::fs::write(
        &path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&doc).expect("the frozen doc")
        ),
    )
    .expect("write the frozen doc");
}

#[test]
#[ignore = "history instrument: replays a repository's first-parent chain, minutes; run by hand"]
fn every_zone_landing_is_read_against_the_baseline_of_its_parent() {
    let repo = std::env::var_os("CE_FPR_REPO").map_or_else(repo_root, PathBuf::from);
    let name = std::env::var("CE_FPR_CORPUS").unwrap_or_else(|_| "self".into());
    let shadow = tmp("fpr-zone-policy");
    let c = replay(&repo, &shadow, &name);
    for r in c.rows.iter().filter(|r| r.tier == "ask") {
        println!(
            "  INTERCEPT {}",
            serde_json::to_string(r).expect("zone landing")
        );
    }
    let row = c.json();
    println!("{}", table(std::slice::from_ref(&row)));
    println!(
        "walked {} commits at {}: {} events ({} share fpr_replay's denominator), {} unreadable ce.toml",
        c.commits,
        repo.display(),
        c.events,
        c.events_shared,
        c.unreadable
    );
    if crate::facts::blessing() {
        merge(&repo_root(), row);
    }
}
