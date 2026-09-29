use super::*;

/// The producer's half of the S1 vocabulary: the seven stem facts as
/// shape bits, one probe per stem the core's battery classifies
/// (StructureProps.shapeVocabulary pins these same ten stems' bits to
/// their frozen codes) — bit positions are frozen wire positions.
#[test]
fn shape_bits_cover_the_vocabulary() {
    let cases: [(&str, u8); 10] = [
        ("parse_result", 5),
        ("mod", 4),
        ("my-file", 6),
        ("parseResult", 12),
        ("ParseResult", 44),
        ("README", 40),
        ("MAX_LIMIT", 41),
        ("2026-08-17-notes", 22),
        ("mixed-and_under", 7),
        ("名字", 64),
    ];
    for (s, want) in cases {
        assert_eq!(shape_bits(s), want, "{s}");
    }
}

/// One small tree, every aggregate hand-checked: dense ids in
/// sorted discovery order, parent/depth chains, fanouts, the
/// shape distribution and both convention bits.
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
    assert_eq!(deep.shapes.get(&4), Some(&1), "one.rs is lowercase only");
    assert_eq!(
        deep.shapes.get(&6),
        Some(&1),
        "two-b.rs is lowercase + dash"
    );
    assert_eq!(
        docs.shapes.get(&44),
        Some(&1),
        "Guide.md is both cases, first upper"
    );
    assert_eq!(docs.conventions, 0, "no README, no config in docs/");
}
