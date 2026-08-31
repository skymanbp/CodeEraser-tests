use super::*;
use crate::scan::functions;
use crate::scan::lang::Lang;
use crate::scan::spec;

/// Unit names and the edges the module minted for them. Names are
/// returned instead of the units so the tree can die here.
fn measure(lang: Lang, src: &str) -> (Vec<String>, Vec<(usize, usize)>) {
    let grammar = lang.grammar().expect("grammar");
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&grammar).expect("language");
    let tree = parser.parse(src, None).expect("parse");
    let sp = spec::spec(lang);
    let units = functions::extract(tree.root_node(), src.as_bytes(), sp);
    let arcs = edges(&units, src.as_bytes(), sp);
    (units.iter().map(|u| u.name.clone()).collect(), arcs)
}

/// Edges as (caller, callee) NAMES — a failure then reads like the
/// snippet instead of like a pair of indices.
fn arcs(lang: Lang, src: &str) -> Vec<(String, String)> {
    let (names, arcs) = measure(lang, src);
    arcs.into_iter()
        .map(|(a, b)| (names[a].clone(), names[b].clone()))
        .collect()
}

fn arc(from: &str, to: &str) -> (String, String) {
    (from.to_string(), to.to_string())
}

#[test]
fn a_function_that_calls_itself_gets_a_self_arc() {
    let src = "fn f(n: u32) -> u32 { if n == 0 { 0 } else { f(n - 1) } }";
    assert_eq!(arcs(Lang::Rust, src), vec![arc("f", "f")]);
}

#[test]
fn a_method_reaches_its_sibling_through_self_and_through_the_type() {
    let src = "struct S; impl S { fn g(&self) { self.h(); Self::h(); } fn h() {} }";
    assert_eq!(
        arcs(Lang::Rust, src),
        vec![arc("g", "h")],
        "one arc, not two"
    );
}

#[test]
fn a_nested_receiver_is_not_my_own() {
    // The false positive measured on the Rust corpus: DirEntry::path
    // calling self.dent.path() reaches a DIFFERENT path.
    let src = "struct D; impl D { fn path(&self) -> u8 { self.dent.path() } }";
    assert!(arcs(Lang::Rust, src).is_empty());
}

#[test]
fn a_name_two_callables_share_resolves_to_neither() {
    let src = "struct A; struct B;\n\
               impl A { fn path(&self) -> u8 { self.path() } }\n\
               impl B { fn path(&self) -> u8 { 0 } }";
    assert!(arcs(Lang::Rust, src).is_empty(), "undercount beats a guess");
}

#[test]
fn mutual_recursion_inside_one_file_yields_both_arcs() {
    let src = "fn a(n: u32) -> u32 { b(n) }\nfn b(n: u32) -> u32 { a(n) }";
    assert_eq!(arcs(Lang::Rust, src), vec![arc("a", "b"), arc("b", "a")]);
}

#[test]
fn a_call_inside_a_closure_belongs_to_the_closure() {
    // A Rust closure hangs off a let_declaration, not the
    // variable_declarator name_of climbs, so it stays anonymous — the
    // point here is the caller, which is the closure and never f.
    let src = "fn f() {\n    let g = || f();\n}";
    assert_eq!(
        arcs(Lang::Rust, src),
        vec![arc("(anonymous)", "f")],
        "not f -> f"
    );
}

#[test]
fn python_reads_self_but_not_another_object_of_the_same_shape() {
    // Request.prepare calling p.prepare() is the measured twin.
    let src = "def f(n):\n    return f(n - 1)\n\n\
               class R:\n    def prepare(self):\n        p = P()\n        p.prepare()\n";
    assert_eq!(arcs(Lang::Python, src), vec![arc("f", "f")]);
}

#[test]
fn python_reaches_a_sibling_method_through_self() {
    let src = "class C:\n    def g(self):\n        self.g()\n";
    assert_eq!(arcs(Lang::Python, src), vec![arc("g", "g")]);
}

#[test]
fn a_go_method_answers_its_receiver_and_ignores_a_bare_call() {
    let src = "package p\nfunc (t *T) g() { t.g() }\nfunc h() { g() }\n";
    assert_eq!(
        arcs(Lang::Go, src),
        vec![arc("(*T) g", "(*T) g")],
        "a bare g() cannot reach a method"
    );
}

#[test]
fn typescript_reads_this_and_bare_names() {
    let src = "function f(n: number): number { return f(n - 1); }\n\
               class C { g() { this.g(); } }";
    assert_eq!(
        arcs(Lang::TypeScript, src),
        vec![arc("f", "f"), arc("g", "g")]
    );
}

#[test]
fn haskell_equations_of_one_function_are_one_callable() {
    let (names, arcs) = measure(Lang::Haskell, "f :: Int -> Int\nf 0 = 0\nf n = f (n - 1)\n");
    assert_eq!(names, vec!["f", "f"], "one unit per equation (D7)");
    // The recursing equation reaches both equations; the base case
    // calls nothing, so it stays outside the cycle on its own.
    assert_eq!(arcs, vec![(1, 0), (1, 1)]);
}

#[test]
fn a_where_local_and_a_top_level_of_one_name_cancel() {
    let src = "go :: Int -> Int\ngo n = n\n\nh x = go x\n  where go k = go (k - 1)\n";
    assert!(
        arcs(Lang::Haskell, src).is_empty(),
        "two callables spell `go`; neither is proved"
    );
}

#[test]
fn a_language_without_call_syntax_yields_nothing() {
    assert!(edges(&[], b"", spec::spec(Lang::Markdown)).is_empty());
}

#[test]
fn a_receiver_qualified_name_strips_to_its_base() {
    for (name, want) in [
        ("(*T) add", "add"),
        ("(T) add", "add"),
        ("add", "add"),
        ("(anonymous)", "(anonymous)"),
    ] {
        assert_eq!(base_name(name), want, "{name}");
    }
}
