//! The frozen precision-doc verifier of the v2.30 language exams: one
//! corpus's scored sample (`lang-precision-<corpus>-v<g>.json`) against
//! the frozen sample it answers, the frozen audit whose truths it
//! echoes (as the product's walk scores them) and the frozen universe
//! its ledger and walk record must add up to — every number re-derived
//! through the scorer's own functions (score.rs, G1). Split from the
//! gates like review.rs and verify.rs: the tamper gate runs it on
//! forged copies.

use super::score::{PRECISION_SCHEMA, ledger, shape_of, summary};
use super::walk::{Walk, universe_files, walk_record};
use crate::eval_graph_precision_parts::verdict_of;
use crate::eval_lang_parts::review::ECHO;
use crate::eval_lang_parts::{AUDIT_TABLES, Exam, SLICES, slice_constants};
use crate::eval_support::{MIN_WHY, of_corpus, sum_obj_into, tally_add};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// One corpus's precision doc against its exam: the envelope; the walk
/// record; one row per sampled primary in sample order, none in a file
/// the walk refuses, echoing its identity and the audit's truth as the
/// walk scores it, in one of the three answer shapes, its verdict
/// re-derived (G6); the summary re-derived from the rows (G1); the
/// universe ledger re-derived from its tallies and, with the refused
/// sites, equal kind by kind to the frozen slice — with a written
/// disposition when the R1 share passes the pre-registered trigger;
/// and the audit's site gaps, in order, none in a refused file, each
/// site answered in one shape.
pub fn verify_precision(exam: &Exam, corpus: &str, doc: &Value, sample: &Value) {
    exam.assert_envelope(corpus, doc, PRECISION_SCHEMA);
    assert!(
        doc["generated_from"]["commit"].is_string(),
        "{corpus}: generated_from"
    );
    let review = AUDIT_TABLES.load(exam, corpus);
    let slice = SLICES.load(exam, corpus);
    let walk = verify_walk(corpus, doc, &slice);
    let rows = doc["rows"].as_array().expect("rows");
    let sampled = of_corpus(sample["rows"].as_array().expect("rows"), corpus);
    assert_eq!(rows.len(), sampled.len(), "{corpus}: judged row count (G3)");
    let truths = review["rows"].as_array().expect("rows");
    for ((row, s), audit) in rows.iter().zip(sampled).zip(truths) {
        verify_row(corpus, row, s, audit, &walk);
    }
    assert_eq!(
        doc["summary"],
        summary(rows),
        "{corpus}: summary drifted from rows (G1)"
    );
    verify_universe(corpus, doc, &slice);
    verify_gaps(corpus, doc, &review, &walk);
}

/// One judged row against its sampled primary and its audit row: the
/// sampled identity echoed, a file the walk reads, the audit's truth as
/// the walk scores it (the audit's own word beside it only where the
/// walk rewrote it), one answer shape, and the verdict re-derived (G6).
fn verify_row(corpus: &str, row: &Value, s: &Value, audit: &Value, walk: &Walk) {
    let rank = s["rank"].as_str().expect("rank");
    for field in ECHO.iter().chain(&["lang"]) {
        assert_eq!(
            row[*field], s[*field],
            "{corpus}/{rank}: {field} is not the sampled one"
        );
    }
    assert!(
        !walk.refused.contains(row["path"].as_str().expect("path")),
        "{corpus}/{rank}: a sampled site in a file the walk refuses — the exam asks only what the product reads, so its universe needs a re-freeze"
    );
    let said = audit["truth"].as_str().expect("truth");
    let scored = walk.scored(said);
    assert_eq!(
        row["truth"],
        json!(scored),
        "{corpus}/{rank}: not the audit's truth as the walk scores it"
    );
    let beside = if scored == said {
        Value::Null
    } else {
        json!(said)
    };
    assert_eq!(
        row["audit_truth"], beside,
        "{corpus}/{rank}: the audit's word rides beside a truth the walk rewrote, and only there"
    );
    assert!(shape_of(row).is_some(), "{corpus}/{rank}: no answer shape");
    let truth = row["truth"].as_str().expect("truth");
    let verdict = verdict_of(truth, row["answered"].as_str(), row["external"] == true);
    assert_eq!(
        row["verdict"],
        json!(verdict),
        "{corpus}/{rank}: verdict contradicts its row (G6)"
    );
}

