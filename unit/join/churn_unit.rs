use super::*;

/// Two same-key methods across impl blocks: the span inside the
/// SECOND gets nth 1 (the with_nth throat, not a re-derivation),
/// and a span crossing both impls refuses into the top level.
#[test]
fn spans_attribute_to_innermost_or_refuse_to_toplevel() {
    let root = crate::testutil::scratch("join-unitmap");
    std::fs::write(
        root.join("x.rs"),
        "impl A {\n    fn add(&self) {\n        let a = 1;\n    }\n}\n\
             impl B {\n    fn add(&self) {\n        let b = 2;\n    }\n}\n",
    )
    .expect("x.rs");
    let mut map = UnitMap::new(&root);
    let first = map.id_of("x.rs", 2, 4);
    assert_eq!((first.key.as_str(), first.nth), ("add/1", 0));
    let second = map.id_of("x.rs", 7, 9);
    assert_eq!((second.key.as_str(), second.nth), ("add/1", 1));
    let crossing = map.id_of("x.rs", 4, 7);
    assert_eq!(crossing.key, "", "cross-unit span is not guessed");
    std::fs::remove_dir_all(&root).ok();
}
