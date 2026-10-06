use super::pubuse_hash;

/// The hashed projection and the consulted projection MOVE
/// TOGETHER (the md_tests coupling stance): a surface edit shifts
/// the key and re-fires the sweep; a body edit never does — the
/// only road the cross-file staleness class returns for the
/// binder is these two drifting apart. Since step 8 the consulted
/// projection also holds the private use bindings' NAMES and the
/// top-level item names (`owns`), so those move it and a private
/// use's path does not. Since step 5b a glob's path, a `pub extern
/// crate` and an item's visibility are facts too: the glob is
/// followed, the crate bound, and only a pub item is a glob's export.
const PAIRS: &[(&str, bool, &str, &str)] = &[
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
    (
        "a glob's path is a fact (step 5b: the glob is followed)",
        false,
        "pub use crate::a::*;\n",
        "pub use crate::b::*;\n",
    ),
    (
        "a pub extern crate's bound name is a fact",
        false,
        "pub extern crate x;\n",
        "pub extern crate x as y;\n",
    ),
    (
        "an item's visibility is a fact (a glob exports pub items only)",
        false,
        "fn f() {}\n",
        "pub fn f() {}\n",
    ),
];

#[test]
fn hash_moves_exactly_when_the_bound_surface_moves() {
    for (why, same, a, b) in PAIRS {
        assert_eq!(pubuse_hash(a) == pubuse_hash(b), *same, "{why}");
    }
}
