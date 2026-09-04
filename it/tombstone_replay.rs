//! FPR replay for the tombstone class (plan v2.26 step 6; the ledger
//! is docs/FPR-TOMBSTONE.md): every first-parent commit of a history
//! is one changeset — parent blob before, child blob after, the same
//! `measure` the hooks run, the same tombstone/1 judgment over one
//! core link — and every seated site is printed for arbitration. A
//! git-history instrument like bench_backfill, so it stays as a
//! standing ignored leg under the EVAL-SET retirement rule.
//!
//!   cargo test --release --test it -- --ignored tombstone_replay --nocapture
//!
//! CE_TOMBSTONE_REPO = the checkout to walk (default: this repo);
//! CE_TOMBSTONE_LIMIT = how many commits, newest first (default: all).

use codeeraser::corelink::Link;
use codeeraser::fourclass::session;
use codeeraser::tombstone::role::Witness;
use codeeraser::tombstone::texts::{self, Side};
use codeeraser::tombstone::{self, Judged, PairText, Policy, wire};
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
    Some(tombstone::measure(
        &pairs,
        &BTreeSet::new(),
        &Policy::default(),
    ))
}

/// The segment exemptions of one measurement (the third witness).
fn segments(f: &tombstone::Findings) -> Vec<&tombstone::Exempt> {
    f.exempt
        .iter()
        .filter(|e| e.why == Witness::Segment)
        .collect()
}

/// One commit's table row: the counts, every seated site with the
/// name it bound, its wire integers, its segment's ledger tokens and
/// an excerpt, and every segment the third witness exempted with its
/// tokens — for arbitration by reading, and for setting
/// `SEGMENT_TOKENS`.
fn row(sha: &str, f: &tombstone::Findings, j: &Judged) -> String {
    let mut sites: Vec<String> = f
        .judged_rows(j)
        .map(|r| {
            format!(
                "{}:{} {} [{}] marks={} names={} ledger={} «{}»",
                r.file,
                r.line,
                r.kind.name(),
                r.name,
                r.marks,
                r.names,
                r.ledger,
                r.excerpt.replace('|', "\\|").replace('\n', " ")
            )
        })
        .collect();
    sites.extend(segments(f).iter().map(|e| {
        format!(
            "{}:{} segment ledger={}",
            e.file,
            e.line.unwrap_or(0),
            e.tokens
        )
    }));
    format!(
        "| {} | {} | {} | {} | {} | {} |",
        &sha[..10],
        j.label,
        j.prose,
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
    let mut link: Link = codeeraser::lockstep::open_family(&crate::common::core_bin(), wire::CAP)
        .expect("a core offering tombstone/1");
    let (mut walked, mut fired, mut label, mut prose, mut exempt, mut seg) =
        (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    println!(
        "| commit | label | prose | erased | exempt | sites (kind, name bound, wire integers, segment ledger tokens, excerpt) + exempt segments |"
    );
    println!("|---|---|---|---|---|---|");
    for sha in &shas {
        let Some(f) = measure_commit(&root, sha) else {
            continue;
        };
        walked += 1;
        exempt += f.exempt.len();
        seg += segments(&f).len();
        let j = if f.rows.is_empty() {
            Judged::default()
        } else {
            wire::judge(&mut link, &f, None).unwrap_or_else(|why| panic!("{sha}: {why}"))
        };
        if j.sites.is_empty() && segments(&f).is_empty() {
            continue;
        }
        if !j.sites.is_empty() {
            fired += 1;
        }
        label += j.label;
        prose += j.prose;
        println!("{}", row(sha, &f, &j));
    }
    println!();
    println!(
        "walked {walked} of {} commits at {}: {fired} with a site — label {label}, prose {prose}; {exempt} changelog-role exemptions, {seg} of them segments",
        shas.len(),
        root.display()
    );
}