/// The walk record against the frozen slice: its refused files are
/// universe files, sorted and distinct, and its tally is theirs.
fn verify_walk(corpus: &str, doc: &Value, slice: &Value) -> Walk {
    let listed: Vec<&str> = doc["walk"]["refused"]
        .as_array()
        .expect("walk.refused")
        .iter()
        .map(|p| p.as_str().expect("path"))
        .collect();
    let refused: BTreeSet<String> = listed.iter().map(|p| p.to_string()).collect();
    let universe = universe_files(slice);
    for path in &refused {
        assert!(
            universe.contains(&path.as_str()),
            "{corpus}: the walk refuses {path}, no file of the frozen universe"
        );
    }
    assert_eq!(
        doc["walk"],
        walk_record(slice, &refused),
        "{corpus}: the walk record is not its refused files' own (sorted, distinct, tallied from the slice)"
    );
    Walk::new(universe, refused)
}

/// The ledger's rates re-derived from its tallies, the tallies and the
/// refused sites equal to the frozen slice kind by kind, and the RG1
/// trigger honoured.
fn verify_universe(corpus: &str, doc: &Value, slice: &Value) {
    let u = &doc["universe"];
    let (mut by, mut refused) = (BTreeMap::new(), BTreeMap::new());
    sum_obj_into(&u["resolution_by"], &mut by);
    sum_obj_into(&u["unresolved_by"], &mut refused);
    let mut per_kind = BTreeMap::new();
    for (key, n) in by.iter().chain(&refused) {
        let (cell, _) = key.rsplit_once('/').expect("lang/kind/outcome");
        tally_add(&mut per_kind, cell, *n);
    }
    sum_obj_into(&doc["walk"]["refused_sites_by"], &mut per_kind);
    assert_eq!(
        json!(per_kind),
        slice["summary"]["sites_by"],
        "{corpus}: the ledger and the refused sites are not the frozen universe"
    );
    assert_eq!(
        *u,
        ledger(by, refused),
        "{corpus}: ledger rates drifted (G1)"
    );
    let trigger = slice_constants()["r0_share_trigger"]
        .as_f64()
        .expect("trigger");
    if u["r0_share"].as_f64().is_some_and(|r| r > trigger) {
        let written = doc["r0_disposition"]
            .as_str()
            .is_some_and(|d| d.chars().count() >= MIN_WHY);
        assert!(
            written,
            "{corpus}: the R1 share passed {trigger} with no written disposition (RG1)"
        );
    }
}

/// The audit's site gaps, in order, none in a refused file, each
/// detected site in one shape.
fn verify_gaps(corpus: &str, doc: &Value, review: &Value, walk: &Walk) {
    let gaps = doc["site_gaps"].as_array().expect("site_gaps");
    let want = review["site_gaps"].as_array().expect("site_gaps");
    assert_eq!(gaps.len(), want.len(), "{corpus}: site gap count");
    for (g, w) in gaps.iter().zip(want) {
        assert!(
            g["path"] == w["path"] && g["line"] == w["line"],
            "{corpus}: not the audit's site gap"
        );
        assert!(
            !walk.refused.contains(g["path"].as_str().expect("path")),
            "{corpus}: a site gap in a file the walk refuses"
        );
        for s in g["sites"].as_array().expect("sites") {
            let named = s["nth"].is_u64() && s["kind"].is_string() && s["spec"].is_string();
            assert!(
                named && shape_of(s).is_some(),
                "{corpus}: a gap site without its answer"
            );
        }
    }
}
