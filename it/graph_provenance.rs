//! G13 (design §5): "sampled before audited before scored" as a
//! CHECKED git fact, not a convention. This is deliberately the one
//! graph gate that runs git — CI checks out with fetch-depth: 0 for
//! exactly this test, and a shallow clone refuses loudly instead of
//! passing vacuously.

use crate::eval_support;
use crate::eval_support::{intro_commit, require_full_history};

const CORPORA: [&str; 5] = ["cobra", "requests", "ripgrep", "self", "zod"];

fn audit_intros() -> Vec<String> {
    eval_support::corpus_intros(
        &|c| format!("cli/tests/eval_graph_review/{c}.json"),
        &CORPORA,
    )
}

/// Legs 1 and 3: sample ≺ every audit table ≺ every precision doc's
/// generated_from.commit — the shared ordering walk.
#[test]
fn sample_audit_scoring_ordered() {
    eval_support::assert_audit_scoring_legs(
        "contracts/eval/graph-sample-v1.json",
        &|c| format!("cli/tests/eval_graph_review/{c}.json"),
        &CORPORA,
        "graph-precision",
        "G13",
    );
}

/// Armed tripwire, two layers (review F7: a resolver landing outside
/// the pre-registered ladder/ path must not slip the ordering check).
/// Layer 1: the design-registered resolver home cli/src/graph/ladder
/// must postdate every audit table. Layer 2: EVERY cli/src/graph file
/// in the current tree either predates the sample freeze (detector
/// era) or descends from the audit freeze — nothing graph-shaped may
/// land inside the blind window between sampling and audit.
#[test]
fn audit_precedes_any_resolver() {
    require_full_history();
    let sample = intro_commit("contracts/eval/graph-sample-v1.json");
    eval_support::assert_resolver_after_audits(
        &sample,
        &audit_intros(),
        "cli/src/graph/ladder",
        "graph",
    );
}
