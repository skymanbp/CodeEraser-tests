//! The accesses across languages (plan v2.31 step 4 A2; design booklet
//! §5.1 rule 9): evaluation order within a statement, the targets that
//! read rather than write, and the names that are no variables.

use crate::flow::lower::shape::run_table;

const TABLE: &str = r#"
py @@ order @@ rule 9: the value is read before the target is written; a subscript write reads its base and index; `del` reads
def f(a, i):
    a = a + 1
    a[i] = a[i] * 2
    del a
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
v 0 -1 1 a
v 1 -1 1 i
x 0 0 0
x 0 0 1
x 1 0 0
x 1 1 0
x 1 0 0
x 1 1 0
x 2 0 0
====
c @@ deref @@ rule 9: `*p++` is a read-write of p; a field write through a pointer reads the pointer
void f(int *p, struct S *s) {
    *p++ = 0;
    s->n += 1;
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
v 0 -1 1 p
v 1 -1 1 s
x 0 0 2
x 1 1 0
====
rs @@ braces @@ rule 9: `{{x}}` in a format string is a literal, not a read; `{y:?}` is a read
fn f(x: i32, y: i32) {
    println!("{{x}} {y:?}");
}
----
s 0 -1 1 0 0
v 0 -1 1 x
v 1 -1 1 y
x 0 1 0
====
ts @@ shorthand @@ rule 9: an object pattern in an assignment writes each shorthand name; a property key is no name
function f(obj: O) {
  let a = 0;
  ({ a } = obj);
  return { a: a };
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 9 0 0
v 0 -1 1 obj
v 1 0 0 a
x 0 1 1
x 1 0 0
x 1 1 1
x 2 1 0
====
go @@ selector @@ rule 9: a method call and a field write read their base; a keyed literal's key is no variable read when it names a field
func f(p Point, v int) Point {
	p.X = v
	p.Move(v)
	return Point{X: v}
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 9 0 0
v 0 -1 1 p
v 1 -1 1 v
x 0 1 0
x 0 0 0
x 1 0 0
x 1 1 0
x 2 1 0
"#;

#[test]
fn the_accesses_run_as_the_table_says() {
    run_table(TABLE);
}
