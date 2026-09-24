//! Plan v2.30 step 3 Java metric battery: the launch-language rows of
//! metrics.rs re-spelled in Java where the construct exists, plus one
//! row per stance the Java table records (spec_java.rs, the register
//! in contracts/fixtures/crosscheck/DIVERGENCES.md) — each why cites
//! the whitepaper page or the register entry. The whitepaper's own
//! Java examples are sonar_whitepaper_java.rs; this table holds what
//! they do not show: the shapes Java spells its own way. A bare method
//! is a whole Java source file to tree-sitter-java, so no row needs a
//! class around it. The table's last block, a `units=` row, pins what
//! the Java table makes a unit and what it reads as each unit's
//! parameters.

use crate::common;

const ROWS: &str = r#"
java @@ cc=5 coc=7 nesting=3 lines=10 @@ metrics.rs's Python row in Java: cc 1 + if + && + for + if; coc if +1, && +1, for +2, if +3; nesting if > for > if
int f(int a, int b) {
    if (a > 0 && b > 0) {
        for (int i = 0; i < 10; i++) {
            if (i > 5) {
                return i;
            }
        }
    }
    return 0;
}
====
java @@ cc=5 coc=5 nesting=1 lines=9 @@ metrics.rs's TypeScript row in Java: `else if` is a flat hybrid (p.7) and the trailing else pays +1 without nesting
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
java @@ cc=2 coc=2 nesting=1 lines=4 @@ Java's `else` has no node — the alternative IS the else body — so a single statement there pays the else +1 exactly as a block does
int s(int a) {
    if (a > 0) return 1;
    else return 0;
}
====
java @@ cc=3 coc=4 nesting=2 lines=8 @@ an if inside an else BLOCK is a nested if, not an else-if: the block is the alternative, so its if sits one level down (+2)
int n(int a, int b) {
    if (a > 0) {
        return 1;
    } else {
        if (b > 0) return 2;
    }
    return 0;
}
====
java @@ cc=3 coc=1 nesting=1 lines=6 @@ one switch_expression spells the arrow switch too: +1 and the labels are free; a label listing two values is one path, `default` another (register D2)
String w(int n) {
    return switch (n) {
        case 1, 2 -> "few";
        default -> "lots";
    };
}
====
java @@ cc=3 coc=3 nesting=2 lines=1 @@ the ternary nests like TypeScript's: outer +1, inner +2 (register D4)
int t(int a, int b) { return a > 0 ? (b > 0 ? 1 : 2) : 3; }
====
java @@ cc=5 coc=10 nesting=3 lines=10 @@ a break to a label is a fundamental +1 (p.8; register D5) and no branch, while a plain `break` names no label and pays nothing
int j(int[][] m) {
    outer:
    for (int[] row : m) {
        for (int v : row) {
            if (v < 0) break outer;
            if (v == 0) break;
        }
    }
    return 0;
}
====
java @@ cc=4 coc=3 nesting=1 lines=7 @@ an enhanced for, a while and a do-while are structures like any loop, none nesting another
int l(int n) {
    int s = 0;
    for (int x : new int[] {1, 2, 3}) { s += x; }
    while (s > 0) { s--; }
    do { s++; } while (s < 3);
    return s;
}
====
java @@ cc=2 coc=1 nesting=1 lines=10 @@ try-with-resources, finally and synchronized are no structures: the if inside all three pays +1 at level 0 (p.9 myMethod, try's reading)
int r(java.io.Reader in) throws Exception {
    try (java.io.BufferedReader b = new java.io.BufferedReader(in)) {
        synchronized (this) {
            if (b.ready()) return 1;
        }
    } finally {
        in.close();
    }
    return 0;
}
====
java @@ cc=3 coc=3 nesting=2 lines=4 @@ an expression lambda absorbs into its host like a block one (register D3): its ternary pays +2 one level down, and the operator run still pays +1
int x(java.util.List<Integer> v) {
    v.replaceAll(e -> e > 0 && e < 9 ? e : 0);
    return v.size();
}
====
java @@ units=K/0, K/1, m/0, s/2, r/1, b/0, d/0, e/0, f/0, g/0, R/2, z/0, anon/0, toString/0 @@ what the Java table makes a unit and what it reads as its parameters (the probe transcripts, booklet §4): a constructor and a record's compact constructor are units named by their class; a method without a body (abstract, interface, annotation element) is none, while a default or static interface method and an enum constant's method are; an anonymous class's method is a unit beside its host. A varargs parameter counts once; a receiver parameter (`K this`) is no formal parameter (JLS 8.4.1); a compact constructor's parameters are the record header's components, which it declares implicitly (JLS 8.10.4.2)
class K {
    K() {}
    K(int a) { this(); }
    void m() {}
    static int s(int a, String... rest) { return a; }
    void r(K this, int x) {}
    abstract static class Inner { abstract void a(); void b() {} }
    interface I { void c(); default void d() {} static void e() {} }
    enum E { A { void f() {} }; void g() {} }
    record R(int x, int y) { R { } int z() { return x; } }
    Object anon() { return new Object() { public String toString() { return ""; } }; }
    @interface Ann { int v() default 0; }
}
"#;

#[test]
fn java_metrics_follow_the_table() {
    common::assert_metric_table(ROWS);
}
