use super::*;

fn keyed(src: &str, lang: Lang) -> (Vec<Unit>, Vec<String>) {
    let units = segments(src, lang);
    let keys = units.iter().map(|u| u.key.clone()).collect();
    (units, keys)
}

#[test]
fn python_functions_become_units() {
    let (units, keys) = keyed(
        "def alpha(a, b):\n    return a\n\ndef beta():\n    pass\n",
        Lang::Python,
    );
    assert_eq!(keys, ["alpha/2", "beta/0"]);
    assert_eq!(owner(&units, 2).unwrap().key, "alpha/2");
    assert_eq!(owner(&units, 3), None); // blank line between defs
}

#[test]
fn markdown_sections_split_on_headings() {
    let (units, keys) = keyed("intro\n# One\nbody\n## Two\nmore\n", Lang::Markdown);
    assert_eq!(keys, ["One", "Two"]);
    assert_eq!(owner(&units, 1), None); // preamble is toplevel
    assert_eq!(owner(&units, 3).unwrap().key, "One");
    assert_eq!(owner(&units, 5).unwrap().key, "Two");
}

/// The owning unit key of `line` in Rust `src` ("" = toplevel).
fn rust_owner(src: &str, line: usize) -> String {
    let units = segments(src, Lang::Rust);
    owner(&units, line)
        .map(|u| u.key.clone())
        .unwrap_or_default()
}

#[test]
fn nested_functions_resolve_to_innermost() {
    let src = "fn outer() {\n    fn inner() {\n        let x = 1;\n    }\n}\n";
    assert_eq!(rust_owner(src, 3), "inner/0");
    assert_eq!(rust_owner(src, 5), "outer/0");
}

/// Attack review F7: impl blocks are units (methods are span-
/// contained, not top-level), and Go methods carry their receiver
/// type in the key.
#[test]
fn impl_blocks_contain_their_methods() {
    let src = "impl A {\n    fn add(&self) {}\n}\nimpl B {\n    fn add(&self) {}\n}\n\
                   impl Show for A {\n    fn show(&self) {}\n}\n";
    assert_eq!(rust_owner(src, 2), "add/1");
    let units = segments(src, Lang::Rust);
    let mut impls: Vec<&str> = units
        .iter()
        .filter(|u| u.key.starts_with("impl "))
        .map(|u| u.key.as_str())
        .collect();
    impls.sort_unstable(); // extraction order is not part of the contract
    // the trait qualifier keeps a type's inherent and trait impls
    // distinct (the FPR replay caught them colliding)
    assert_eq!(impls, ["impl A", "impl B", "impl Show for A"]);
}

/// The multi-param rows pin the M5-close arity repayment (3h
/// blind-audit defect): the receiver-collapsed count keyed every
/// method `/1`; the `parameters` field carries the real list.
/// Grouped `a, b int` stays ONE declaration by standing stance.
#[test]
fn go_method_keys_carry_the_receiver_type_and_real_arity() {
    let src = "func (t T) add(x int) {}\nfunc (u *U) add(x int) {}\nfunc free(x int) {}\n\
                   func (t T) mix(x int, y string) {}\nfunc (t T) grouped(a, b int) {}\n\
                   func (t T) none() {}\n";
    let (_, keys) = keyed(src, Lang::Go);
    let want = [
        "(T) add/1",
        "(*U) add/1",
        "free/1",
        "(T) mix/2",
        "(T) grouped/1",
        "(T) none/0",
    ];
    for k in want {
        assert!(keys.contains(&k.to_string()), "missing {k}; keys: {keys:?}");
    }
}

#[test]
fn named_non_function_units_are_registered() {
    // the register's CLASSES case: a relocated pub const must be
    // attributable by name (R-L2-6's 1-in-35 structural hole)
    let src = "pub const CLASSES: [&str; 4] = [\n    \"a\",\n];\n\nstruct Row {\n    id: u64,\n}\n";
    assert_eq!(rust_owner(src, 2), "CLASSES");
    assert_eq!(rust_owner(src, 6), "Row");
}
