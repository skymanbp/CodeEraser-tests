//! G13 for the v2.30 language exams (booklet language-expansion.md
//! §14 item 14): per language, "sampled before audited before scored,
//! and audited before any ladder" as CHECKED git facts — the legs of
//! graph_provenance.rs re-run per exam, plus the pending state: while
//! a language's audit is not committed, no commit may have touched its
//! ladder and no precision doc may exist. Runs git; CI checks out with
//! fetch-depth: 0 and a shallow clone refuses loudly.

use crate::eval_lang_parts::review::audited;
use crate::eval_lang_parts::{AUDIT_TABLES, EXAMS, Exam, PRECISION_DOCS};
use crate::eval_support::{
    assert_all_postdate, assert_docs_postdate_audits, assert_resolver_after_audits, git_in,
    intro_commit, require_full_history,
};

fn sample_path(exam: &Exam) -> String {
    format!("contracts/eval/lang-sample-{}-v1.json", exam.lang)
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
            .map(|(c, _)| intro_commit(&AUDIT_TABLES.path(c)))
            .collect(),
    )
}

/// sample ≺ every audit table ≺ every precision doc's
/// generated_from.commit; with the audit pending, nothing is scored.
#[test]
fn lang_sample_audit_scoring_ordered() {
    require_full_history();
    for exam in &EXAMS {
        let sample = intro_commit(&sample_path(exam));
        let Some(audits) = audits(exam) else {
            for (corpus, _) in exam.corpora {
                assert!(
                    !exists(&PRECISION_DOCS.path(corpus)),
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
        let stems: Vec<String> = exam
            .corpora
            .iter()
            .filter(|(c, _)| exists(&PRECISION_DOCS.path(c)))
            .map(|(c, _)| PRECISION_DOCS.stem(c))
            .collect();
        assert_docs_postdate_audits(&audits, &stems, "scored before the audit froze (G13)");
    }
}

/// With the audit pending, no commit may ever have touched the
/// language's ladder pathspec; once it is in, the shared two-layer
/// resolver tripwire runs with this language's sample and ladder.
#[test]
fn lang_audit_precedes_the_ladder() {
    require_full_history();
    for exam in &EXAMS {
        let sample = intro_commit(&sample_path(exam));
        match audits(exam) {
            Some(audits) => assert_resolver_after_audits(&sample, &audits, exam.ladder, exam.lang),
            None => {
                let ladder = git_in(Some(".."), &["log", "--format=%H", "--", exam.ladder]);
                assert!(
                    ladder.trim().is_empty(),
                    "{}: a ladder landed while the audit is pending (G13)",
                    exam.lang
                );
            }
        }
    }
}
