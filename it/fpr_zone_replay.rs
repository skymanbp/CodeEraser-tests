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
//! `guard::zone::landing`, the same function `zone_assess` calls, and
//! since v2.35 (GD1) `guard::zone::grows` against the parent blob's
//! size: a write that does not grow the file is carried and its tier
//! does not fire. There is no second copy of the 25/75 cut anywhere in
//! this file. The population and the scope are the hook's
//! (common::policy: `Lang::sized_path`, the parent's materialised
//! policy, the budget fence).
//!
//!   cargo test --release -j 8 --test it -- --ignored fpr_zone_replay --nocapture
//!
//! CE_FPR_REPO / CE_FPR_TIP / CE_FPR_LIMIT select the corpus exactly
//! as they do for fpr_replay, so the two ledgers share a denominator;
//! CE_FPR_CORPUS names the row (default `self`). CE_BLESS=1 merges
//! the measured row into contracts/eval/fpr-zone-v1.json, replacing
//! the row of the same name and leaving the other corpus alone. With
//! CE_FPR_OUT naming a directory the doc is read and written there
//! instead (common::ledger::out_file), so the measured tree stays clean.

use crate::common::policy::{Blobs, Write, each_write};
use crate::common::{chain, repo_root, tmp};
use crate::fpr_zone_replay_parts::{Corpus, LEDGER, Landed, table};
use codeeraser::guard::zone;
use codeeraser::scan::lang::Lang;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// What the walk accumulates.
#[derive(Default)]
struct Tally {
    events: usize,
    shared: usize,
    softs: BTreeMap<usize, usize>,
    rows: Vec<Landed>,
}

/// One event: the child blob's line count against the two lines the
/// hook would have read, through the product's own `zone::landing`, and
/// whether it grew the file (`zone::grows` on the parent's size).
fn measure(shadow: &Path, w: &Write, blobs: &mut Blobs, t: &mut Tally) {
    let p = w.policy;
    let table = zone::table_for(shadow, &p.cfg, w.rel);
    let (soft, hard) = (
        p.soft.unwrap_or(table.file_lines_warn),
        table.file_lines_fail,
    );
    let lines = blobs.lines(w.commit, w.rel);
    t.events += 1;
    *t.softs.entry(soft).or_default() += 1;
    if Lang::judged_path(Path::new(w.rel)).is_some_and(|l| l.fingerprints()) {
        t.shared += 1;
    }
    let Some(l) = zone::landing(lines, soft, hard, p.tiers) else {
        return;
    };
    let before = if w.status == 'A' { 0 } else { blobs.lines(w.parent, w.rel) };
    let carried = !zone::grows(Ok(before), lines);
    t.rows.push(Landed {
        sha: w.commit[..8].to_string(),
        rel: w.rel.to_string(),
        before,
        lines,
        soft,
        hard,
        permille: l.permille,
        tier: if carried { "observe" } else { l.tier },
        carried,
        shadowed: hard != 0 && lines > hard,
    });
}

/// One corpus replayed end to end.
pub fn replay(repo: &Path, shadow: &Path, name: &str) -> Corpus {
    let commits = chain(repo);
    let mut t = Tally::default();
    let unreadable = each_write(repo, shadow, &commits, |w, b| measure(shadow, w, b, &mut t));
    let tip = commits.last().expect("a tip");
    Corpus {
        name: name.to_string(),
        tip: tip[..8].to_string(),
        commits: commits.len() - 1,
        events: t.events,
        events_shared: t.shared,
        unreadable,
        softs: t.softs,
        rows: t.rows,
    }
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
        // the two corpora are two runs: the row of the same name is
        // replaced, the other kept, both in name order
        LEDGER.merge(&repo_root(), row, &[], |c| {
            (0, c["name"].as_str().unwrap_or_default().to_string())
        });
    }
}
