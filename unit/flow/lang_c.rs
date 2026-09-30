//! C lowered as the table says (plan v2.31 step 4 A2): each block one
//! rule of design booklet §5.1 on the C grammar, its rows the first
//! unit's statements, variables and accesses (unit/flow/shape.rs).

use crate::flow::lower::shape::run_table;

const TABLE: &str = r#"
c @@ for-switch @@ rules 4, 5, 6: a C-style for with an update label, a switch whose cases all fall through and whose default has no value, `for (;;)` infinite
int f(int n) {
    int s = 0, i;
    for (i = 0; i < n; i++) {
        switch (i % 3) {
        case 0:
            s += i;
            break;
        case 1:
            continue;
        default:
            s -= 1;
        }
    }
    for (;;) {
        if (s > 100) return s;
        s++;
    }
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 3 0 0
s 3 2 0 0 0
s 4 3 4 1 0
s 5 4 5 4 0
s 6 5 1 0 0
s 7 5 11 0 4
s 8 4 5 4 0
s 9 8 13 0 12
s 10 4 5 4 0
s 11 10 1 0 0
s 12 2 14 0 0
s 13 12 1 0 0
s 14 -1 3 2 0
s 15 14 0 0 0
s 16 15 2 0 0
s 17 16 9 0 0
s 18 15 1 0 0
v 0 -1 1 n
v 1 0 0 s
v 2 0 0 i
x 0 1 1
x 1 2 1
x 2 2 0
x 2 0 0
x 4 2 0
x 6 2 0
x 6 1 2
x 11 1 2
x 13 2 2
x 16 1 0
x 17 1 0
x 18 1 2
====
c @@ goto-static @@ rules 3, 8, 10 and ruling 7: a goto reaches its label; a static local and a prototype are no locals, a function pointer is; `&x` takes the address; exit never returns
int f(int a) {
    static int calls = 0;
    int g(int);
    int (*fp)(int) = g;
    int x;
    calls++;
    if (a < 0) goto fail;
    x = fp(a);
    read_into(&x);
    return x;
fail:
    exit(1);
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 1 0 0
s 4 -1 1 0 0
s 5 -1 2 0 0
s 6 5 13 0 10
s 7 -1 1 0 0
s 8 -1 1 0 0
s 9 -1 9 0 0
s 10 -1 14 0 0
s 11 10 15 0 0
v 0 -1 1 a
v 1 2 0 fp
v 2 3 8 x
x 2 1 1
x 5 0 0
x 7 1 0
x 7 0 0
x 7 2 1
x 8 2 0
x 9 2 0
====
c @@ preproc @@ rule 7: a preprocessor condition in the body makes the unit dynamic
int f(int a) {
    int r = a;
#ifdef FAST
    r = a * 2;
#endif
    return r;
}
----
s 0 -1 1 0 0
s 1 -1 1 8 0
s 2 -1 9 0 0
v 0 -1 1 a
v 1 0 0 r
x 0 0 0
x 0 1 1
x 1 0 0
x 1 1 1
x 2 1 0
====
c @@ do-while @@ rules 3, 5, 9: do-while, `while (1)` infinite, an index or deref write reads its base, a `_Noreturn` declaration's name never returns
_Noreturn void die(const char *m);
int f(int *p, int n) {
    int buf[4];
    buf[0] = n;
    *p = buf[0];
    do {
        n--;
    } while (n > 0);
    do {
        if (n) break;
    } while (1);
    die("x");
    return n;
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 3 0 0
s 4 3 0 0 0
s 5 4 1 0 0
s 6 -1 3 2 0
s 7 6 0 0 0
s 8 7 2 0 0
s 9 8 11 0 6
s 10 -1 15 0 0
s 11 -1 9 0 0
v 0 -1 1 p
v 1 -1 1 n
v 2 0 0 buf
x 1 1 0
x 1 2 0
x 2 2 0
x 2 0 0
x 3 1 0
x 5 1 2
x 8 1 0
x 11 1 0
====
c @@ conditional @@ rule 9: a write in a conditional branch or an `&&` right operand is a read-write; a member write reads its base
void f(int c, struct S *s) {
    int x = 0, y;
    y = c ? (x = 1) : 2;
    c && (x = 3);
    s->v = x;
    s->w = y;
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 1 0 0
s 4 -1 1 0 0
v 0 -1 1 c
v 1 -1 1 s
v 2 0 0 x
v 3 0 0 y
x 0 2 1
x 1 0 0
x 1 2 2
x 1 3 1
x 2 0 0
x 2 2 2
x 3 2 0
x 3 1 0
x 4 3 0
x 4 1 0
====
c @@ blocks @@ rules 1, 8: a nested block shadows, an empty statement is a statement
int f(int n) {
    int v = 1;
    {
        int v = 2;
        n += v;
    }
    while (n-- > 0)
        ;
    return v + n;
}
----
s 0 -1 1 0 0
s 1 -1 0 0 0
s 2 1 1 0 0
s 3 1 1 0 0
s 4 -1 3 0 0
s 5 4 1 0 0
s 6 -1 9 0 0
v 0 -1 1 n
v 1 0 0 v
v 2 2 0 v
x 0 1 1
x 2 2 1
x 3 2 0
x 3 0 2
x 4 0 2
x 6 1 0
x 6 0 0
"#;

#[test]
fn c_lowers_as_the_table_says() {
    run_table(TABLE);
}
