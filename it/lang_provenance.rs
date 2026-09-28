//! G13 for the v2.30 language exams (booklet language-expansion.md
//! §14 item 14): per language, "sampled before audited before scored,
//! and audited before any ladder" as CHECKED git facts — the legs of
//! graph_provenance.rs re-run per exam, plus the pending state: while
//! a language's audit is not committed, no commit may have touched its
//! ladder and no precision doc may exist. The C family's exams came
//! after their ladder (step 2 landed it before any exam existed) and
//! say so (Exam::ladder_first): for them the gate checks the reversal
//! as a fact and holds the ladder still through the blind window. The
//! last leg holds each precision doc to the code that answered it
//! (plan v2.30 step 5). Runs git; CI checks out with fetch-depth: 0
//! and a shallow clone refuses loudly.

use crate::eval_lang_parts::review::audited;
use crate::eval_lang_parts::score::scored;
use crate::eval_lang_parts::{AUDIT_TABLES, EXAMS, Exam, PRECISION_DOCS, SAMPLES};
use crate::eval_support::{
    assert_all_postdate, assert_blind_window_clear, assert_docs_postdate_audits,
    assert_resolver_after_audits, first_commit, git_in, intro_commit, is_ancestor,
    is_strict_ancestor, require_full_history, touched_between,
};

/// The first commit of an exam's frozen sample, at its generation.
fn sampled(exam: &Exam) -> String {
    intro_commit(&SAMPLES.path(exam, exam.lang))
}

fn exists(path: &str) -> bool {
    crate::common::repo_root().join(path).exists()
}

/// The intro commits of one exam's audit tables, or None while its
/// audit is pending — an exam audits all its corpora in one freeze,
/// and its flag must agree with the files (review::audited).
fn audits(exam: &Exam) -> Option<Vec<String>> {
    if !audited(exam) {
        return None;
    }
    Some(
        exam.corpora
            .iter()
            .map(|(c, _)| intro_commit(&AUDIT_TABLES.path(exam, c)))
            .collect(),
    )
}

/// sample ≺ every audit table ≺ every precision doc's
/// generated_from.commit; with the audit pending, nothing is scored.
#[test]
fn lang_sample_audit_scoring_ordered() {
    require_full_history();
    for exam in &EXAMS {
        let sample = sampled(exam);
        let Some(audits) = audits(exam) else {
            for (corpus, _) in exam.corpora {
                assert!(
                    !exists(&PRECISION_DOCS.path(exam, corpus)),
                    "{corpus}: scored while its audit is pending (G13)"
                );
            }
            continue;
        };
        assert_all_postdate(
            &sample,
            &audits,
            &format!("{}: an audit table predates the sample (G13)", exam.lang),
        );
        let docs: Vec<String> = exam
            .corpora
            .iter()
            .filter(|(c, _)| exists(&PRECISION_DOCS.path(exam, c)))
            .map(|(c, _)| PRECISION_DOCS.file(exam, c))
            .collect();
        assert_docs_postdate_audits(&audits, &docs, "scored before the audit froze (G13)");
    }
}

/// With the audit pending, no commit may ever have touched the
/// language's ladder pathspecs; once it is in, the shared two-layer
/// resolver tripwire runs with this language's sample and ladder. An
/// exam whose ladder came first is held to what it says instead
/// (ladder_first).
#[test]
fn lang_audit_precedes_the_ladder() {
    require_full_history();
    for exam in &EXAMS {
        let sample = sampled(exam);
        let audits = audits(exam);
        if let Some(why) = exam.ladder_first {
            ladder_first(exam, &sample, audits.as_deref(), why);
            continue;
        }
        match audits {
            Some(audits) => assert_resolver_after_audits(&sample, &audits, exam.ladder, exam.lang),
            None => assert!(
                first_commit(exam.ladder).is_none(),
                "{}: a ladder landed while the audit is pending (G13)",
                exam.lang
            ),
        }
    }
}

