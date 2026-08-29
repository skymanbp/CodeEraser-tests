use super::pubuse_hash;

/// The hashed projection and the consulted projection MOVE
/// TOGETHER (the md_tests coupling stance): a surface edit shifts
/// the key and re-fires the sweep; a body edit never does — the
/// only road the cross-file staleness class returns for the
/// binder is these two drifting apart. Since step 8 the consulted
/// projection also holds the private use bindings' NAMES and the
/// top-level item names (`owns`), so those move it and a private
/// use's path does not.
#[test]
fn hash_moves_exactly_when_the_bound_surface_moves() {
    let pairs: &[(&str, bool, &str, &str)] = &[
        (
            "body-only edit holds",
            true,
            "pub use crate::a::X;\nfn f() {}\n",
            "pub use crate::a::X;\nfn f() {\n    let _ = 1;\n}\n",
        ),
        (
            "surface edit moves",
            false,
            "pub use crate::a::X;\n",
            "pub use crate::b::X;\n",
        ),
        (
            "alias change moves",
            false,
            "pub use crate::a::X as Y;\n",
            "pub use crate::a::X as Z;\n",
        ),
        (
            "a private use's path is no fact",
            true,
            "use crate::a::X;\n",
            "use crate::b::X;\n",
        ),
        (
            "a private use's bound name is a tie-break fact",
            false,
            "use crate::a::X;\n",
            "use crate::a::Y;\n",
        ),
        (
            "a top-level item name is a tie-break fact",
            false,
            "pub use crate::a::X;\nfn f() {}\n",
            "pub use crate::a::X;\nfn g() {}\n",
        ),
    ];
    for (why, same, a, b) in pairs {
        assert_eq!(pubuse_hash(a) == pubuse_hash(b), *same, "{why}");
    }
}
