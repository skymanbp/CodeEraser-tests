//! G13 for the v2.31 flow exams (booklet analysis-track.md §5.5
//! 「顺序」, §13 items 16, 22 and 25): "sampled before reviewed before
//! scored" as CHECKED git facts, read off the three docs'
//! generated_from commits — each on this history, each strictly before
//! the next; the lowering (LOWERING: cli/src/flow/ but its mod.rs, the
//! module table and the judged mask, which are policy) in place at the
//! sample's commit — an ancestor or the commit itself, since the sample
//! was drawn from the tree the lowering landed in — and still from the
//! sample to the review (the blind window reads LOWERING alone: the
//! answers' other paths may move before the precision doc exists);
//! every precision doc held to the code that answered it from its own
//! commit on. A reverse probe holds the gate red for a path that moved
//! after the sample. Runs git; a shallow clone refuses loudly.

use crate::eval_flow_parts::review::REVIEWS;
use crate::eval_flow_parts::{EXAMS, FlowExam, LOWERING, SAMPLES, Stage, exam};
use crate::eval_support::{
    LOCK, assert_doc_answers, assert_tree_holds, first_commit, is_ancestor, is_strict_ancestor,
    load, require_full_history, touched_between,
};
use crate::flow_precision::PRECISIONS;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// What a precision doc's answers come from besides the lowering: the
/// unit extraction, the walk that picks the universe, the language
/// registry and the pinned grammars. The lockfile is read by its pins
/// (LOCK); the rest as history.
const ANSWERED_BY: [&str; 4] = [
    "cli/src/scan/functions.rs",
    "cli/src/scan/walk.rs",
    "cli/src/scan/lang.rs",
    LOCK,
];

/// The answer paths read as history: LOWERING and ANSWERED_BY without
/// the lockfile.
fn history_paths() -> Vec<&'static str> {
    let mut paths = LOWERING.to_vec();
    paths.extend(ANSWERED_BY.iter().filter(|p| **p != LOCK));
    paths
}

/// The generation commits of the docs the exam has filed, in order:
/// the sample, the review once audited, the precision doc once scored.
fn chain(exam: &FlowExam) -> Vec<String> {
    let docs = [
        (SAMPLES, Stage::Sampled),
        (REVIEWS, Stage::Audited),
        (PRECISIONS, Stage::Scored),
    ];
    docs.iter()
        .filter(|(_, at)| exam.stage >= *at)
        .map(|(family, _)| {
            let from = &family.load(exam, exam.lang)["generated_from"];
            from["commit"].as_str().expect("commit").to_string()
        })
        .collect()
}

/// sample ≺ review ≺ precision, each on this history.
#[test]
fn flow_sample_review_precision_ordered() {
    require_full_history();
    for exam in EXAMS.iter() {
        let chain = chain(exam);
        for commit in &chain {
            assert!(
                is_ancestor(commit, "HEAD"),
                "{}: {commit} is outside this history (G13)",
                exam.lang
            );
        }
        for pair in chain.windows(2) {
            assert!(
                is_strict_ancestor(&pair[0], &pair[1]),
                "{}: {} is not strictly after {} (G13)",
                exam.lang,
                pair[1],
                pair[0]
            );
        }
    }
}

/// The lowering (`ladder`) in place at the sample's commit — its first
/// commit an ancestor of it or the commit itself — and untouched from
/// the sample to the review (to HEAD while the review is pending).
fn lowering_first(exam: &FlowExam, ladder: &[&str]) {
    let (lang, why) = (exam.lang, exam.ladder_first);
    let chain = chain(exam);
    let first = first_commit(ladder)
        .unwrap_or_else(|| panic!("{lang}: no lowering, yet the exam says it came first ({why})"));
    assert!(
        is_ancestor(&first, &chain[0]),
        "{lang}: the lowering does not precede the sample, yet the exam says it does ({why})"
    );
    let end = chain.get(1).map_or("HEAD", String::as_str);
    let touched = touched_between(&chain[0], end, ladder);
    assert!(
        touched.trim().is_empty(),
        "{lang}: the lowering moved inside the sample→review blind window (G13):\n{touched}"
    );
}

#[test]
fn flow_lowering_precedes_the_sample() {
    require_full_history();
    for exam in EXAMS.iter() {
        lowering_first(exam, exam.ladder);
    }
}

/// Every precision doc answers the code it names (the generator's
/// rule, flow_precision/mod.rs): generated on a clean tree, at a
/// commit of this history, and no commit since — nor the working tree —
/// has moved LOWERING, ANSWERED_BY or the lockfile's pins; filling the
/// judged mask (flow/mod.rs) retires nothing.
#[test]
fn flow_docs_answer_the_code_they_name() {
    require_full_history();
    let paths = history_paths();
    for exam in EXAMS.iter().filter(|e| e.stage >= Stage::Scored) {
        assert_tree_holds(exam.lang, &paths);
        let from = &load(&PRECISIONS.file(exam, exam.lang))["generated_from"];
        assert_doc_answers(exam.lang, from, &paths, "flow_precision/mod.rs");
    }
}

/// Reverse probes: the lowering leg goes red, by its own message, for
/// a path first committed after the sample (the flow samples
/// themselves) and for one that moved inside the blind window
/// (cli/src/scan/lang.rs, which commit B touched — an ANSWERED_BY path,
/// never a blind-window one).
#[test]
fn flow_provenance_refuses_a_moved_lowering() {
    require_full_history();
    let exam = exam("python");
    let probes = [
        (
            "contracts/eval/flow-sample-*",
            "does not precede the sample",
        ),
        ("cli/src/scan/lang.rs", "blind window"),
    ];
    for (path, said) in probes {
        let err = catch_unwind(AssertUnwindSafe(|| lowering_first(exam, &[path])))
            .expect_err("a moved lowering must refuse");
        let message = err.downcast_ref::<String>().cloned().unwrap_or_default();
        assert!(
            message.contains(said),
            "{path}: refused otherwise: {message}"
        );
    }
}