/// The C family's reversal (booklet §14 item 14), checked as a fact:
/// the ladder's first commit strictly precedes the sample freeze, and
/// the ladder stood still from the sample to every audit table — to
/// HEAD while the audit is pending — with the graph-code blind window
/// (layer 2) held as for every exam once the audit is in.
fn ladder_first(exam: &Exam, sample: &str, audits: Option<&[String]>, why: &str) {
    let first = first_commit(exam.ladder).unwrap_or_else(|| {
        panic!(
            "{}: no ladder, yet the exam says one came first ({why})",
            exam.lang
        )
    });
    assert!(
        is_strict_ancestor(&first, sample),
        "{}: the ladder does not precede the sample, yet the exam says it does ({why})",
        exam.lang
    );
    let head = [String::from("HEAD")];
    for end in audits.unwrap_or(&head) {
        let touched = touched_between(sample, end, exam.ladder);
        assert!(
            touched.trim().is_empty(),
            "{}: the ladder moved inside the sample→audit blind window (G13):\n{touched}",
            exam.lang
        );
    }
    if let Some(audits) = audits {
        assert_blind_window_clear(sample, audits, exam.lang);
    }
}

/// What a precision doc's answers are computed from besides its exam's
/// own ladder (Exam::ladder): the shared rungs and path helpers, the
/// site detector, the resolver-config names, the walk that picks the
/// universe, the language registry and the pinned grammars. A change
/// anywhere else that moves an answer is the release replay's to find
/// (eval_lang_parts/replay.rs).
const ANSWERED_BY: [&str; 9] = [
    "cli/src/graph/ladder/mod.rs",
    "cli/src/graph/ladder/paths.rs",
    "cli/src/graph/roots.rs",
    "cli/src/graph/keys.rs",
    "cli/src/graph/sites*",
    "cli/src/scan/walk.rs",
    "cli/src/scan/outputs.rs",
    "cli/src/scan/lang.rs",
    "cli/Cargo.lock",
];

/// The working tree's uncommitted moves of `paths` - staged or not,
/// untracked included: what touched_between cannot see until the
/// commit lands (step 7's lesson: a read-only face added to lang.rs
/// passed every local leg and CI refused the commit).
fn uncommitted(paths: &[&str]) -> String {
    let mut args = vec!["status", "--porcelain", "--untracked-files=all", "--"];
    args.extend(paths);
    git_in(Some(".."), &args)
}

/// Every scored doc answers the code it names (the generator's rule,
/// eval_lang_parts/generate.rs): generated on a clean tree, at a
/// commit of this history, and no commit since has touched its ladder
/// or ANSWERED_BY — a ladder change leaves the doc to be regenerated.
#[test]
fn lang_docs_answer_the_code_they_name() {
    require_full_history();
    for exam in EXAMS.iter().filter(|e| scored(e)) {
        let mut paths = exam.ladder.to_vec();
        paths.extend(ANSWERED_BY);
        let pending = uncommitted(&paths);
        assert!(
            pending.trim().is_empty(),
            "{}: the code its docs' answers come from moved in the working tree - regenerate them on a clean tree before committing:\n{pending}",
            exam.lang
        );
        for (corpus, _) in exam.corpora {
            let from = &PRECISION_DOCS.load(exam, corpus)["generated_from"];
            let commit = from["commit"].as_str().expect("commit");
            assert_eq!(
                from["dirty"],
                serde_json::json!(false),
                "{corpus}: generated on a dirty tree, so {commit} is not the code that answered"
            );
            assert!(
                is_ancestor(commit, "HEAD"),
                "{corpus}: generated at {commit}, outside this history"
            );
            let since = touched_between(commit, "HEAD", &paths);
            assert!(
                since.trim().is_empty(),
                "{corpus}: the code its answers come from moved after {commit} - regenerate it (eval_lang_parts/generate.rs):\n{since}"
            );
        }
    }
}
