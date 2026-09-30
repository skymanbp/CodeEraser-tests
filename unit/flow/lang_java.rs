//! Java lowered as the table says (plan v2.31 step 4 A2): each block one
//! rule of design booklet §5.1 on the Java grammar, its rows the first
//! unit's statements, variables and accesses (unit/flow/shape.rs).

use crate::flow::lower::shape::run_table;

const TABLE: &str = r#"
java @@ switch @@ rule 6: colon groups fall through and a `default` label gives has_else; arrow rules do not fall through
int f(int k) {
    int r = 0;
    switch (k) {
        case 1:
        case 2:
            r = 1;
            break;
        default:
            r = 2;
    }
    switch (k) {
        case 3 -> r += 3;
        default -> { return r; }
    }
    return r;
}
----
s 0 -1 1 0 0
s 1 -1 4 1 0
s 2 1 5 4 0
s 3 1 5 4 0
s 4 3 1 0 0
s 5 3 11 0 1
s 6 1 5 4 0
s 7 6 1 0 0
s 8 -1 4 1 0
s 9 8 5 0 0
s 10 9 1 0 0
s 11 8 5 0 0
s 12 11 0 0 0
s 13 12 9 0 0
s 14 -1 9 0 0
v 0 -1 1 k
v 1 0 0 r
x 0 1 1
x 1 0 0
x 4 1 1
x 7 1 1
x 8 0 0
x 10 1 2
x 13 1 0
x 14 1 0
====
java @@ try-resources @@ rules 2, 3, 8: a resource is declared and read in the try's head; catch and finally; System.exit never returns; throw
void f(String path) throws IOException {
    try (Reader in = open(path)) {
        use(in);
    } catch (IOException e) {
        System.exit(1);
    } finally {
        done();
    }
    throw new IllegalStateException();
}
----
s 0 -1 6 0 0
s 1 0 0 0 0
s 2 1 1 0 0
s 3 0 7 0 0
s 4 3 0 0 0
s 5 4 15 0 0
s 6 0 8 0 0
s 7 6 0 0 0
s 8 7 1 0 0
s 9 -1 10 0 0
v 0 -1 1 path
v 1 0 0 in
v 2 3 0 e
x 0 0 0
x 0 1 1
x 2 1 0
x 3 2 1
====
java @@ instanceof @@ rule 8: an `instanceof` pattern variable binds in the statement's scope, where flow scoping reaches past the if
int f(Object o) {
    if (!(o instanceof String s)) {
        return 0;
    }
    return s.length();
}
----
s 0 -1 2 0 0
s 1 0 0 0 0
s 2 1 9 0 0
s 3 -1 9 0 0
v 0 -1 1 o
v 1 0 0 s
x 0 0 0
x 0 1 1
x 3 1 0
====
java @@ labels-for @@ rules 4, 8: an enhanced for declares; a C-style for with two inits and two updates; a labelled continue and break reach the outer loop
int f(int[][] grid) {
    int n = 0;
    outer:
    for (int[] row : grid) {
        for (int i = 0, j = 1; i < row.length; i++, j++) {
            if (row[i] < 0) continue outer;
            if (row[i] > 9) break outer;
            n += row[i] * j;
        }
    }
    return n;
}
----
s 0 -1 1 0 0
s 1 -1 14 0 0
s 2 1 3 0 0
s 3 2 0 0 0
s 4 3 1 0 0
s 5 3 3 0 0
s 6 5 0 0 0
s 7 6 2 0 0
s 8 7 12 0 2
s 9 6 2 0 0
s 10 9 11 0 2
s 11 6 1 0 0
s 12 5 14 0 0
s 13 12 1 0 0
s 14 12 1 0 0
s 15 -1 9 0 0
v 0 -1 1 grid
v 1 0 0 n
v 2 2 0 row
v 3 4 0 i
v 4 4 0 j
x 0 1 1
x 2 0 0
x 2 2 1
x 4 3 1
x 4 4 1
x 5 3 0
x 5 2 0
x 7 2 0
x 7 3 0
x 9 2 0
x 9 3 0
x 11 2 0
x 11 3 0
x 11 4 0
x 11 1 2
x 13 3 2
x 14 4 2
x 15 1 0
====
java @@ captures @@ rules 9, 10: a lambda and an anonymous class capture the host names they mention; a field write through `this` writes no local
void f(List<Integer> xs) {
    int base = 1;
    int count = 0;
    xs.forEach(x -> use(x + base));
    Runnable r = new Runnable() { public void run() { use(count); } };
    this.total = count;
    r.run();
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 1 0 0
s 4 -1 1 0 0
s 5 -1 1 0 0
v 0 -1 1 xs
v 1 0 2 base
v 2 1 2 count
v 3 3 0 r
x 0 1 1
x 1 2 1
x 2 0 0
x 3 3 1
x 4 2 0
x 5 3 0
====
java @@ loops-sync @@ rules 5, 9: do-while; `synchronized` is a block with a head; a write in a ternary branch is a read-write; `while (true)` infinite
int f(int a) {
    int x = 0;
    do {
        a--;
    } while (a > 0);
    synchronized (this) {
        x = a > 1 ? (a = 2) : 3;
    }
    while (true) {
        if (x > 5) break;
        x++;
    }
    return x;
}
----
s 0 -1 1 0 0
s 1 -1 3 0 0
s 2 1 0 0 0
s 3 2 1 0 0
s 4 -1 0 0 0
s 5 4 0 0 0
s 6 5 1 0 0
s 7 -1 3 2 0
s 8 7 0 0 0
s 9 8 2 0 0
s 10 9 11 0 7
s 11 8 1 0 0
s 12 -1 9 0 0
v 0 -1 1 a
v 1 0 0 x
x 0 1 1
x 1 0 0
x 3 0 2
x 6 0 0
x 6 0 2
x 6 1 1
x 9 1 0
x 11 1 2
x 12 1 0
====
java @@ instanceof-elif @@ rule 8: a pattern binder in an elif condition after a nested then branch lands in the if chain's enclosing scope
int f(boolean a, boolean b, boolean c, Object o) {
    if (a) { if (b) { if (c) { x(); } } } else if (o instanceof T t) { return t.hashCode(); }
    return 0;
}
----
s 0 -1 2 1 0
s 1 0 0 0 0
s 2 1 2 0 0
s 3 2 0 0 0
s 4 3 2 0 0
s 5 4 0 0 0
s 6 5 1 0 0
s 7 0 2 0 0
s 8 7 0 0 0
s 9 8 9 0 0
s 10 -1 9 0 0
v 0 -1 1 a
v 1 -1 1 b
v 2 -1 1 c
v 3 -1 1 o
v 4 7 0 t
x 0 0 0
x 2 1 0
x 4 2 0
x 7 3 0
x 7 4 1
x 9 4 0
====
java @@ instanceof-do-while @@ rule 8: a pattern binder in a do-while condition, walked after the body's nested statements, lands in the loop's enclosing scope
int f(boolean a, boolean b, Object o) {
    do { if (a) { if (b) { x(); } } } while (o instanceof T t && t.hashCode() > 0);
    return 0;
}
----
s 0 -1 3 0 0
s 1 0 0 0 0
s 2 1 2 0 0
s 3 2 0 0 0
s 4 3 2 0 0
s 5 4 0 0 0
s 6 5 1 0 0
s 7 -1 9 0 0
v 0 -1 1 a
v 1 -1 1 b
v 2 -1 1 o
v 3 0 0 t
x 0 2 0
x 0 3 1
x 0 3 0
x 2 0 0
x 4 1 0
"#;

#[test]
fn java_lowers_as_the_table_says() {
    run_table(TABLE);
}
