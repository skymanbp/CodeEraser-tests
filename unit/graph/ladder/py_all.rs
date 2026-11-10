//! A package's `__all__` as the walk reads it (lane v235_graphtrust, T3):
//! the names its literal bindings and edits list, and whether any
//! binding or edit is one no literal reads (opaque); nothing where the
//! module never spells the name — the visibility slice's own reading
//! (fourclass/visibility/py.rs), whose whole-list form is the literal
//! list only when nothing is opaque.

use super::{is_init, read};

/// Each block: `want @@ why` over a module's text; `want` is `-` for no
/// `__all__`, else the literal names joined by `,`, then `?` when the
/// list is opaque.
const CASES: &str = r#"
helper,sub @@ a literal list, read in name order
__all__ = ["sub", "helper"]
====
a,b,c @@ a tuple, then `+=` and `extend` / `append` of literals, union
__all__ = ("a",)
__all__ += ["b"]
__all__.append("c")
====
? @@ a concatenation names what no literal reads
__all__ = ["s" + "ub"]
====
? @@ a call
__all__ = names()
====
sub? @@ a literal list augmented by a call: the literal names stand, the list is opaque
__all__ = ["sub"]
__all__ += names()
====
sub? @@ a literal list, then a rebinding no literal reads
__all__ = ["sub"]
__all__ = base + ["x"]
====
- @@ a module that never spells the name; a comment or a docstring is prose
"""__all__ is not here."""
# __all__ = ["no"]
x = 1
"#;

#[test]
fn an_init_reads_its_all_as_the_visibility_slice_does() {
    for block in CASES.trim_start().split("====\n") {
        let mut lines = block.lines();
        let (want, why) = lines
            .next()
            .and_then(|h| h.split_once(" @@ "))
            .expect("want @@ why");
        let body: String = lines.map(|l| format!("{l}\n")).collect();
        let got = match read(&body) {
            None => "-".to_string(),
            Some((names, opaque)) => names.join(",") + if opaque { "?" } else { "" },
        };
        assert_eq!(got, want, "{why}\n--- module ---\n{body}");
    }
}

#[test]
fn only_a_package_init_is_read() {
    assert!(is_init("__init__.py") && is_init("a/b/__init__.py"));
    assert!(!is_init("a/__init__.pyi") && !is_init("a/x__init__.py"));
}
