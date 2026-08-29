use super::*;

/// Two Rust functions where the second mentions the first by
/// name: one edge (1 -> 0), none the other way, and the
/// three-char noise floor drops short names.
#[test]
fn mention_edges_are_word_bounded_and_floored() {
    let text = "fn alpha_one() { 1 }\nfn beta_two() { alpha_one() }\nfn ab() { beta_two_x() }\n";
    let tops = top_level(&units::segments(text, crate::scan::lang::Lang::Rust));
    assert_eq!(tops.len(), 3, "three top-level units");
    let mut out = SeamFacts::default();
    push_refs(&mut out, 0, &tops, text);
    // beta_two mentions alpha_one; ab's beta_two_x is NOT a
    // word-bounded beta_two (identifier tail) — no edge
    assert_eq!(out.tables.refs, vec![[0, 1, 0]]);
    assert!(!mentions("xalpha_one()", "alpha_one"), "left bound");
}

/// Two same-key methods in one file: only the ledger's nth tells
/// them apart, and a key-only map billed impl B's churn to impl A.
#[test]
fn churn_join_map_keys_same_key_units_by_nth() {
    let text = "impl A {\n    fn add(&self) { 1 }\n}\nimpl B {\n    fn add(&self) { 2 }\n}\n";
    let all = units::segments(text, crate::scan::lang::Lang::Rust);
    let m = key_map(&all, &top_level(&all));
    assert_eq!(m.get(&("add/1".to_string(), 0)), Some(&0));
    assert_eq!(m.get(&("add/1".to_string(), 1)), Some(&1));
}
