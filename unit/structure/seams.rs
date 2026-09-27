use super::*;

/// Rust functions where the second mentions the first by name: one
/// edge (1 -> 0), none the other way, the three-char noise floor
/// drops short names, and a name spelled only inside a string or a
/// comment (the fourth unit) is no reference at all.
#[test]
fn mention_edges_are_word_bounded_floored_and_blind_to_opaque_spans() {
    let text = "fn alpha_one() { 1 }\nfn beta_two() { alpha_one() }\nfn ab() { beta_two_x() }\nfn gamma_three() -> usize {\n    // alpha_one is history\n    let s = \"alpha_one beta_two\";\n    s.len()\n}\n";
    let lang = crate::scan::lang::Lang::Rust;
    let tops = top_level(&units::segments(text, lang));
    assert_eq!(tops.len(), 4, "four top-level units");
    let mut out = SeamFacts::default();
    push_refs(&mut out, 0, &tops, text, lang);
    // beta_two mentions alpha_one; ab's beta_two_x is NOT a
    // word-bounded beta_two (identifier tail); gamma_three spells two
    // names in a comment and a string only — no edge from any of them
    assert_eq!(out.tables.refs, vec![[0, 1, 0]]);
    assert!(!mentions("xalpha_one()", "alpha_one"), "left bound");
}

/// Two same-key methods in one file: only the anchor tells them
/// apart (impl A's chain against impl B's), and a key-only map billed
/// impl B's churn to impl A.
#[test]
fn churn_join_map_keys_same_key_units_by_anchor() {
    let text = "impl A {\n    fn add(&self) { 1 }\n}\nimpl B {\n    fn add(&self) { 2 }\n}\n";
    let all = units::segments(text, crate::scan::lang::Lang::Rust);
    let anchors = crate::fourclass::anchor::for_units(&all);
    let adds: Vec<String> = all
        .iter()
        .zip(&anchors)
        .filter(|(u, _)| u.key == "add/1")
        .map(|(_, a)| a.clone())
        .collect();
    assert_eq!(adds.len(), 2, "{anchors:?}");
    assert_ne!(adds[0], adds[1], "two chains, two anchors");
    let m = key_map(&all, &top_level(&all));
    assert_eq!(m.get(&("add/1".to_string(), adds[0].clone())), Some(&0));
    assert_eq!(m.get(&("add/1".to_string(), adds[1].clone())), Some(&1));
}
