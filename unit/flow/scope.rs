//! The variable table across languages (plan v2.31 step 4 A2; design
//! booklet §5.1 rules 8 and 10): where each binding lands, what a name
//! resolves to, and which names the table leaves out.

use crate::flow::lower::shape::run_table;

const TABLE: &str = r#"
py @@ back-edge @@ rule 8: a name a later statement binds is the unit's from the start, so a read the loop's back edge reaches resolves
def f(xs):
    for x in xs:
        if x:
            use(last)
        last = x
----
s 0 -1 3 0 0
s 1 0 0 0 0
s 2 1 2 0 0
s 3 2 0 0 0
s 4 3 1 0 0
s 5 1 1 0 0
v 0 -1 1 xs
v 1 0 0 x
v 2 5 0 last
x 0 0 0
x 0 1 1
x 2 1 0
x 4 2 0
x 5 1 0
x 5 2 1
====
ts @@ var-let @@ rule 8: `var` is function-wide and read before its statement, `let` block-scoped and shadowing
function f(n: number) {
  for (let i = 0; i < n; i++) {
    let n = i;
    total = n;
    var total;
  }
  return total;
}
----
s 0 -1 1 0 0
s 1 -1 3 0 0
s 2 1 0 0 0
s 3 2 1 0 0
s 4 2 1 0 0
s 5 2 1 0 0
s 6 1 14 0 0
s 7 6 1 0 0
s 8 -1 9 0 0
v 0 -1 1 n
v 1 5 0 total
v 2 0 0 i
v 3 3 0 n
x 0 2 1
x 1 2 0
x 1 0 0
x 3 2 0
x 3 3 1
x 4 3 0
x 4 1 1
x 7 2 2
x 8 1 0
====
py @@ nonlocal-lambda @@ rules 8, 10: a nonlocal name is no local; a lambda captures the host names it mentions
def f(a):
    nonlocal seen
    seen = a
    k = 2
    g = lambda x: x * k
    return g
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 1 0 0
s 4 -1 9 0 0
v 0 -1 1 a
v 1 2 2 k
v 2 3 0 g
x 1 0 0
x 2 1 1
x 3 2 1
x 4 2 0
====
rs @@ shadow @@ rule 8: a block's let shadows until the block ends; a closure's parameter is no host name
fn f(x: i32) -> i32 {
    let y = x;
    {
        let y = y + 1;
        use_it(y);
    }
    let h = |y: i32| y;
    h(y)
}
----
s 0 -1 1 0 0
s 1 -1 0 0 0
s 2 1 1 0 0
s 3 1 1 0 0
s 4 -1 1 0 0
s 5 -1 1 0 0
v 0 -1 1 x
v 1 0 0 y
v 2 2 0 y
v 3 4 0 h
x 0 0 0
x 0 1 1
x 2 1 0
x 2 2 1
x 3 2 0
x 4 3 1
x 5 3 0
x 5 1 0
====
go @@ nested-redeclare @@ rule 8: `:=` in an inner block declares a new name even when an outer one is spelled alike
func f() int {
	n := 1
	if n > 0 {
		n := 2
		use(n)
	}
	return n
}
----
s 0 -1 1 0 0
s 1 -1 2 0 0
s 2 1 0 0 0
s 3 2 1 0 0
s 4 2 1 0 0
s 5 -1 9 0 0
v 0 0 0 n
v 1 3 0 n
x 0 0 1
x 1 0 0
x 3 1 1
x 4 1 0
x 5 0 0
"#;

#[test]
fn the_scopes_resolve_as_the_table_says() {
    run_table(TABLE);
}
