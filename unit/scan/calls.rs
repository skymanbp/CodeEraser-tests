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

/// (language, snippet, the arcs it must mint, why the row is here).
/// A table rather than a test apiece: the cases differ only in their
/// data, and the repeated per-case scaffold is duplication this repo
/// prices (the metric batteries carry their whys the same way).
type Case = (
    Lang,
    &'static str,
    &'static [(&'static str, &'static str)],
    &'static str,
);

const CASES: &[Case] = &[
    (
        Lang::Rust,
        "fn f(n: u32) -> u32 { if n == 0 { 0 } else { f(n - 1) } }",
        &[("f", "f")],
        "a bare self-call is a cycle of length one",
    ),
    (
        Lang::Rust,
        "struct S; impl S { fn g(&self) { self.h(); Self::h(); } fn h() {} }",
        &[("g", "h")],
        "self.h() and Self::h() reach the same sibling: one arc, not two",
    ),
    (
        Lang::Rust,
        "struct D; impl D { fn path(&self) -> u8 { self.dent.path() } }",
        &[],
        "the measured false positive: self.dent.path() is a DIFFERENT path",
    ),
    (
        Lang::Rust,
        "struct A; struct B;\n\
         impl A { fn path(&self) -> u8 { self.path() } }\n\
         impl B { fn path(&self) -> u8 { 0 } }",
        &[],
        "two callables spell `path`; undercount beats a guess",
    ),
    (
        Lang::Rust,
        "fn a(n: u32) -> u32 { b(n) }\nfn b(n: u32) -> u32 { a(n) }",
        &[("a", "b"), ("b", "a")],
        "mutual recursion inside one file mints both arcs",
    ),
    (
        Lang::Rust,
        "fn f() {\n    let g = || f();\n}",
        &[("(anonymous)", "f")],
        "the caller is the closure, never its host (a Rust closure hangs \
         off a let_declaration, so name_of leaves it anonymous)",
    ),
    (
        Lang::Python,
        "def f(n):\n    return f(n - 1)\n\n\
         class R:\n    def prepare(self):\n        p = P()\n        p.prepare()\n",
        &[("f", "f")],
        "Request.prepare calling p.prepare() is the measured twin: not self",
    ),
    (
        Lang::Python,
        "class C:\n    def g(self):\n        self.g()\n",
        &[("g", "g")],
        "self reaches a sibling method",
    ),
    (
        Lang::Go,
        "package p\nfunc (t *T) g() { t.g() }\nfunc h() { g() }\n",
        &[("(*T) g", "(*T) g")],
        "the receiver binding resolves; a bare g() cannot reach a method",
    ),
    (
        Lang::TypeScript,
        "function f(n: number): number { return f(n - 1); }\n\
         class C { g() { this.g(); } }",
        &[("f", "f"), ("g", "g")],
        "this and a bare name both resolve",
    ),
    (
        Lang::Haskell,
        "go :: Int -> Int\ngo n = n\n\nh x = go x\n  where go k = go (k - 1)\n",
        &[],
        "a where-local `go` and a top-level one are two callables",
    ),
];

#[test]
fn every_case_mints_exactly_the_arcs_it_proves() {
    for (lang, src, want, why) in CASES {
        let got = arcs(*lang, src);
        let want: Vec<_> = want
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        assert_eq!(got, want, "{why}\n--- source ---\n{src}");
    }
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
