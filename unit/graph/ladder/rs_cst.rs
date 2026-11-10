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

/// v2.35 G8 (R1-7): `{self as b}` binds the module itself under `b` —
/// the same surface entry as `a as b`, never a path ending in `self`.
#[test]
fn a_self_alias_binds_the_module_itself() {
    let surface_of = |src: &str| {
        let grammar = crate::scan::lang::Lang::Rust.grammar().expect("rust");
        let tree = crate::scan::ast::parse(src, &grammar).expect("parses");
        super::surface(&tree, src)["uses"].clone()
    };
    let braced = surface_of("pub use crate::a::{self as b};\n");
    assert_eq!(braced, surface_of("pub use crate::a as b;\n"));
    assert!(!braced.to_string().contains("self"), "{braced}");
}
