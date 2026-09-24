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

/// One case per block, blocks separated by a `====` line: a header
/// `ext @@ arcs @@ why` — the language by file extension, the arcs
/// the snippet must mint as `caller -> callee, caller -> callee`
/// (`none` for none), why the row is here — over the snippet itself.
/// A table rather than a test apiece: the cases differ only in their
/// data, and the repeated per-case scaffold is duplication this repo
/// prices (the metric batteries carry their whys the same way). One
/// literal rather than a slice of typed rows: rows of one tuple shape
/// repeat every ten tokens, and the clone gate read the typed table
/// as clones of itself once the C-family rows joined.
const CASES: &str = r#"
rs @@ f -> f @@ a bare self-call is a cycle of length one
fn f(n: u32) -> u32 { if n == 0 { 0 } else { f(n - 1) } }
====
rs @@ g -> h @@ self.h() and Self::h() reach the same sibling: one arc, not two
struct S; impl S { fn g(&self) { self.h(); Self::h(); } fn h() {} }
====
rs @@ none @@ the measured false positive: self.dent.path() is a DIFFERENT path
struct D; impl D { fn path(&self) -> u8 { self.dent.path() } }
====
rs @@ none @@ two callables spell `path`; undercount beats a guess
struct A; struct B;
impl A { fn path(&self) -> u8 { self.path() } }
impl B { fn path(&self) -> u8 { 0 } }
====
rs @@ a -> b, b -> a @@ mutual recursion inside one file mints both arcs
fn a(n: u32) -> u32 { b(n) }
fn b(n: u32) -> u32 { a(n) }
====
rs @@ (anonymous) -> f @@ the caller is the closure, never its host (a Rust closure hangs off a let_declaration, so name_of leaves it anonymous)
fn f() {
    let g = || f();
}
====
py @@ f -> f @@ Request.prepare calling p.prepare() is the measured twin: not self
def f(n):
    return f(n - 1)

class R:
    def prepare(self):
        p = P()
        p.prepare()
====
py @@ g -> g @@ self reaches a sibling method
class C:
    def g(self):
        self.g()
====
go @@ (*T) g -> (*T) g @@ the receiver binding resolves; a bare g() cannot reach a method
package p
func (t *T) g() { t.g() }
func h() { g() }
====
ts @@ f -> f, g -> g @@ this and a bare name both resolve
function f(n: number): number { return f(n - 1); }
class C { g() { this.g(); } }
====
hs @@ none @@ a where-local `go` and a top-level one are two callables
go :: Int -> Int
go n = n

h x = go x
  where go k = go (k - 1)
====
c @@ f -> f @@ a C self-call is a cycle of length one
int f(int n) { return n ? f(n - 1) : 0; }
====
cpp @@ C::g -> C::g @@ `this->` reaches a sibling member; the member's name carries its class
class C { void g() { this->g(); } };
====
cpp @@ K::a -> K::b, K::b -> K::a @@ members defined out of class share no container, so the owner road (functions::owner_of) is what lets a bare `b()` inside `K::a` reach `K::b`
struct K { void a(); void b(); };
void K::a() { b(); }
void K::b() { a(); }
====
cpp @@ F::twin -> F::make @@ `F::make()` inside a member of F is the class's own static; the `F()` in make is a constructor no unit declares and mints nothing
struct F { static F make(); F twin() { return F::make(); } };
F F::make() { return F(); }
"#;

