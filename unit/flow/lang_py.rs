//! Python lowered as the table says (plan v2.31 step 4 A2): each block
//! one rule of design booklet §5.1 on the Python grammar, its rows the
//! unit's statements, variables and accesses (unit/flow/shape.rs).

use crate::flow::lower::shape::run_table;

const TABLE: &str = r#"
py @@ elif @@ rule 2: an elif chain is an if in the else position, each branch one child
def f(x):
    if x > 0:
        y = 1
    elif x < 0:
        y = 2
    else:
        y = 3
    return y
----
s 0 -1 2 1 0
s 1 0 0 0 0
s 2 1 1 0 0
s 3 0 2 1 0
s 4 3 0 0 0
s 5 4 1 0 0
s 6 3 0 0 0
s 7 6 1 0 0
s 8 -1 9 0 0
v 0 -1 1 x
v 1 2 0 y
x 0 0 0
x 2 1 1
x 3 0 0
x 5 1 1
x 7 1 1
x 8 1 0
====
py @@ for-else @@ rule 4: a loop with an else is loop · else block · label, its own break a goto to the label
def f(xs):
    for x in xs:
        if x:
            break
        continue
    else:
        x = 0
    return x
----
s 0 -1 3 0 0
s 1 0 0 0 0
s 2 1 2 0 0
s 3 2 0 0 0
s 4 3 13 0 8
s 5 1 12 0 0
s 6 -1 0 0 0
s 7 6 1 0 0
s 8 -1 14 0 0
s 9 -1 9 0 0
v 0 -1 1 xs
v 1 0 0 x
x 0 0 0
x 0 1 1
x 2 1 0
x 7 1 1
x 9 1 0
====
py @@ while-true-try @@ rules 2, 3, 5: `while True` is infinite; try is body · else · except · finally
def f(a):
    while True:
        try:
            a = g(a)
        except ValueError as err:
            log(err)
            raise
        else:
            break
        finally:
            done()
    sys.exit(1)
    a = 2
----
s 0 -1 3 2 0
s 1 0 0 0 0
s 2 1 6 0 0
s 3 2 0 0 0
s 4 3 1 0 0
s 5 2 0 0 0
s 6 5 11 0 0
s 7 2 7 0 0
s 8 7 0 0 0
s 9 8 1 0 0
s 10 8 10 0 0
s 11 2 8 0 0
s 12 11 0 0 0
s 13 12 1 0 0
s 14 -1 15 0 0
s 15 -1 1 0 0
v 0 -1 1 a
v 1 7 0 err
x 4 0 0
x 4 0 1
x 7 1 1
x 9 1 0
x 15 0 1
====
py @@ with-global @@ ruling 5 and rule 8: with is a try with an empty catch, its target bound; a global name is no local
def f(path):
    global cache
    with open(path) as fh:
        data = fh.read()
    cache = data
    return 1
----
s 0 -1 1 0 0
s 1 -1 6 0 0
s 2 1 0 0 0
s 3 2 1 0 0
s 4 1 7 16 0
s 5 -1 1 0 0
s 6 -1 9 0 0
v 0 -1 1 path
v 1 1 0 fh
v 2 3 0 data
x 1 0 0
x 1 1 1
x 3 1 0
x 3 2 1
x 5 2 0
====
py @@ match @@ rule 6: match has an arm per case, `case _:` its default; patterns bind, a guard reads
def f(cmd):
    match cmd:
        case [op, arg] if arg:
            r = op
        case _:
            r = None
    return r
----
s 0 -1 4 1 0
s 1 0 5 0 0
s 2 1 0 0 0
s 3 2 1 0 0
s 4 0 5 0 0
s 5 4 0 0 0
s 6 5 1 0 0
s 7 -1 9 0 0
v 0 -1 1 cmd
v 1 1 0 op
v 2 1 0 arg
v 3 3 0 r
x 0 0 0
x 1 1 1
x 1 2 1
x 1 2 0
x 3 1 0
x 3 3 1
x 6 3 1
x 7 3 0
====
py @@ scopes @@ rules 8, 9, 10: self ignored, defaults read first, a comprehension's names are not the unit's, a nested def captures
def m(self, n=K, *rest):
    total = 0
    ys = [v * n for v in rest]
    def inner():
        return total
    return inner, ys
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 1 0 0
s 4 -1 9 0 0
v 0 -1 5 self
v 1 -1 1 n
v 2 -1 1 rest
v 3 1 2 total
v 4 2 0 ys
x 1 3 1
x 2 1 0
x 2 2 0
x 2 4 1
x 4 4 0
====
py @@ conditional @@ rule 9: walrus and a write in `and` / the else of a conditional are read-writes; `+=` is one; eval is dynamic
def f(a, b):
    x = 0
    ok = a and (x := b)
    x += 1
    y = x if ok else (x := 2)
    eval("x")
    return y
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 1 0 0
s 4 -1 1 8 0
s 5 -1 9 0 0
v 0 -1 1 a
v 1 -1 1 b
v 2 0 0 x
v 3 1 0 ok
v 4 3 0 y
x 0 2 1
x 1 0 0
x 1 1 0
x 1 2 2
x 1 3 1
x 2 2 2
x 3 2 0
x 3 3 0
x 3 2 2
x 3 4 1
x 5 4 0
"#;

#[test]
fn python_lowers_as_the_table_says() {
    run_table(TABLE);
}
