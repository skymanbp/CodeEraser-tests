use super::*;

/// The classifier's vocabulary, one probe per PATTERNS row plus
/// the refusal cases — codes are frozen wire positions.
#[test]
fn pattern_codes_cover_the_vocabulary() {
    let cases: [(&str, u8); 10] = [
        ("parse_result", 0),
        ("mod", 0),
        ("my-file", 1),
        ("parseResult", 2),
        ("ParseResult", 3),
        ("README", 4),
        ("MAX_LIMIT", 4),
        ("2026-08-17-notes", 5),
        ("mixed-and_under", 6),
        ("名字", 6),
    ];
    for (s, want) in cases {
        assert_eq!(pattern_code(s), want, "{s}");
    }
}

/// One small tree, every aggregate hand-checked: dense ids in
/// sorted discovery order, parent/depth chains, fanouts, the
/// pattern distribution and both convention bits.
#[test]
fn build_aggregates_a_small_tree_by_hand() {
    let paths: Vec<String> = [
        "README.md",
        "Cargo.toml",
        "src/lib.rs",
        "src/deep/one.rs",
        "src/deep/two-b.rs",
        "docs/Guide.md",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let t = build(&paths);
    assert_eq!(t.dirs.len(), 4, "root + docs + src + src/deep");
    let root = &t.dirs[0];
    assert_eq!((root.depth, root.subdirs, root.files), (0, 2, 2));
    assert_eq!(root.conventions, CONV_README | CONV_CONFIG);
    // sorted discovery: docs(1) < src(2) < src/deep(3)
    let (docs, src, deep) = (&t.dirs[1], &t.dirs[2], &t.dirs[3]);
    assert_eq!((docs.parent, docs.depth, docs.files), (0, 1, 1));
    assert_eq!((src.parent, src.subdirs, src.files), (0, 1, 1));
    assert_eq!((deep.parent, deep.depth, deep.files), (2, 2, 2));
    assert_eq!(deep.patterns[0], 1, "one.rs is lower_snake");
    assert_eq!(deep.patterns[1], 1, "two-b.rs is lower_kebab");
    assert_eq!(docs.patterns[3], 1, "Guide.md is pascal");
    assert_eq!(docs.conventions, 0, "no README, no config in docs/");
}
