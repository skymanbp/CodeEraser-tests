//! M5-2 graph-slice instrument: the frozen SITE universe per corpus
//! (design brief §5 — git history: docs/reviews/2026-08-12-m5-2-graph-design.md).
//! The universe is sites, not edges — frozen BEFORE any resolver
//! exists, so the resolver can never choose its own precision
//! denominator; the falsification constants (min_per_lang,
//! r0_share_trigger) are written into the doc before any measurement
//! exists. The instrument skeleton (walker, envelope, gate opening,
//! drift walk) lives in eval_support/universe.rs, shared with the
//! M5-3 t3-universe family.
//!
//! Regenerate — the `--ignored` generator half retired in 0c7c936
//! (M7.5a); revive it with its coeval support (EVAL-SET.md「再生成」):
//!   git show 0c7c936^:cli/tests/eval_graph.rs > cli/tests/eval_graph.rs && git archive 0c7c936^ cli/tests/eval_support | tar -x
//!   cargo test --test it -- --ignored eval_graph:: --nocapture   # per corpus: CE_SLICE_REPO + CE_GRAPH_NAME + CE_GRAPH_TIP
//!   rm -rf cli/tests/eval_graph.rs cli/tests/eval_support   # untracked in both repositories: a plain rm, never an index write below the gitlink

use crate::eval_support;
use crate::eval_support::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Pre-registered falsification constants (design §5), one binding
/// for generator AND gate — duplicated literals were the exact
/// throat-drift shape this file polices elsewhere (Opus review).
fn constants() -> Value {
    json!({"min_per_lang": 15, "r0_share_trigger": 0.80})
}

// the row throat and the scorer are eval_support::site_row /
// site_summary, shared with the v2.30 language exams (eval_lang.rs)
const FAMILY: UniverseFamily = UniverseFamily {
    family: "graph-slice",
    constants,
    summarize: site_summary,
};

/// CI gate, no git, every frozen slice: the shared envelope (summary
/// re-derived by the generator's own scorer, frozen constants and
/// scope, pinned tip, sorted rows) plus the family's own coverage
/// fact — the frozen docs jointly cover all five languages with
/// sites (D2-4 — the 2b exit criterion).
#[test]
fn graph_slice_consistent() {
    let mut lang_sites: BTreeMap<String, u64> = BTreeMap::new();
    each_frozen_doc(FAMILY.family, |path, doc| {
        assert_doc_envelope(path, doc, &FAMILY);
        for (langkind, n) in doc["summary"]["sites_by"].as_object().expect("sites_by") {
            let lang = langkind.split('/').next().expect("lang/kind");
            *lang_sites.entry(lang.to_string()).or_insert(0) += n.as_u64().expect("n");
        }
    });
    eval_support::assert_nonzero_seats(
        &lang_sites,
        &SCOPE_EXTS,
        "no sites in any frozen slice (D2-4)",
    );
}

/// The detector and the frozen self universe must not drift apart
/// silently (design RG3 made a CI fact — Opus review: a spec.rs or
/// md.rs change used to invalidate all five docs with zero red
/// signal). Every sha-matched row is re-detected: per-kind counts
/// must match, and every detected spec must be a substring of its
/// statement window (the 2b exit criterion, previously asserted only
/// on toy fixtures).
#[test]
fn self_universe_tracks_detector() {
    // whole-row equality through the family throat, the floor and the
    // CE_BLESS=1 re-sign are the skeleton every self view shares;
    // this family adds the per-site statement-window check
    assert_self_tracks(&FAMILY, site_row, 25);
    let doc = load(&eval_doc("graph-slice"));
    each_frozen_match(&doc, |row, path, lang, text| {
        let sites = sites_within_windows(path, lang, text);
        assert_eq!(
            row["sites"],
            json!(kind_counts(&sites)),
            "{path}: detector drifted from the frozen universe"
        );
    });
}
