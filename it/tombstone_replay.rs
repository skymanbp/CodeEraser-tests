//! FPR replay for the tombstone measurement (plan v2.26 step 6; the
//! ledger is docs/FPR-TOMBSTONE.md): every first-parent commit of a
//! history is one changeset — parent blob before, child blob after,
//! the same `measure` the hooks run — and every site is printed for
//! arbitration. A git-history instrument like bench_backfill, so it
//! stays as a standing ignored leg under the EVAL-SET retirement rule.
//!
//!   cargo test --release --test it -- --ignored tombstone_replay --nocapture
//!
//! CE_TOMBSTONE_REPO = the checkout to walk (default: this repo);
//! CE_TOMBSTONE_LIMIT = how many commits, newest first (default: all).

use codeeraser::fourclass::session;
use codeeraser::tombstone::texts::{self, Side};
use codeeraser::tombstone::{self, PairText};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    std::env::var_os("CE_TOMBSTONE_REPO").map_or_else(crate::common::repo_root, PathBuf::from)
}

fn commits(root: &Path, limit: Option<usize>) -> Vec<String> {
    let (ok, out) = crate::common::git_out(root, &["rev-list", "--first-parent", "HEAD"]);
    assert!(ok, "rev-list at {}", root.display());
    let all: Vec<String> = out.lines().map(str::to_string).collect();
    match limit {
        Some(n) => all.into_iter().take(n).collect(),
        None => all,
    }
}

/// One commit's measurement, or None when it has no parent (the root
/// commit erases nothing: there is no before) or git could not pair it.
fn measure_commit(root: &Path, sha: &str) -> Option<tombstone::Findings> {
    let parent = format!("{sha}^");
    crate::common::git_out(root, &["rev-parse", "--verify", "-q", &parent])
        .0
        .then_some(())?;
    let pairs = session::scoped_pairs(root, &[&parent, sha])?;
    let (loaded, _) = texts::load(root, &pairs, Side::Rev(&parent), Side::Rev(sha))?;
    let pairs: Vec<PairText> = loaded
        .iter()
        .map(|l| PairText {
            rel: &l.rel,
            before: &l.before,
            after: &l.after,
            lang: l.lang,
        })
        .collect();
    Some(tombstone::measure(&pairs, &BTreeSet::new()))
}

/// One commit's table row: the counts and every site with the name
/// it bound and an excerpt, for arbitration by reading.
fn row(sha: &str, f: &tombstone::Findings) -> String {
    let sites: Vec<String> = f
        .sites
        .iter()
        .map(|s| {
            format!(
                "{}:{} {} [{}] «{}»",
                s.file,
                s.line,
                s.kind.name(),
                s.name,
                s.excerpt.replace('|', "\\|").replace('\n', " ")
            )
        })
        .collect();
    format!(
        "| {} | {} | {} | {} | {} | {} |",
        &sha[..10],
        f.label,
        f.prose,
        f.erased.len(),
        f.exempt.len(),
        sites.join("; ")
    )
}

#[test]
#[ignore = "git-history instrument: run by hand, see the module header"]
fn tombstone_replay() {
    let root = repo();
    let limit = std::env::var("CE_TOMBSTONE_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok());
    let shas = commits(&root, limit);
    let (mut walked, mut fired, mut label, mut prose, mut exempt) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    println!("| commit | label | prose | erased | exempt | sites (kind, name bound, excerpt) |");
    println!("|---|---|---|---|---|---|");
    for sha in &shas {
        let Some(f) = measure_commit(&root, sha) else {
            continue;
        };
        walked += 1;
        exempt += f.exempt.len();
        if f.label + f.prose == 0 {
            continue;
        }
        fired += 1;
        label += f.label;
        prose += f.prose;
        println!("{}", row(sha, &f));
    }
    println!();
    println!(
        "walked {walked} of {} commits at {}: {fired} with a site — label {label}, prose {prose}; {exempt} changelog-role exemptions",
        shas.len(),
        root.display()
    );
}
