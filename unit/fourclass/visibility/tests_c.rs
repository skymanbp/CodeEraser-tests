//! C / C++ legs (plan v2.30 step 2; design booklet §7 rows C / C++):
//! linkage on the C side, access specifiers, nesting and namespaces on
//! the C++ side, both through the extractor's own root. Every shape
//! here was probed on tree-sitter-c 0.24.2 / tree-sitter-cpp 0.23.4
//! before the reading was written (the scripts/tsprobe transcripts).
//! Two text tables through the shared runner (tests.rs run_tables): a
//! run of same-shaped `check(Lang, …)` legs is the token shape the
//! clone gate pairs with tests.rs, the line-filter loop of the other
//! text tables is the shape it pairs with selfref_tests.rs, and a test
//! pair apiece was the skeleton it paired across the language files.

use super::super::tests::run_tables;

/// `<c|cpp> <source> ⇒ <name:letters …>` per line, newlines spelled
/// `\n`, `#` lines commentary — measured over the functions the
/// extractor sees.
const FNS: &str = r#"
# C: `static` is internal linkage whatever else is spelled beside it; a
# bare `inline` and an `extern` definition are external.
c static int a(void) { return 0; }\nint b(void) { return 0; }\nstatic inline int c(void) { return 0; }\ninline int d(void) { return 0; }\nextern int e(void) { return 0; } ⇒ a:- b:ES c:- d:ES e:ES
# C++ members: the nearest preceding access specifier, the body's
# default (`class` private, `struct` / `union` public), `protected` as
# exported-and-restricted, and a class-static member read by its
# access — never as internal linkage.
cpp class K {\n  void a() {}\npublic:\n  void b() {}\n  static void c() {}\nprotected:\n  void d() {}\nprivate:\n  void e() {}\n};\nstruct S { void f() {} private: void g() {} };\nunion U { void h() {} }; ⇒ K::a:- K::b:ES K::c:ES K::d:ESR K::e:- S::f:ES S::g:- U::h:ES
# The scope bit reads every enclosing class body's own access: a nested
# class's inner specifier governs its members, its position in the
# outer body governs their scope.
cpp class O {\npublic:\n  struct In { void m() {} };\n  class Hid { public: void n() {} };\nprivate:\n  struct Priv { void p() {} };\n}; ⇒ O::In::m:ES O::Hid::n:ES O::Priv::p:E
# Outside a class body: `static` and an anonymous namespace close bit 0
# at any depth; a named namespace, `extern "C"` and a template are
# transparent; an out-of-class member definition reads as exported — it
# cannot see its access specifier from here, and exported is the safe
# side (D13).
cpp static void fs() {}\nnamespace { void an() {} }\nnamespace n { static void ns() {} void nf() {} namespace { void deep() {} } }\nvoid K::out() {}\nextern "C" void c1() {}\ntemplate <typename T> void tf(T t) {} ⇒ fs:- an:- ns:- nf:ES deep:- K::out:ES c1:ES tf:ES
# A function body closes the scope bit for what it declares (a local
# struct's method keeps its own public bit) and a lambda is no unit at
# all; a template's member is read through the wrapper.
cpp void f() { struct L { void m() {} }; auto g = [](int x) { return x; }; }\ntemplate <typename T> struct TS { void tsm() {} }; ⇒ f:ES L::m:E TS::tsm:ES
"#;

/// The same line shape over EVERY unit the register keys, spelled as
/// stored (`name/arity` for a function).
const ALL: &str = r#"
# The C register: a type specifier with a body and a typedef are units
# (`struct T *` in a signature is a reference and no row), a macro is
# exported wherever it sits, and a struct declared inside a function
# body keeps its own bit and loses the scope bit — the Rust body-local
# reading.
c struct T { int x; };\nstatic struct T *f(struct T *t) { return t; }\ntypedef struct N { int v; } N;\n#define M 1\nenum E { A };\nvoid g(void) { struct L { int z; }; } ⇒ T:ES f/1:- N:ES N:ES M:ES E:ES L:E g/0:ES
# The C++ register: a forward `class Fwd;` is no unit, a typedef and an
# alias key by their new name, a namespace by its own, a class in an
# anonymous namespace loses bit 0, and a macro inside a class body is
# no member of it.
cpp class Fwd;\nstruct Top { int x; };\ntypedef struct { int y; } Anon;\ntypedef int (*fp)(int);\nusing Alias = int;\nenum class E { A };\nnamespace ns { class C { public: void m() {} }; }\nnamespace { struct Hidden { int q; }; }\nclass B {\n#define FN(x) ((x) + 1)\n}; ⇒ Top:ES Anon:ES fp:ES Alias:ES E:ES C:ES C::m/0:ES ns:ES Hidden:- B:ES FN:ES
"#;

#[test]
fn c_family_words_follow_the_tables() {
    run_tables(FNS, ALL);
}
