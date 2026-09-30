//! C++ lowered as the table says (plan v2.31 step 4 A2): each block one
//! rule of design booklet §5.1 on the C++ grammar, its rows the first
//! unit's statements, variables and accesses (unit/flow/shape.rs).

use crate::flow::lower::shape::run_table;

const TABLE: &str = r#"
cpp @@ try-lambda @@ rules 2, 8, 10: try · catch, throw; a lambda captures the host names it mentions; a reference binds both sides address-taken, a range-for reference too
int f(std::vector<int>& v) {
    int total = 0;
    int& ref = total;
    auto add = [&](int x) { total += x; };
    for (auto& e : v) {
        e = 0;
    }
    try {
        add(1);
        throw std::runtime_error("x");
    } catch (const std::exception& err) {
        log(err);
    }
    ref = 2;
    return total;
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 3 0 0
s 4 3 0 0 0
s 5 4 1 0 0
s 6 -1 6 0 0
s 7 6 0 0 0
s 8 7 1 0 0
s 9 7 10 0 0
s 10 6 7 0 0
s 11 10 0 0 0
s 12 11 1 0 0
s 13 -1 1 0 0
s 14 -1 9 0 0
v 0 -1 9 v
v 1 0 10 total
v 2 1 8 ref
v 3 2 0 add
v 4 3 8 e
v 5 10 8 err
x 0 1 1
x 1 1 0
x 1 2 1
x 2 3 1
x 3 0 0
x 3 4 1
x 5 4 1
x 8 3 0
x 10 5 1
x 12 5 0
x 13 2 1
x 14 1 0
====
cpp @@ if-init @@ rules 8, 9: an if's init runs in its head; a structured binding binds each name; `or` guards a write; std::exit never returns
int f(std::map<int, int>& m) {
    if (auto it = m.find(1); it != m.end()) {
        auto [k, val] = *it;
        return k + val;
    }
    bool ok = false;
    m.empty() or (ok = true);
    if (!ok) std::exit(2);
    return 0;
}
----
s 0 -1 2 0 0
s 1 0 0 0 0
s 2 1 1 0 0
s 3 1 9 0 0
s 4 -1 1 0 0
s 5 -1 1 0 0
s 6 -1 2 0 0
s 7 6 15 0 0
s 8 -1 9 0 0
v 0 -1 9 m
v 1 0 0 it
v 2 2 0 k
v 3 2 0 val
v 4 4 0 ok
x 0 0 0
x 0 1 1
x 0 1 0
x 0 0 0
x 2 1 0
x 2 2 1
x 2 3 1
x 3 2 0
x 3 3 0
x 4 4 1
x 5 0 0
x 5 4 2
x 6 4 0
====
cpp @@ defaults @@ rules 3, 9: a default argument is read in the unit's first, synthetic statement; a `[[noreturn]]` declaration's name never returns
[[noreturn]] void fail();
int g(int a, int b = a + 1) {
    if (a) fail();
    return b;
}
----
s 0 -1 1 0 0
s 1 -1 2 0 0
s 2 1 15 0 0
s 3 -1 9 0 0
v 0 -1 1 a
v 1 -1 1 b
x 0 0 0
x 1 0 0
x 3 1 0
====
cpp @@ switch-while @@ rules 5, 6, 9: `while (true)` infinite; a switch without default has no has_else; a member write through `this` writes no local
void K::run(int n) {
    while (true) {
        switch (n) {
        case 1:
            this->count = n;
            break;
        case 2:
            return;
        }
        n--;
    }
}
----
s 0 -1 3 2 0
s 1 0 0 0 0
s 2 1 4 0 0
s 3 2 5 4 0
s 4 3 1 0 0
s 5 3 11 0 2
s 6 2 5 4 0
s 7 6 9 0 0
s 8 1 1 0 0
v 0 -1 1 n
x 2 0 0
x 4 0 0
x 8 0 2
====
cpp @@ condition-decl @@ rules 1, 8: a local struct's method is its own unit; a condition declaration declares in the loop
int f(int n) {
    struct L { int m() { return 1; } };
    while (int k = next(n)) {
        n -= k;
    }
    return n;
}
----
s 0 -1 1 0 0
s 1 -1 3 0 0
s 2 1 0 0 0
s 3 2 1 0 0
s 4 -1 9 0 0
v 0 -1 1 n
v 1 1 0 k
x 1 0 0
x 1 1 1
x 3 1 0
x 3 0 2
x 4 0 0
====
cpp @@ preproc-extern @@ rules 3, 7, 8: an extern local is no local; `#if` in the body is dynamic; std::abort never returns
int f(int n) {
    extern int shared;
    int x = n;
#if DEBUG
    x = 0;
#endif
    std::abort();
    return x + shared;
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 8 0
s 3 -1 15 0 0
s 4 -1 9 0 0
v 0 -1 1 n
v 1 1 0 x
x 1 0 0
x 1 1 1
x 2 1 1
x 4 1 0
"#;

#[test]
fn cpp_lowers_as_the_table_says() {
    run_table(TABLE);
}
