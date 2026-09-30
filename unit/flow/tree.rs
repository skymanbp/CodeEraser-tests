//! The statement tree across languages (plan v2.31 step 4 A2; design
//! booklet §5.1 rules 1–7 and 11): the rewrites that keep every jump
//! inside the tree contract, the one-child branch, and the flags the
//! lowering reads off the source rather than off a table row.

use crate::flow::lower::shape::run_table;

const TABLE: &str = r#"
rs @@ labelled-block @@ rule 4 and ruling 3: a labelled block is block · label, a break out of it a goto to the label
fn f(a: i32) -> i32 {
    let mut n = a;
    'done: {
        if n > 3 {
            break 'done;
        }
        n += 1;
    }
    n
}
----
s 0 -1 1 0 0
s 1 -1 0 0 0
s 2 1 1 0 0
s 3 1 2 0 0
s 4 3 0 0 0
s 5 4 13 0 7
s 6 1 1 0 0
s 7 -1 14 0 0
s 8 -1 1 0 0
v 0 -1 1 a
v 1 0 0 n
x 0 0 0
x 0 1 1
x 3 1 0
x 6 1 2
x 8 1 0
====
java @@ yield @@ rule 4: a yield is a break to its switch, its value read
int f(int k, int v) {
    switch (k) {
        case 1 -> { yield v; }
        default -> { yield 0; }
    }
    return v;
}
----
s 0 -1 4 1 0
s 1 0 5 0 0
s 2 1 0 0 0
s 3 2 11 0 0
s 4 0 5 0 0
s 5 4 0 0 0
s 6 5 11 0 0
s 7 -1 9 0 0
v 0 -1 1 k
v 1 -1 1 v
x 0 0 0
x 3 1 0
x 7 1 0
====
ts @@ with @@ rule 7: a `with` statement makes the unit dynamic
function f(o: object) {
  with (o) {
    x = 1;
  }
  return o;
}
----
s 0 -1 1 8 0
s 1 -1 9 0 0
v 0 -1 1 o
x 0 0 0
x 1 0 0
====
py @@ error @@ ruling 1: a statement holding an ERROR node makes the unit dynamic
def f(x):
    y = x +
    return y
----
s 0 -1 1 8 0
v 0 -1 1 x
v 1 0 0 y
x 0 0 0
x 0 1 0
x 0 1 1
====
lua @@ nearest-label @@ rule 3: a goto reaches the label in its own enclosing block when two share a name
local function f(t)
  for i = 1, 2 do
    if t then goto continue end
    ::continue::
  end
  for j = 1, 2 do
    goto continue
    ::continue::
  end
end
----
s 0 -1 3 0 0
s 1 0 0 0 0
s 2 1 2 0 0
s 3 2 0 0 0
s 4 3 13 0 5
s 5 1 14 0 0
s 6 -1 3 0 0
s 7 6 0 0 0
s 8 7 13 0 9
s 9 7 14 0 0
v 0 -1 1 t
v 1 0 0 i
v 2 6 0 j
x 0 1 1
x 2 0 0
x 6 2 1
====
c @@ branch @@ rules 2, 4: a branch lowering to several nodes (init · loop) is one synthetic block; a lone statement is the branch itself
int f(int x) {
    if (x)
        for (int i = 0; i < x; i++) g(i);
    else
        return 0;
    return 1;
}
----
s 0 -1 2 1 0
s 1 0 0 0 0
s 2 1 1 0 0
s 3 1 3 0 0
s 4 3 1 0 0
s 5 3 14 0 0
s 6 5 1 0 0
s 7 0 9 0 0
s 8 -1 9 0 0
v 0 -1 1 x
v 1 2 0 i
x 0 0 0
x 2 1 1
x 3 1 0
x 3 0 0
x 4 1 0
x 6 1 2
"#;

#[test]
fn the_tree_lowers_as_the_table_says() {
    run_table(TABLE);
}
