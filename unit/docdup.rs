use super::*;

/// The seeded counterfactual (3d exit criterion): an Apache-2.0
/// header plus a skeleton-only docstring leave ZERO live
/// segments — the two categories the 2026-08-07 delta review
/// caught self-detonating (F8).
#[test]
fn seeded_license_and_skeleton_survive_as_zero_live_segments() {
    // hard-wrapped filler: real license headers wrap; a single
    // 300-char line would (correctly) fall to the overlong mask
    let filler = format!("# {}\n", "word ".repeat(20).trim()).repeat(3);
    let py = format!(
        "# Licensed under the Apache License, Version 2.0 (the \"License\");\n{filler}\ndef f():\n    \"\"\"Args:\n    Returns:\n    Raises:\n    \"\"\"\n    return 1\n"
    );
    let facts = doc_facts(&py, Lang::Python);
    let live = facts
        .segs
        .iter()
        .filter(|s| s.exempt == exempt::EXEMPT_LIVE)
        .count();
    assert_eq!(live, 0, "seeded categories must not survive live");
    assert_eq!(facts.ledger.license_header, 1);
    assert!(facts.ledger.skeleton_line >= 3);
    assert!(facts.ledger.below_floor >= 1, "skeleton-only docstring");
}

#[test]
fn admission_floor_and_conservation_hold() {
    let md = format!(
        "short paragraph\n\n{}\n",
        "alpha beta gamma delta ".repeat(15).trim()
    );
    let facts = doc_facts(&md, Lang::Markdown);
    assert_eq!(facts.segs.len(), 1, "only the 60-word paragraph admits");
    assert_eq!(facts.ledger.below_floor, 1);
    assert_eq!(facts.segs[0].words.len(), 60);
    assert!(facts.segs[0].shingles.len() <= facts.segs[0].words.len());
}

/// Plan v2.30 step 5b-8: a plain-text paragraph past the admission
/// floor is one live text_para — nothing strips (the line is longer
/// than `doc_line_cap`, which masks a comment line and never prose),
/// nothing exempts, and the ledger stays empty.
#[test]
fn plain_text_prose_admits_as_one_live_paragraph() {
    let txt = format!("{}\n", "alpha beta gamma delta ".repeat(15).trim());
    assert!(txt.len() > spec::table().doc_line_cap);
    let facts = doc_facts(&txt, Lang::Text);
    assert_eq!(facts.segs.len(), 1, "one paragraph, admitted");
    let s = &facts.segs[0];
    assert_eq!(
        (s.kind, s.exempt, s.words.len()),
        (spec::KIND_TEXT_PARA, exempt::EXEMPT_LIVE, 60)
    );
    let lg = &facts.ledger;
    assert_eq!(
        (lg.overlong_line, lg.skeleton_line, lg.below_floor),
        (0, 0, 0)
    );
}
