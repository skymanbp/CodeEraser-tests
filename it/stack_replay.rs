//! Same-file copies, MEASURED ONLY (v2.35 guard lane, GD6; VERIFIED
//! P-A1; REVIEW-1 Q3/Q8). The write-time probe excludes the target's
//! own file, so a second copy of a function inside ONE file is invisible
//! at write time while `ce dedup` counts the block. Before any class is
//! shipped for it, this ignored instrument measures how often real
//! history does it: every A/M fingerprinting file of every first-parent
//! commit is one write — `resulting` = the child blob, `on_disk` = the
//! parent's — and a STACK run is a same-file clone run of the resulting
//! text (the class's floors) that occurs more often in it than in the
//! on-disk text. Nothing is judged: no frozen doc, no gate, no hook
//! line. It prints events / events with ≥ 1 run per corpus and a seeded
//! sample of the hits for a hand reading.
//!
//!   CE_FPR_CORPUS=self CE_FPR_TIP=<sha> cargo test --release --test it -- --ignored stack_replay --nocapture
//!
//! The corpus names and their windows are the hard-budget ledger's
//! (fpr_budget_replay: `self`, `requests`, the per-language corpora);
//! CE_STACK_SAMPLE (default 60) and CE_STACK_SEED (default 235) shape
//! the sample.

use crate::common::changed;
use crate::common::policy::Blobs;
use codeeraser::dedup::index::Instance;
use codeeraser::dedup::pairs::{self, Filter, Streams, default_min_distinct};
use codeeraser::dedup::relocate::occurrences;
use codeeraser::dedup::tokens::fnv1a_fields;
use codeeraser::dedup::{Params, tokens, winnow};
use codeeraser::scan::lang::Lang;
use std::path::Path;

/// One write's stack runs: (first line, last line, tokens) in the
/// resulting text — same-file clone runs (the product's winnowing and
/// block extension, `pairs::clone_blocks`, over the one file) that the
/// resulting text holds more often than the on-disk text.
pub fn stack_runs(
    resulting: &[tokens::Token],
    on_disk: &[u64],
    p: Params,
) -> Vec<(usize, usize, usize)> {
    let hashes: Vec<u64> = resulting.iter().map(|t| t.hash).collect();
    let instances: Vec<Instance> = winnow::fingerprints(&hashes, p)
        .iter()
        .map(|f| Instance {
            hash: f.hash,
            file: "T".into(),
            start_tok: f.start,
            start_line: resulting[f.start].start_line,
            end_line: resulting[f.start + p.kgram - 1].end_line,
        })
        .collect();
    let streams: Streams = [("T".to_string(), resulting.to_vec())]
        .into_iter()
        .collect();
    let f = Filter {
        min_tokens: p.guarantee(),
        min_distinct: default_min_distinct(),
    };
    let mut out = Vec::new();
    for b in pairs::clone_blocks(&instances, &streams, f)
        .expect("cover/1")
        .blocks
    {
        // the run's tokens: the offset whose first token sits on the
        // block's first line and whose last ends on its last line
        let fits = |at: &usize| {
            resulting[*at].start_line == b.a_start
                && resulting
                    .get(at + b.tokens - 1)
                    .is_some_and(|t| t.end_line == b.a_end)
        };
        let Some(at) = (0..resulting.len()).find(fits) else {
            continue;
        };
        let run = &hashes[at..at + b.tokens];
        if occurrences(&hashes, run) > occurrences(on_disk, run) {
            out.push((b.b_start, b.b_end, b.tokens));
        }
    }
    out.sort();
    out.dedup();
    out
}

/// One hit for the hand reading.
pub struct Hit {
    pub sha: String,
    pub rel: String,
    pub runs: Vec<(usize, usize, usize)>,
}

/// One corpus walked: (events, hits).
pub fn replay(repo: &Path, commits: &[String]) -> (usize, Vec<Hit>) {
    let p = Params::default();
    let (mut events, mut hits) = (0, Vec::new());
    let mut blobs = Blobs::new(repo);
    for step in commits.windows(2) {
        for (status, rel) in changed(repo, &step[0], &step[1]) {
            let Some(lang) = Lang::judged_path(Path::new(&rel)).filter(|l| l.fingerprints()) else {
                continue;
            };
            if status == 'D' {
                continue;
            }
            events += 1;
            let child = tokens::stream(&blobs.bytes(&step[1], &rel), lang).expect("tokens");
            let parent: Vec<u64> = if status == 'A' {
                Vec::new()
            } else {
                let t = tokens::stream(&blobs.bytes(&step[0], &rel), lang).expect("tokens");
                t.iter().map(|t| t.hash).collect()
            };
            let runs = stack_runs(&child, &parent, p);
            if !runs.is_empty() {
                hits.push(Hit {
                    sha: step[1][..8].to_string(),
                    rel,
                    runs,
                });
            }
        }
    }
    (events, hits)
}

/// A seeded sample of `n` hits, in walk order: the hits ranked by the
/// fnv1a of (seed, commit, file) — deterministic and platform-free, a
/// different seed a different draw — and the first `n` kept.
pub fn sample(hits: &[Hit], n: usize, seed: u64) -> Vec<&Hit> {
    let seed = seed.to_le_bytes();
    let mut keyed: Vec<(u64, usize)> = hits
        .iter()
        .enumerate()
        .map(|(i, h)| {
            (
                fnv1a_fields(&[&seed, h.sha.as_bytes(), h.rel.as_bytes()]),
                i,
            )
        })
        .collect();
    keyed.sort();
    let mut picked: Vec<usize> = keyed.into_iter().take(n).map(|(_, i)| i).collect();
    picked.sort();
    picked.into_iter().map(|i| &hits[i]).collect()
}

#[test]
#[ignore = "history instrument: walks a corpus's first-parent chain, minutes; run by hand"]
fn every_write_is_read_for_same_file_copies() {
    let name = std::env::var("CE_FPR_CORPUS").unwrap_or_else(|_| "self".into());
    let (repo, commits) = crate::fpr_budget_replay::corpus(&name);
    let (events, hits) = replay(&repo, &commits);
    let env = |k: &str, d: u64| {
        std::env::var(k)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(d)
    };
    let picked = sample(
        &hits,
        env("CE_STACK_SAMPLE", 60) as usize,
        env("CE_STACK_SEED", 235),
    );
    for h in &picked {
        println!("  SAMPLE {} {} {:?}", h.sha, h.rel, h.runs);
    }
    println!(
        "stack {name} @ {}: {} commits, {events} events, {} with >= 1 stack run ({} sampled); repo {}",
        &commits.last().expect("a tip")[..8],
        commits.len() - 1,
        hits.len(),
        picked.len(),
        repo.display()
    );
}
