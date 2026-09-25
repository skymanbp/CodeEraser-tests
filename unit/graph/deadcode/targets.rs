use super::*;

/// The manifests of the fixture tree: a Cargo `[[bin]]`, a cabal
/// executable, an R package with a `Collate` list, one without, and a
/// DESCRIPTION that names no package.
const MANIFESTS: [(&str, &str); 5] = [
    (
        "Cargo.toml",
        "[package]\nname='t'\n[[bin]]\nname='gen'\npath='src/tools/gen.rs'\n",
    ),
    (
        "hs/x.cabal",
        "executable x\n  hs-source-dirs: app\n  main-is: Runner.hs\n",
    ),
    (
        "listed/DESCRIPTION",
        "Package: listed\nCollate: 'aaa.R' b.R 'gone.R'\n",
    ),
    ("bare/DESCRIPTION", "Package: bare\n"),
    ("app/DESCRIPTION", "Title: an app, no package\n"),
];

/// The slice-3 defect, red→green at the fact level: a declared
/// [[bin]] path is a target, its undeclared sibling is not; a
/// cabal main-is lands the same way through its stanza roots; a
/// manifest-less main is no target by itself. Plan v2.30 step 4: an R
/// package's DESCRIPTION declares its `Collate` files under `R/`
/// (a listed file that is missing declares nothing, an unlisted one is
/// no target), else every R file directly in `R/` (not a nested one);
/// a DESCRIPTION without `Package` is no package. A ce.toml-declared
/// root is a target when walked and nothing when it names a missing
/// file.
#[test]
fn declared_targets_come_from_the_manifests() {
    let root = crate::testutil::scratch("dc-targets");
    let sources = [
        "src/tools/gen.rs",
        "src/tools/other.rs",
        "hs/app/Runner.hs",
        "it/main.rs",
        "listed/R/aaa.R",
        "listed/R/b.R",
        "listed/R/extra.R",
        "bare/R/one.R",
        "bare/R/two.r",
        "bare/R/sub/deep.R",
        "app/R/y.R",
    ];
    let mut tree = MANIFESTS.to_vec();
    tree.extend(sources.map(|f| (f, "")));
    crate::testutil::write_tree(&root, &tree);
    let files: BTreeSet<String> = sources.map(String::from).into();
    let d = Declared::gather(&root, &files, &BTreeSet::new());
    let hits: Vec<&str> = sources.into_iter().filter(|f| d.hit(f)).collect();
    assert_eq!(
        hits,
        [
            "src/tools/gen.rs",
            "hs/app/Runner.hs",
            "listed/R/aaa.R",
            "listed/R/b.R",
            "bare/R/one.R",
            "bare/R/two.r",
        ]
    );
    assert_eq!(d.packages().collect::<Vec<_>>(), ["bare", "listed"]);
    let want: BTreeMap<String, BTreeSet<String>> = [
        ("bare", ["bare/R/one.R", "bare/R/two.r"]),
        ("listed", ["listed/R/aaa.R", "listed/R/b.R"]),
    ]
    .map(|(r, c)| (r.to_string(), c.map(String::from).into()))
    .into();
    assert_eq!(*d.package_code_by_root(), want, "each package's code");
    let declared = ["it/main.rs", "it/gone.rs"].map(String::from).into();
    let d = Declared::gather(&root, &files, &declared);
    assert!(d.hit("it/main.rs"), "declared in ce.toml and walked");
    assert!(
        !d.hit("it/gone.rs"),
        "declared but missing declares nothing"
    );
    std::fs::remove_dir_all(&root).ok();
}
