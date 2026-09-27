use super::*;
use crate::fourclass::units;
use crate::scan::lang::Lang;

/// The ledger claim (plan v2.30 step 5b item 29) on real segments:
/// impl B's `add` keeps its anchor when impl A and its `add` are
/// deleted above it — the shift `nth` suffers — while the two impls'
/// `add`s never share one; a top-level unit's anchor is the empty
/// chain's, the same in both snapshots.
#[test]
fn anchors_of_segments_survive_an_earlier_siblings_deletion() {
    let both = "impl A {\n    fn add(&self) { 1 }\n}\nimpl B {\n    fn add(&self) { 2 }\n}\nfn lonely() {}\n";
    let b_only = "impl B {\n    fn add(&self) { 2 }\n}\nfn lonely() {}\n";
    let table = |text: &str| -> Vec<(String, String)> {
        let segs = units::segments(text, Lang::Rust);
        segs.iter()
            .map(|u| u.key.clone())
            .zip(for_units(&segs))
            .collect()
    };
    let (before, after) = (table(both), table(b_only));
    let adds: Vec<&String> = before
        .iter()
        .filter(|(k, _)| k == "add/1")
        .map(|(_, a)| a)
        .collect();
    let last = |t: &[(String, String)], key: &str| {
        t.iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, a)| a.clone())
            .expect(key)
    };
    // the four claims as one tuple: A's add and B's add differ, B's add
    // and lonely keep their anchors across the deletion, lonely sits
    // alone under the empty chain
    assert_eq!(
        (
            adds.len() == 2 && adds[0] != adds[1],
            last(&before, "add/1") == last(&after, "add/1"),
            last(&before, "lonely/0") == last(&after, "lonely/0"),
            last(&after, "lonely/0").ends_with("#0"),
        ),
        (true, true, true, true),
        "{before:?} / {after:?}"
    );
}
