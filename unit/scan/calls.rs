use super::*;
use crate::scan::functions;
use crate::scan::lang::Lang;
use crate::scan::spec;
use crate::testutil::blocks;

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
/// snippet instead of like a pair of indices. A name two units spell
/// (overloads, same-named members of two classes) carries its
/// occurrence, `f#0` / `f#1` — the unit key's own nth — so an arc
/// still says which of them it reached.
pub(super) fn arcs(lang: Lang, src: &str) -> Vec<(String, String)> {
    let (names, arcs) = measure(lang, src);
    let label = |i: usize| {
        let spelled = |n: &&String| **n == names[i];
        if names.iter().filter(spelled).count() < 2 {
            return names[i].clone();
        }
        format!("{}#{}", names[i], names[..i].iter().filter(spelled).count())
    };
    arcs.into_iter()
        .map(|(a, b)| (label(a), label(b)))
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
====
cpp @@ f#0 -> f#1, f#1 -> f#1 @@ a call reaches the one overload its argument count admits: the delegating f(int) is no recursion — the step-2 reading, one callable per name, charged it a point
int f(int a) { return f(a, 0); }
int f(int a, int b) { return b ? f(a, b - 1) : a; }
====
cpp @@ none @@ two overloads admitting the same count: undercount beats a guess (callees::Named::pick: two seats admitting the count reach neither)
int g(int a) { return g(1.5); }
int g(double d) { return 0; }
====
cpp @@ h#0 -> h#0, v -> v @@ a default argument widens the counts an overload admits, and a C variadic lifts its upper bound
int h(int a, int b = 0) { return a ? h(a - 1) : b; }
int h(int a, int b, int c) { return 0; }
void v(int n, ...) { v(1, 2, 3); }
====
cpp @@ none @@ a pack expansion passes a count no reader knows: it admits both overloads, so it reaches neither
void w(int a) {}
void w(int a, int b) {}
template <class... T> void z(T... t) { w(t...); }
====
java @@ f -> f @@ a Java self-call is a cycle of length one
class A { int f(int n) { return n == 0 ? 0 : f(n - 1); } }
====
java @@ g -> h @@ this.h() and A.h() reach the same sibling: one arc, not two
class A { void g() { this.h(); A.h(); } static void h() {} }
====
java @@ f#0 -> f#1, f#1 -> f#1 @@ Java overloads read as C++ ones do
class A {
    int f(int a) { return f(a, 0); }
    int f(int a, int b) { return b == 0 ? a : f(a, b - 1); }
}
====
java @@ v -> v, r -> r @@ a varargs parameter lifts the upper bound, and a receiver parameter takes no argument
class A {
    void v(int... xs) { v(1, 2, 3); }
    void r(A this, int x) { r(x); }
}
====
java @@ next -> hasNext, next -> next @@ an anonymous class is one class body: a bare call and `this.` inside it reach its own members, though no qualifier can name it
class A {
    Object it() {
        return new java.util.Iterator<Object>() {
            public boolean hasNext() { return false; }
            public Object next() { return hasNext() ? this.next() : null; }
        };
    }
}
====
java @@ m#0 -> m#0 @@ each enum constant's body is a class of its own: a self-call reaches its own m, never the other constant's
enum E {
    A { int m(int n) { return n > 0 ? m(n - 1) : 0; } },
    B { int m(int n) { return 0; } };
    abstract int m(int n);
}
====
lua @@ f -> f, M.p -> M.p, M:q -> M:q, spin -> spin @@ a Lua `local function` sees itself (its name is bound before its body, manual §3.4.11); `M.p()` and `self:q()` reach the members of the table the caller's own name spells, and `self:` inside a table constructor's field reaches that table's fields
local function f(n) return f(n - 1) end
function M.p(n) return M.p(n) end
function M:q() return self:q() end
local t = { spin = function(self) return self:spin() end }
====
r @@ fact -> fact, obj$step -> obj$step, ping -> pong, pong -> ping @@ R: a bare call reaches the function its assignment names, and `obj$step()` inside `obj$step` the member the caller's own name spells
fact <- function(n) if (n <= 1) 1 else n * fact(n - 1)
obj$step <- function(n) obj$step(n)
ping <- function(n) pong(n)
pong <- function(n) ping(n)
"#;

/// `caller -> callee, caller -> callee` as name pairs; `none` = none.
pub(super) fn named(spec: &str) -> Vec<(String, String)> {
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
    for (lang, [want, why], src) in blocks(CASES) {
        assert_eq!(arcs(lang, src), named(want), "{why}\n--- source ---\n{src}");
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
