use super::*;
use crate::fourclass::units::segments;
use crate::scan::lang::Lang;

/// (case, before, after, vanished keys, appeared keys).
type Case = (
    &'static str,
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
);

const CASES: [Case; 5] = [
    (
        "a function that leaves is a vanished declaration",
        "fn a() {}\nfn b() {}\n",
        "fn b() {}\n",
        &["a/0"],
        &[],
    ),
    (
        "a const that arrives is an appeared declaration",
        "fn b() {}\n",
        "const K: u8 = 1;\nfn b() {}\n",
        &[],
        &["K"],
    ),
    (
        "visibility is not identity: pub fn a is still a/0",
        "fn a() {}\n",
        "pub fn a() {}\n",
        &[],
        &[],
    ),
    (
        "arity is identity: a/0 leaving and a/1 arriving are two declarations",
        "fn a() {}\n",
        "fn a(x: u8) -> u8 { x }\n",
        &["a/0"],
        &["a/1"],
    ),
    (
        "a key twice on its side is no candidate at all",
        "impl A {\n    fn m(&self) {}\n}\nimpl B {\n    fn m(&self) {}\n}\n",
        "",
        &["impl A", "impl B"],
        &[],
    ),
];

fn keys(ds: &[Decl]) -> Vec<&str> {
    ds.iter().map(|d| d.key.as_str()).collect()
}

#[test]
fn declaration_candidates_are_one_sided_and_multiplicity_one() {
    for (name, before, after, gone, arrived) in CASES {
        let (b, a) = (segments(before, Lang::Rust), segments(after, Lang::Rust));
        let (rem, add) = tables(&b, &a);
        assert_eq!(keys(&rem), gone.to_vec(), "{name}: vanished");
        assert_eq!(keys(&add), arrived.to_vec(), "{name}: appeared");
        // the wire identity is the hash of the key, nothing else
        for d in rem.iter().chain(add.iter()) {
            assert_eq!(
                key_hash(d),
                crate::dedup::tokens::fnv1a(d.key.as_bytes()),
                "{name}"
            );
        }
    }
}
