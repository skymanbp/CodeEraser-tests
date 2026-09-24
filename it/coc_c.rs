//! Plan v2.30 step 2 C / C++ metric battery: the launch-language rows
//! of metrics.rs re-spelled in C where the construct exists, plus one
//! row per stance the C-family table records against lizard
//! (contracts/fixtures/crosscheck/DIVERGENCES.md, C / C++ section) —
//! each why cites the whitepaper page or the register entry. One
//! block per row, blocks separated by a `====` line: a header
//! `ext @@ cc=… coc=… nesting=… lines=… @@ why` over the source itself
//! (common::assert_metric_table). One literal rather than a slice of
//! typed rows — a second MetricCase table is the clone shape the write
//! gate refuses, and rows of any one tuple shape repeat every dozen
//! tokens, so the clone gate read this table as clones of itself in
//! the array form. Naming and arity come off the declarator chain
//! (scan/declarator.rs), so the table's last block, a `units=` row,
//! pins the spelling and the parameter count the chain produces for
//! every declarator shape the probe transcripts hold.

use crate::common;
use codeeraser::scan::lang::Lang;

const ROWS: &str = r#"
c @@ cc=5 coc=7 nesting=3 lines=10 @@ metrics.rs's Python row in C: cc 1 + if + && + for + if; coc if +1, && +1, for +2, if +3; nesting if > for > if
int f(int a, int b) {
    if (a && b) {
        for (int i = 0; i < 10; i++) {
            if (i > 5) {
                return i;
            }
        }
    }
    return 0;
}
====
c @@ cc=5 coc=5 nesting=1 lines=9 @@ metrics.rs's TypeScript row in C: `else if` is a flat hybrid (p.7) and the trailing else pays +1 without nesting
int g(int a, int b) {
    if (a > 0 && b > 0) {
        return a + b;
    } else if (a > 0 || b > 0) {
        return 1;
    } else {
        return 0;
    }
}
====
c @@ cc=5 coc=1 nesting=1 lines=8 @@ p.5 + p.10 getWords: switch +1 and the cases are free; every case incl. `default:` is a cyclomatic path (register D2 — lizard counts the `case` keyword only)
const char *words(int n) {
    switch (n) {
        case 1: return "one";
        case 2: return "a couple";
        case 3: return "a few";
        default: return "lots";
    }
}
====
c @@ cc=3 coc=3 nesting=2 lines=1 @@ the ternary nests like TypeScript's: outer +1, inner +2 (register D4)
int t(int a, int b) { return a ? (b ? 1 : 2) : 3; }
====
c @@ cc=2 coc=2 nesting=1 lines=6 @@ a labeled jump is a fundamental +1 (p.8; register D5) and no branch
int j(int n) {
    if (n < 0) goto fail;
    return n;
fail:
    return -1;
}
====
c @@ cc=1 coc=0 nesting=0 lines=7 @@ the preprocessor is invisible to both metrics (register D1 — lizard counts #ifdef / #elif as branches)
int p(int n) {
#ifdef FAST
    return n;
#else
    return n + 1;
#endif
}
====
c @@ cc=4 coc=3 nesting=0 lines=1 @@ p.8 operator runs: &&, ||, && are three runs and three branches
int r(int a, int b, int c, int d) { return a && b || c && d; }
====
c @@ cc=1 coc=0 nesting=0 lines=9 @@ the operators inside a `#if` / `#elif` condition are compile-time text, not branches (register D1 — fmt's is_big_endian read 2 for the `&&` in its `#elif` before the opaque field existed)
int q(int a) {
#if defined(X) && defined(Y)
    return a;
#elif Z || W
    return 1;
#else
    return 0;
#endif
}
====
cpp @@ cc=4 coc=3 nesting=1 lines=7 @@ range-for, while and do-while are structures like any loop, none nesting another
int w(int n) {
    int s = 0;
    for (int x : {1, 2, 3}) { s += x; }
    while (s > 0) { s--; }
    do { s++; } while (s < 3);
    return s;
}
====
cpp @@ cc=2 coc=2 nesting=2 lines=4 @@ p.9 myMethod2: a lambda absorbs into its host (one unit, its if counted there), increments nothing and raises nesting — the lambda body is level 1, so the if inside it sits at level 2 (register D3)
int l(int v) {
    auto f = [&](int x) { if (x > 0) return x; return 0; };
    return f(v);
}
====
cpp @@ cc=5 coc=4 nesting=1 lines=8 @@ catch is a structure; `and` / `or` are the alternative tokens for && / || (register D19) and form two runs; try nests nothing
int c(int a, bool b) {
    try {
        if (a > 0 and b or a < -5) return 1;
    } catch (...) {
        return -1;
    }
    return 0;
}
====
cpp @@ units=K::~K/0, K::operator==/1, K::operator bool/0, K::b/0, In::m/0, ns::In::out/0, K::K/0, maker/1, hidden/0, refret/1, trailing/1, spec/1, Box::b/0, local/0, L::m/0, one/1 @@ every declarator shape the probe transcripts hold, with the name the chain spells for it and the arity it reads: in-body members carry their class chain, out-of-class definitions carry the qualifier they spell (the two meet in owner_of), a namespace is not part of the chain, a local struct's method is a unit of its own, the innermost function_declarator owns the parameter list, `(void)` is an empty list (register D12) where `void *p` is one, a conversion operator names its type, and template arguments are not part of a name — `spec<int>` and `Box<T>::b` spell `spec` and `Box::b` (the crosscheck read a partial specialization's three-line argument list into a member's name before the rule existed)
struct K { K(); ~K() {} bool operator==(const K&) const { return true; } operator bool() const { return true; } };
void K::b() {}
namespace ns { struct In { void m() {} }; }
void ns::In::out() {}
K::K() {}
int (*maker(int n))(int) { return 0; }
static inline int hidden(void) { return 0; }
int& refret(int& a) { return a; }
auto trailing(int a) -> int { return a; }
template <> void spec<int>(int) {}
template <typename T> void Box<T>::b() {}
void local() { struct L { void m() {} }; }
int one(void *p) { return p != 0; }
"#;

#[test]
fn c_family_metrics_follow_the_table() {
    common::assert_metric_table(ROWS);
}

/// What the definition kind admits that is no definition (the
/// fmt/format.h transcript, crosscheck 2026-09-24): `= default` /
/// `= delete` have no body and `= 0` is a field; an unexpanded macro
/// alone on a line before `namespace detail {` reads as a definition
/// named `namespace` with no parameter list; a macro block inside a
/// body (`FMT_CATCH(...) { }`, a GNU nested function in C) reads as a
/// nested definition and is absorbed into its host — its `if` counts
/// there, flat — while a local class's member stays a unit of its own;
/// the same macro block at class scope has no type and is no
/// constructor, destructor or conversion, the three that go without.
#[test]
fn c_family_definitions_need_a_body_a_list_a_name_and_no_host() {
    const SHAPES: &[(&str, &str)] = &[
        (
            "struct K { K() = default; K(const K&) = delete; virtual void v() = 0; void real() {} };\n",
            "K::real",
        ),
        (
            "FMT_BEGIN_NAMESPACE\nnamespace detail { void f() {} }\nFMT_END_NAMESPACE\n",
            "f",
        ),
        (
            "int h(int a) {\n  MACRO_CATCH(const int&) { if (a) return 1; }\n  struct L { void m() {} };\n  return 0;\n}\n",
            "h L::m",
        ),
        (
            "struct F { auto format() const -> int { return 0; } FMT_CATCH(const int&) { } };\n",
            "F::format",
        ),
        (
            "struct K { K() {} explicit K(int) {} ~K() {} operator int() const { return 0; } };\n\
             K::K(long) {}\ntemplate <typename T> struct Box { Box() {} };\n\
             template <typename T> Box<T>::Box(int) {}\n",
            "K::K K::K K::~K K::operator int K::K Box::Box Box::Box",
        ),
        // a macro before a destructor reads as its return type and the
        // parser drops the `~` into an ERROR child: the tilde goes back
        ("struct K { FMT_CONSTEXPR20 ~K() { } };\n", "K::~K"),
    ];
    for (src, want) in SHAPES {
        let units = common::measure_units(Lang::Cpp, src);
        let names: Vec<&str> = units.iter().map(|u| u.name.as_str()).collect();
        assert_eq!(names.join(" "), *want, "units of:\n{src}");
    }
    let host = &common::measure_units(Lang::Cpp, SHAPES[2].0)[0];
    assert_eq!(
        (host.cc, host.coc, host.nesting),
        (2, 1, 1),
        "the absorbed block's if counts in its host, flat"
    );
    let nested = common::measure_units(
        Lang::C,
        "int host(int a) {\n  int nested(int b) { return b > 0 ? b : -b; }\n  return nested(a);\n}\n",
    );
    assert_eq!(nested.len(), 1, "a GNU nested function is absorbed");
    assert_eq!((nested[0].name.as_str(), nested[0].cc), ("host", 2));
}
