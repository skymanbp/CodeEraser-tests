//! The call-site scope battery of scan/calls.rs, split from its table
//! file (calls.rs) when the C++ overload and Java rows took that file
//! past 300 lines: every rule here is shown refusing one shape and
//! still resolving its neighbour, through the table file's own `arcs`
//! and `named` readings.

use super::tests::{arcs, named};
use crate::testutil::blocks;

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
====
java @@ m -> m @@ `super.m()` names the superclass's m — an override calling its super is delegation, never a recursion — while `this.m()` is the caller's own
class B extends A { void m() { super.m(); } }
----
class B extends A { void m() { this.m(); } }
====
java @@ run -> step, step -> run @@ `this.` reaches the caller's OWN class body — two classes whose methods spell each other's names are no cycle — while inside one class the pair is
class A { void run() { this.step(); } }
class B { void step() { this.run(); } }
----
class A {
    void run() { this.step(); }
    void step() { run(); }
}
====
java @@ m -> m @@ a qualifier naming ANOTHER class proves nothing (undercount), while the class's own name inside its member reaches the member
class K { static void m() {} }
class L { void x() { K.m(); } }
----
class K { void m() { K.m(); } }
====
java @@ m#1 -> m#0 @@ two local classes may share a name, yet each is one class body: a call inside one never lands on the other's overload, and inside its own body it picks by count
class A {
    void f() {
        class L { void m() { this.m(1); } }
    }
    void g() {
        class L { void m(int x) {} }
    }
}
----
class A {
    void f() {
        class L {
            void m() {}
            void m(int x) { this.m(); }
        }
    }
    void g() {
        class L { void m() {} }
    }
}
====
lua @@ g -> g @@ a Lua local is in scope only past its declaration statement (manual §3.5): inside `local g = function() … end` the `g` is whatever `g` was before — while a local declared first and assigned after is the one its body calls
local g = function() return g() end
----
local g
g = function() return g() end
====
lua @@ a -> helper @@ a local declared further down is not yet in scope for the function above it, while one declared before it is
local function a() return helper() end
local function helper() end
----
local function helper() end
local function a() return helper() end
====
lua @@ M.a -> b @@ a table is no class: inside `M.a` a bare `b()` never reaches the member `M.b`, while a local `b` the block declares is the one it calls
function M.a() return b() end
function M.b() end
----
local function b() end
function M.a() return b() end
====
r @@ obj$step -> obj$step @@ an R member reaches only the object the caller's own name spells: `other$step()` inside `obj$step` is some other object's, `obj$step()` is its own
obj$step <- function(n) other$step(n)
----
obj$step <- function(n) obj$step(n)
"#;

#[test]
fn a_callee_is_reached_only_where_the_call_site_can_see_it() {
    for (lang, [want, why], body) in blocks(SCOPE_CASES) {
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
