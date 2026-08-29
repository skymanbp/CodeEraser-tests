use super::*;

/// The slice-3 defect, red→green at the fact level: a declared
/// [[bin]] path is a target, its undeclared sibling is not; a
/// cabal main-is lands the same way through its stanza roots; a
/// ce.toml-declared root is a target when walked and nothing
/// when it names a missing file.
#[test]
fn declared_targets_come_from_the_manifests() {
    let root = crate::testutil::scratch("dc-targets");
    let sources = [
        "src/tools/gen.rs",
        "src/tools/other.rs",
        "hs/app/Runner.hs",
        "it/main.rs",
    ];
    let mut tree = vec![
        (
            "Cargo.toml",
            "[package]\nname='t'\n[[bin]]\nname='gen'\npath='src/tools/gen.rs'\n",
        ),
        (
            "hs/x.cabal",
            "executable x\n  hs-source-dirs: app\n  main-is: Runner.hs\n",
        ),
    ];
    tree.extend(sources.map(|f| (f, "")));
    crate::testutil::write_tree(&root, &tree);
    let files: BTreeSet<String> = sources.map(String::from).into();
    let d = Declared::gather(&root, &files, &BTreeSet::new());
    assert!(d.hit("src/tools/gen.rs"), "declared [[bin]] path");
    assert!(!d.hit("src/tools/other.rs"), "undeclared sibling");
    assert!(d.hit("hs/app/Runner.hs"), "cabal main-is through its root");
    assert!(
        !d.hit("it/main.rs"),
        "a manifest-less main is no target by itself"
    );
    let declared = ["it/main.rs", "it/gone.rs"].map(String::from).into();
    let d = Declared::gather(&root, &files, &declared);
    assert!(d.hit("it/main.rs"), "declared in ce.toml and walked");
    assert!(
        !d.hit("it/gone.rs"),
        "declared but missing declares nothing"
    );
    std::fs::remove_dir_all(&root).ok();
}
