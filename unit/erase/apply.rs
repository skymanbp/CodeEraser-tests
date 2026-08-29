/// The boundary is a path component, not a prefix: an inert guard
/// on a `suite` submodule would let `suite/x.md` through, an over-
/// eager one would refuse `suitecase.md`.
#[test]
fn below_a_gitlink_is_a_component_boundary() {
    for (path, want) in [
        ("suite", true),
        ("suite/a/b.md", true),
        ("suitecase.md", false),
        ("othersuite/x.md", false),
    ] {
        assert_eq!(super::below(path, "suite"), want, "{path}");
    }
}
