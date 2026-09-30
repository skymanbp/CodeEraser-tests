//! FPR replay of the flow family PER LANGUAGE over each language's own
//! history (plan v2.31 step 4 commit D; design booklet
//! analysis-track.md §5.5 「回放台账」, §13 item 6): the second
//! reading beside the precision exams — the exams measure a finding
//! against a blind truth, this ledger asks what the corpus's own
//! authors did with it. The corpus is the language's first exam corpus
//! at the exam's pinned tip (eval_flow_parts::EXAMS, so no environment
//! variable can move what it measures), read over the WINDOW newest
//! first-parent commits (common/ledger.rs, the duplicate-write
//! ledger's window), each changed file of the language an event; the
//! counting rule is fpr_flow_replay_parts/book.rs's and lane.rs's.
//!
//!   CE_FPR_FLOW_CORPUS=lua cargo test --release --test it -- --ignored fpr_flow_replay --nocapture
//!
//! The clones are read from CE_FLOW_CLONE_ROOT when it is set (a lane
//! reads the main checkout's), else `.ce-eval/corpora`; git is only
//! read, never written. CE_BLESS=1 merges the measured row into
//! contracts/eval/fpr-flow-v1.json, replacing the row of the same
//! language and keeping the other nine as their own runs left them.
//! With CE_FPR_OUT naming a directory the doc is read and written
//! there instead (common::ledger::out_file), so the ten runs leave the
//! measured tree clean and the doc is copied in once all ten are in.

use crate::bench_support::today;
use crate::common::ledger::WINDOW;
use crate::common::{chain_of, core_bin, repo_root};
use crate::eval_flow_parts::batches::clone_base;
use crate::eval_flow_parts::exam;
use crate::eval_support::{generated_from, pinned_at};
use crate::fpr_flow_replay_parts::render::{advisory_table, kind_table, table};
use crate::fpr_flow_replay_parts::{LEDGER, corpus_of, rank, rate, row, walk::replay};
use codeeraser::corelink::Link;
use std::path::Path;
use std::time::Instant;

#[test]
#[ignore = "history instrument: replays one exam corpus's newest first-parent commits through the real core, minutes; run by hand"]
fn every_flow_language_is_read_over_its_own_history() {
    let lang = std::env::var("CE_FPR_FLOW_CORPUS")
        .expect("CE_FPR_FLOW_CORPUS names the language (eval_flow_parts::EXAMS)");
    let exam = exam(&lang);
    let (corpus, tip) = corpus_of(exam);
    let repo = pinned_at(Path::new(&clone_base()), corpus, tip);
    let commits = chain_of(&repo, tip, Some(WINDOW));
    let (mut link, _) = Link::open(&core_bin()).expect("open core");
    let started = Instant::now();
    let tally = replay(&repo, exam.language(), &commits, &mut link);
    let secs = started.elapsed().as_secs_f64();
    let measured = row(exam, commits.len() - 1, &tally, generated_from(), &today());
    let one = std::slice::from_ref(&measured);
    println!(
        "{}
{}
{}",
        table(one),
        kind_table(one),
        advisory_table(one)
    );
    let (strict, narrow) = (rate(&measured["strict"]), rate(&measured["narrow"]));
    println!(
        "walked {} commits of {corpus} at {} for {lang} in {secs:.1} s: {tally:?}; strict {strict} ppm, narrow {narrow} ppm",
        commits.len() - 1,
        repo.display(),
    );
    if crate::facts::blessing() {
        LEDGER.merge(&repo_root(), measured, &[], rank);
    }
}
