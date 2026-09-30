//! The reads a grammar hides (plan v2.31 step 4 E; design booklet
//! §5.1 rules 8, 9 and 10): one block per class the first-generation
//! precision docs itemised — RS-1 format placeholders, TS-1 forward
//! captures, TS-2 `typeof`, TS-3 parameter properties, CPP-1 member
//! initializers, CPP-2 the most vexing parse, R-1 dispatch — each the
//! first unit's rows (unit/flow/shape.rs), and LEG-1's statement end
//! lines.

use crate::flow::lower::lower_file;
use crate::flow::lower::shape::run_table;
use crate::scan::lang::Lang;

const TABLE: &str = r#"
rs @@ format @@ RS-1: every placeholder of a string in a macro reads its name, a raw string's too; `{name$}` counts read; `{{` / `}}` are literal
fn f(name: &str, pad: usize, k: i32, w: usize, idx: usize) {
    write!(out, r"-{name}");
    println!("{:pad$}{{{k}}}{:>w$.1}{0}{}", 1, 2.0);
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
v 0 -1 1 name
v 1 -1 1 pad
v 2 -1 1 k
v 3 -1 1 w
v 4 -1 1 idx
x 0 0 0
x 1 1 0
x 1 2 0
x 1 3 0
====
ts @@ forward @@ TS-1: a closure or getter naming a const declared after it, or the one its own initializer declares, captures it
function f() {
  const a = z.lazy(() => a);
  const g = h(() => b);
  const A = o({ get x() { return A; } });
  const b = 1;
  return g;
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 1 0 0
s 4 -1 9 0 0
v 0 0 2 a
v 1 1 0 g
v 2 2 2 A
v 3 3 2 b
x 0 0 1
x 1 1 1
x 2 2 1
x 3 3 1
x 4 1 0
====
ts @@ typeof @@ TS-2: a type taken from a value reads it, in an annotation or an alias
function f() {
  const o = 1;
  let t: typeof o;
  type T = typeof o;
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
v 0 0 0 o
v 1 1 0 t
x 0 0 1
x 1 0 0
x 2 0 0
====
ts @@ property @@ TS-3: a parameter that declares a field is read in the synthetic entry statement
class Z {
  constructor(public value: string, readonly r: number, plain: number) {}
}
----
s 0 -1 1 0 0
v 0 -1 1 value
v 1 -1 1 r
v 2 -1 1 plain
x 0 0 0
x 0 1 0
====
cpp @@ initializers @@ CPP-1: a constructor's member-initializer list reads its arguments in the synthetic entry statement
struct S {
  S(int a, int b, int c) : x_(a), B(b + 1) {}
  int x_;
};
----
s 0 -1 1 0 0
v 0 -1 1 a
v 1 -1 1 b
v 2 -1 1 c
x 0 0 0
x 0 1 0
====
cpp @@ vexing @@ CPP-2: `T x(a, b);` reads its arguments; a real prototype's type names resolve to nothing
void f(int data, int size) {
  data += 1;
  wrapper w(data, sizeof(size));
  int g(long);
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
v 0 -1 1 data
v 1 -1 1 size
x 0 0 2
x 1 0 0
x 1 1 0
====
R @@ dispatch @@ R-1: UseMethod / NextMethod hand the whole call on: every parameter is read there
type <- function(x, error_call = caller_env()) {
  UseMethod("type")
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
v 0 -1 1 x
v 1 -1 1 error_call
x 1 0 0
x 1 1 0
"#;

#[test]
fn the_hidden_reads_run_as_the_table_says() {
    run_table(TABLE);
}

/// LEG-1: each statement's end line beside its start — the if, its
/// then block, the return, the assignment over three lines — the
/// legend's own (the wire never carries it).
#[test]
fn every_statement_names_its_end_line() {
    let src = "def f(a):\n    if a:\n        return 1\n    x = (\n        2\n    )\n";
    let done = lower_file(src, Lang::Python).expect("a flow language");
    let u = &done.units[0];
    let lines: Vec<(u32, u32)> = u
        .legend
        .stmt_at
        .iter()
        .zip(&u.legend.stmt_end)
        .map(|(at, end)| (at.0, *end))
        .collect();
    assert_eq!(lines, [(2, 3), (3, 3), (3, 3), (4, 6)]);
}
