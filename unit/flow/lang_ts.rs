//! TypeScript and TSX lowered as the table says (plan v2.31 step 4
//! A2): each block one rule of design booklet §5.1 on the one table
//! the two grammars share, its rows the first unit's statements,
//! variables and accesses (unit/flow/shape.rs).

use crate::flow::lower::shape::run_table;

const TABLE: &str = r#"
ts @@ c-for @@ rule 4: a C-style for is init · loop, the update a label ending the body, the loop's continue a goto to it
function f(n: number) {
  let s = 0;
  for (let i = 0; i < n; i++) {
    if (i % 2) continue;
    s += i;
  }
  return s;
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 3 0 0
s 3 2 0 0 0
s 4 3 2 0 0
s 5 4 13 0 7
s 6 3 1 0 0
s 7 2 14 0 0
s 8 7 1 0 0
s 9 -1 9 0 0
v 0 -1 1 n
v 1 0 0 s
v 2 1 0 i
x 0 1 1
x 1 2 1
x 2 2 0
x 2 0 0
x 4 2 0
x 6 2 0
x 6 1 2
x 8 2 2
x 9 1 0
====
ts @@ switch @@ rule 6: every case falls through, a default gives has_else, a break leaves the switch
function f(k: string) {
  let r = 0;
  switch (k) {
    case "a":
      r = 1;
      break;
    case "b":
    default:
      r = 2;
  }
  return r;
}
----
s 0 -1 1 0 0
s 1 -1 4 1 0
s 2 1 5 4 0
s 3 2 1 0 0
s 4 2 11 0 1
s 5 1 5 4 0
s 6 1 5 4 0
s 7 6 1 0 0
s 8 -1 9 0 0
v 0 -1 1 k
v 1 0 0 r
x 0 1 1
x 1 0 0
x 3 1 1
x 7 1 1
x 8 1 0
====
ts @@ labels @@ rule 4 and ruling 3: a labelled continue reaches its loop; a break out of a labelled block is a goto to a label after it
function f(xs: number[][]) {
  outer: for (const row of xs) {
    for (const x of row) {
      if (x < 0) continue outer;
      if (x > 9) break outer;
    }
  }
  done: {
    if (xs.length) break done;
    g();
  }
  return 0;
}
----
s 0 -1 14 0 0
s 1 0 3 0 0
s 2 1 0 0 0
s 3 2 3 0 0
s 4 3 0 0 0
s 5 4 2 0 0
s 6 5 12 0 1
s 7 4 2 0 0
s 8 7 11 0 1
s 9 -1 14 0 0
s 10 9 0 0 0
s 11 10 2 0 0
s 12 11 13 0 14
s 13 10 1 0 0
s 14 -1 14 0 0
s 15 -1 9 0 0
v 0 -1 1 xs
v 1 1 0 row
v 2 3 0 x
x 1 0 0
x 1 1 1
x 3 1 0
x 3 2 1
x 5 2 0
x 7 2 0
x 11 0 0
====
ts @@ loops-try @@ rules 2, 3, 5: do-while, `while (true)` infinite, try · catch · finally, throw
function f(a: number) {
  do {
    a--;
  } while (a > 0);
  while (true) {
    try {
      a = g(a);
      if (a) throw new Error("x");
    } catch (e) {
      h(e);
    } finally {
      a = 0;
    }
  }
}
----
s 0 -1 3 0 0
s 1 0 0 0 0
s 2 1 1 0 0
s 3 -1 3 2 0
s 4 3 0 0 0
s 5 4 6 0 0
s 6 5 0 0 0
s 7 6 1 0 0
s 8 6 2 0 0
s 9 8 10 0 0
s 10 5 7 0 0
s 11 10 0 0 0
s 12 11 1 0 0
s 13 5 8 0 0
s 14 13 0 0 0
s 15 14 1 0 0
v 0 -1 1 a
v 1 10 0 e
x 0 0 0
x 2 0 2
x 7 0 0
x 7 0 1
x 8 0 0
x 10 1 1
x 12 1 0
x 15 0 1
====
ts @@ destructure @@ rules 8, 9: patterns bind each name, defaults read first, var is function-wide, a shorthand reads, a member write reads its base, `??` guards a write
function f({ a, b }: P, [c] = [1], d = a) {
  var t = { a };
  let [x, ...ys] = b;
  t.a = c;
  x ||= d;
  const z = d ?? (x = 2);
  return [t, ys, z, x];
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 1 0 0
s 4 -1 1 0 0
s 5 -1 1 0 0
s 6 -1 9 0 0
v 0 -1 1 a
v 1 -1 1 b
v 2 -1 1 c
v 3 -1 1 d
v 4 1 0 t
v 5 2 0 x
v 6 2 0 ys
v 7 5 0 z
x 0 0 0
x 1 0 0
x 1 4 1
x 2 1 0
x 2 5 1
x 2 6 1
x 3 2 0
x 3 4 0
x 4 3 0
x 4 5 2
x 5 3 0
x 5 5 2
x 5 7 1
x 6 4 0
x 6 6 0
x 6 7 0
x 6 5 0
====
ts @@ closure @@ rules 3, 7, 10: an arrow is its own unit, the host names it captures are exempt; process.exit never returns
function f(n: number) {
  let hits = 0;
  const inc = () => hits++;
  inc();
  if (n < 0) process.exit(1);
  return hits;
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 2 0 0
s 4 3 15 0 0
s 5 -1 9 0 0
v 0 -1 1 n
v 1 0 2 hits
v 2 1 0 inc
x 0 1 1
x 1 2 1
x 2 2 0
x 3 0 0
x 5 1 0
====
ts @@ arrow @@ rule 1 and ruling 3: an expression-bodied arrow is one return
const g = (x: number, y = x) => x + y;
----
s 0 -1 1 0 0
s 1 -1 9 0 0
v 0 -1 1 x
v 1 -1 1 y
x 0 0 0
x 1 0 0
x 1 1 0
====
tsx @@ jsx @@ rule 9: JSX reads its locals, `<Item />` reads a local component
function App(props: Props) {
  const Item = pick(props);
  const title = props.title;
  return <div className="x"><Item label={title} /></div>;
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 9 0 0
v 0 -1 1 props
v 1 0 0 Item
v 2 1 0 title
x 0 0 0
x 0 1 1
x 1 0 0
x 1 2 1
x 2 1 0
x 2 2 0
"#;

#[test]
fn typescript_lowers_as_the_table_says() {
    run_table(TABLE);
}
