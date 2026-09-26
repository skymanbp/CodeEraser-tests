//! M5-3d docdup-segments instrument: the frozen document-segment
//! universe per corpus (design vol.2 §5.1-5.2, instruments §9.5) —
//! file inventory with content identity, admitted segments by kind,
//! the four-route exemption ledger and a per-file digest over the
//! full shingle sets. Frozen BEFORE any docdup judge exists (3g), so
//! the coarse filter can never choose its own denominator. The
//! family skeleton is eval_support/family.rs.
//!
//! Regenerate — the `--ignored` generator half retired in 0c7c936
//! (M7.5a); revive it with its coeval support (EVAL-SET.md「再生成」):
//!   git show 0c7c936^:cli/tests/eval_docdup_universe.rs > cli/tests/eval_docdup_universe.rs && git archive 0c7c936^ cli/tests/eval_support | tar -x
//!   cargo test --release --test it -- --ignored eval_docdup_universe:: --nocapture   # per corpus: CE_SLICE_REPO + CE_GRAPH_NAME + CE_GRAPH_TIP
//!   rm -rf cli/tests/eval_docdup_universe.rs cli/tests/eval_support   # untracked in both repositories: a plain rm, never an index write below the gitlink

use crate::eval_support::*;
use codeeraser::docdup::spec::{KIND_HTML_TEXT, KIND_NAMES};
use std::collections::BTreeMap;

const FAMILY: UniverseFamily = UniverseFamily {
    family: "docdup-segments",
    constants: docdup_constants,
    summarize: docdup_summarize,
};

/// CI gate, no git: the family skeleton asserts the envelope (summary
/// re-derived with row-level conservation, frozen constants and
/// scope, pinned tip, sorted rows) and the graph-slice sibling
/// anchor; this family then requires every frozen-era segment kind
/// alive somewhere across the five corpora (html_text's leg is below)
/// and every exemption route either counted or explained in
/// route_notes.
#[test]
fn docdup_segments_consistent() {
    // kinds, exemption classes and ledger keys share one namespace-
    // disjoint accumulator across the five corpora
    let mut totals: BTreeMap<String, u64> = BTreeMap::new();
    FAMILY.each_consistent(|_path, doc| {
        for key in ["segs_by", "exempt_by", "ledger"] {
            sum_obj_into(&doc["summary"][key], &mut totals);
        }
    });
    // html_text (plan v2.30 step 5) joined after the five universes
    // were frozen over the launch extensions, so no frozen row can
    // hold one; its liveness leg is html_text_is_alive_on_the_pages
    let html = KIND_NAMES[KIND_HTML_TEXT as usize];
    let frozen_kinds = KIND_NAMES.iter().copied().filter(|k| *k != html);
    assert_covered(&totals, frozen_kinds, "segments");
    let notes = docdup_constants()["route_notes"].clone();
    for route in ["license_header", "inline_allow", "skeleton_line"] {
        assert!(
            totals.get(route).copied().unwrap_or(0) > 0 || notes[route].as_str().is_some(),
            "exemption route {route} is zero everywhere and unexplained"
        );
    }
}

/// The extraction throat and the frozen self universe must not drift
/// apart silently (ordinary churn shrinks the verifiable set between
/// freezes; the floor only guards against a vacuous gate).
#[test]
fn self_docdup_tracks_segments() {
    assert_self_tracks(&FAMILY, docdup_row, 25);
}

/// The fourth kind's liveness (plan v2.30 step 5): the frozen universes
/// hold no HTML file, so html_text is proven on the repository's own
/// pages through the same row throat the frozen rows use — every page
/// named here must yield html_text segments, stored or ledgered under
/// the admission floor (the GUI page's labels are all short), the
/// site's prose pages must store live ones, and the ledger must show
/// the code and script elements the extractor shed on the way.
#[test]
fn html_text_is_alive_on_the_pages() {
    let root = crate::common::repo_root();
    let mut shed = 0;
    for (page, prose) in [
        ("site/index.html", true),
        ("site/how/index.html", true),
        ("gui/ui/index.html", false),
    ] {
        let text = std::fs::read_to_string(root.join(page)).expect(page);
        let row = docdup_row(page, "html", &text);
        let stored = row["segs_by"]["html_text"].as_u64().unwrap_or(0);
        let short = row["ledger"]["below_floor"].as_u64().unwrap_or(0);
        assert!(stored + short > 0, "{page}: no html_text segment: {row}");
        assert!(!prose || stored > 0, "{page}: no live html_text: {row}");
        shed += row["ledger"]["code_element"].as_u64().unwrap_or(0)
            + row["ledger"]["script_element"].as_u64().unwrap_or(0);
    }
    assert!(shed > 0, "no code or script element shed across the pages");
}
