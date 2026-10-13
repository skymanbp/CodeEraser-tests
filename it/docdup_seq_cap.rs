//! A segment whose shingle sequence passes the core's `doc_seq_cap`
//! (plan v2.33 step 11b item 3). A sequence repeats a shingle as often as
//! its text does, so a paragraph that cycles a few words is small as a
//! set — it passes the set ceiling and the coarse filter pairs it with its
//! twin — and long as a sequence. The core degraded the whole judgment
//! request for it, and the lockstep loop called that "cap mirror drift";
//! now the segment is counted with the over-cap ones by name and only its
//! pairs go unjudged — the rest of the tree is judged as before.

use crate::common;
use crate::common::core_bin;
use serde_json::Value;

/// 66 words of prose, verbatim twice: the pair the run still reports.
const SHARED: &str = "Every ledger of this kind is written once and read many times, so the reader \
is the one to please: each row lands whole, in the order it arrived, and no later row ever moves \
an earlier one. Rotation copies nothing at all; it closes the file and opens another, and the \
oldest rows retire together the day their file does, never one at a time.\n";

#[test]
fn a_sequence_over_the_cap_is_counted_by_name_and_the_rest_is_judged() {
    let cap = codeeraser::tables::get().limits.docdup.doc_seq_cap;
    let long = "amber birch cedar delta ember fjord grove ".repeat(cap / 7 + 8);
    let doc = format!(
        "--- a.md\n{SHARED}\n{long}\n--- b.md\n{SHARED}\n{long}\n--- main.rs\nfn main() {{}}\n"
    );
    let dir = common::doc_tree("docdup-seq-cap", &doc);
    let core = core_bin();
    let out = common::run_ce(&dir, &["docdup", ".", "--format", "json", "--core", &core]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "judged, not refused: {text} {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report: Value = serde_json::from_str(&text).expect("a json report");
    assert_eq!(report["counts"]["over_cap_segments"], 2, "{report}");
    let dups = report["dups"].as_array().expect("dups");
    assert_eq!(dups.len(), 1, "the 66-word twins still judged: {report}");
}
