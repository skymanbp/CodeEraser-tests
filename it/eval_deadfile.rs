//! The dead-file exam's CI gate (lane v235_licence, item L3; family and
//! constants in eval_deadfile_parts): at stage Sampled. Every corpus of
//! the exam table has exactly one frozen universe at the exam's
//! generation, its envelope re-derived (summary from its own rows,
//! constants, pinned tip, embedded name, sorted rows); every language of
//! the universes has exactly one frozen sample, and it IS the draw the
//! pre-registered sampler makes from those universes — allocation,
//! primary rows, reserve, both hashes; no review or precision doc exists
//! yet (the blind audit comes after the sample, the precision docs on a
//! later commit). No git, no corpus clone.

use crate::eval_deadfile_parts::generate::{pools, sample_doc};
use crate::eval_deadfile_parts::{
    LATER, SAMPLES, SCHEMAS, SLICE, constants, corpora, doc_path, gate::assert_language_docs,
};
use crate::eval_support::{assert_envelope_core, frozen_docs, load};
use std::collections::BTreeSet;

#[test]
fn every_corpus_has_one_frozen_universe() {
    let mut want = BTreeSet::new();
    for (name, tip) in corpora() {
        let path = doc_path(SLICE.family, name);
        let doc = load(&path);
        assert_eq!(doc["schema"], SCHEMAS.0, "{path}");
        assert_eq!(
            assert_envelope_core(&path, &doc, &SLICE).as_deref(),
            Some(name),
            "{path}"
        );
        assert_eq!(doc["corpus"]["tip"], tip, "{path}: tip drifted");
        want.insert(path);
    }
    let filed: BTreeSet<String> = frozen_docs("deadfile-slice-").into_iter().collect();
    assert_eq!(filed, want, "frozen deadfile-slice set drifted");
}

#[test]
fn every_language_sample_is_the_pre_registered_draw() {
    let (_, sources) = pools();
    assert_language_docs(SAMPLES, SCHEMAS.1, &constants(), |lang, pool, _| {
        sample_doc(lang, pool, &sources)
    });
}

#[test]
fn the_exam_is_at_stage_sampled() {
    for family in LATER {
        let filed = frozen_docs(&format!("{family}-"));
        assert!(
            filed.is_empty(),
            "{family} filed before its audit: {filed:?}"
        );
    }
}
