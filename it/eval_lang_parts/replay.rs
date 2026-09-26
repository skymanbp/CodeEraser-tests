//! The release half of the precision docs' freshness (plan v2.30 step
//! 5). lang_provenance.rs trips per commit when a commit after a doc's
//! generation touches its ladder or the code its answers come from;
//! this `#[ignore]` replay — it reads every pinned corpus clone —
//! re-scores every scored corpus and compares the doc key by key, so a
//! change anywhere else that moves an answer is found before a release
//! (docs/RELEASE.md §0). Only the provenance and the hand-written RG1
//! disposition are the doc's own.
//!   cargo test --test it -- --ignored eval_lang_parts::replay --nocapture

use super::generate::scored_doc;
use super::score::scored;
use super::{EXAMS, PRECISION_DOCS};

/// The doc's own keys, never re-derived.
const OWN: [&str; 2] = ["generated_from", "r0_disposition"];

#[test]
#[ignore = "reads every pinned corpus clone"]
fn lang_precision_replay() {
    let mut moved = Vec::new();
    for exam in EXAMS.iter().filter(|e| scored(e)) {
        for (corpus, tip) in exam.corpora {
            let frozen = PRECISION_DOCS.load(exam, corpus);
            let fresh = scored_doc(exam, corpus, tip);
            let keys = frozen.as_object().expect("doc").keys();
            for key in keys.chain(fresh.as_object().expect("doc").keys()) {
                if !OWN.contains(&key.as_str()) && frozen[key] != fresh[key] {
                    moved.push(format!("{corpus}: {key}"));
                }
            }
            println!("{corpus}: replayed");
        }
    }
    moved.sort();
    moved.dedup();
    assert!(
        moved.is_empty(),
        "answers moved since the docs were generated - regenerate them (generate.rs): {moved:?}"
    );
}