/// One rule, two halves per block: the shape it must REFUSE, then a
/// `----` line, then the neighbouring shape it must still resolve
/// (its arcs in the header, as above). The pairing is the claim — a
/// negative leg on its own cannot show that a rule is not simply
/// wider than the code it was written against.
const SCOPE_CASES: &str = r#"
rs @@ a -> b, b -> a @@ a bare name never reaches a method: inside `fn drop` the bare `drop` is the prelude's free function, and this repository was measured charging itself a recursion point for it — while a module body holds no members, so bare names still resolve there
struct L; impl Drop for L { fn drop(&mut self) { drop(self.x.take()); } }
----
mod m { fn a() { b() } fn b() { a() } }
====
rs @@ walk -> walk @@ an import written inside a body is the innermost binding of that name, so the bare call is the imported function — the crosscheck corpus held this exact shape and it was the only unit in four corpora the increment moved — while an import of some OTHER name shadows nothing and the recursion still counts
fn symlink(s: u8) { use std::os::unix::fs::symlink; symlink(s); }
----
fn walk(n: u8) { use std::fs::read; walk(n) }
====
py @@ helper -> top @@ a sibling CLASS is not the caller's container — a file-wide lookup minted a FALSE 2-cycle out of those two spellings — while a bare name still reaches what the call site CAN see: A.helper sees module-level top, and top's helper() is the import, never the class method
class A:
    def run(self):
        self.step()
class B:
    def step(self):
        self.run()
----
from utils import helper
class A:
    def helper(self):
        top()
def top():
    helper()
====
cpp @@ A::run -> helper, helper -> top, top -> helper @@ a bare name inside a member reaches its OWN class's members and then what the call site can see — never another class's member (the Python sibling-class pair, in C++) — while a free function at file scope is visible from inside a class body
struct A { void run() { step(); } };
struct B { void step() { run(); } };
----
struct A { void run() { helper(); } };
void helper() { top(); }
void top() { helper(); }
====
cpp @@ K::m -> K::m @@ a qualifier naming ANOTHER class proves nothing (undercount, never a wrong arc), while the class's own name inside its member reaches the member
struct K { void m() {} };
struct L { void x() { K::m(); } };
----
struct K { void m() { K::m(); } };
====
cpp @@ walk -> walk @@ a using-declaration inside a body binds the name it spells, so the bare call is the imported one — the Rust `use` rule
void f(int s) { using ns::f; f(s); }
----
void walk(int n) { using ns::read; walk(n); }
"#;

/// A table's blocks as (language, the header columns after the
/// extension, the body).
fn blocks(table: &str) -> impl Iterator<Item = (Lang, Vec<&str>, &str)> {
    table.trim().split("\n====\n").map(|block| {
        let (head, body) = block.split_once('\n').expect("a header line over a body");
        let mut cols = head.split(" @@ ");
        let ext = cols.next().expect("an extension column");
        let lang =
            Lang::from_path(std::path::Path::new(&format!("x.{ext}"))).expect("a known extension");
        (lang, cols.collect(), body)
    })
}

/// `caller -> callee, caller -> callee` as name pairs; `none` = none.
fn named(spec: &str) -> Vec<(String, String)> {
    spec.split(", ")
        .filter(|arc| *arc != "none")
        .map(|arc| {
            let (a, b) = arc.split_once(" -> ").expect("caller -> callee");
            (a.to_string(), b.to_string())
        })
        .collect()
}

#[test]
fn every_case_mints_exactly_the_arcs_it_proves() {
    for (lang, cols, src) in blocks(CASES) {
        let [want, why]: [&str; 2] = cols.try_into().expect("arcs and why");
        assert_eq!(arcs(lang, src), named(want), "{why}\n--- source ---\n{src}");
    }
}

#[test]
fn a_callee_is_reached_only_where_the_call_site_can_see_it() {
    for (lang, cols, body) in blocks(SCOPE_CASES) {
        let [want, why]: [&str; 2] = cols.try_into().expect("arcs and why");
        let (refused, resolved) = body.split_once("\n----\n").expect("refused ---- resolved");
        assert!(
            arcs(lang, refused).is_empty(),
            "{why}\n--- source ---\n{refused}"
        );
        assert_eq!(
            arcs(lang, resolved),
            named(want),
            "the rule is wider than it should be: {why}\n--- source ---\n{resolved}"
        );
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
