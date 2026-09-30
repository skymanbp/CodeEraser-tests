//! Rust lowered as the table says (plan v2.31 step 4 A2): each block one
//! rule of design booklet §5.1 on the Rust grammar, its rows the first
//! unit's statements, variables and accesses (unit/flow/shape.rs).

use crate::flow::lower::shape::run_table;

const TABLE: &str = r#"
rs @@ loops @@ rules 4, 5, 8: `loop` is infinite, a labelled break or continue reaches its loop, `while let` binds in the body
fn f(v: Vec<i32>) -> i32 {
    let mut it = v.into_iter();
    let mut s = 0;
    'outer: loop {
        while let Some(x) = it.next() {
            if x < 0 { continue 'outer; }
            if x > 9 { break 'outer; }
            s += x;
        }
        break;
    }
    s
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 3 2 0
s 3 2 0 0 0
s 4 3 3 0 0
s 5 4 0 0 0
s 6 5 2 0 0
s 7 6 0 0 0
s 8 7 12 0 2
s 9 5 2 0 0
s 10 9 0 0 0
s 11 10 11 0 2
s 12 5 1 0 0
s 13 3 11 0 2
s 14 -1 1 0 0
v 0 -1 1 v
v 1 0 0 it
v 2 1 0 s
v 3 4 0 x
x 0 0 0
x 0 1 1
x 1 2 1
x 4 1 0
x 4 3 1
x 6 3 0
x 9 3 0
x 12 3 0
x 12 2 2
x 14 2 0
====
rs @@ match @@ rule 6: a match always has its default and no arm falls through; a break in an arm leaves the loop, not the match
fn f(xs: &[Option<i32>]) -> i32 {
    let mut n = 0;
    for x in xs {
        match x {
            Some(v) if *v > 0 => n += v,
            Some(_) => break,
            None => return -1,
        }
    }
    n
}
----
s 0 -1 1 0 0
s 1 -1 3 0 0
s 2 1 0 0 0
s 3 2 4 1 0
s 4 3 5 0 0
s 5 4 1 0 0
s 6 3 5 0 0
s 7 6 11 0 1
s 8 3 5 0 0
s 9 8 9 0 0
s 10 -1 1 0 0
v 0 -1 1 xs
v 1 0 0 n
v 2 1 0 x
v 3 4 0 v
x 0 1 1
x 1 0 0
x 1 2 1
x 3 2 0
x 4 3 1
x 4 3 0
x 5 3 0
x 5 1 2
x 10 1 0
====
rs @@ if-let @@ rule 8: an `if let` binding lives in its branch, the else reads the outer name; a shadowing let is a new variable
fn f(path: Option<String>) -> usize {
    let n = 1;
    if let Some(path) = path.as_ref() {
        return path.len() + n;
    } else {
        drop(path);
    }
    let n = n + 1;
    n
}
----
s 0 -1 1 0 0
s 1 -1 2 1 0
s 2 1 0 0 0
s 3 2 9 0 0
s 4 1 0 0 0
s 5 4 1 0 0
s 6 -1 1 0 0
s 7 -1 1 0 0
v 0 -1 1 path
v 1 0 0 n
v 2 1 0 path
v 3 6 0 n
x 0 1 1
x 1 0 0
x 1 2 1
x 3 2 0
x 3 1 0
x 5 0 0
x 6 1 0
x 6 3 1
x 7 3 0
====
rs @@ macros @@ rules 3, 9, 10: a macro's arguments and `{name}` pieces are read, `panic!` never returns, `&mut` takes the address
fn f(a: i32) {
    let b = a * 2;
    let c = 3;
    let mut d = 0;
    bump(&mut d);
    println!("{b} {}", c);
    if a < 0 {
        panic!("negative");
    }
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 1 0 0
s 4 -1 1 0 0
s 5 -1 2 0 0
s 6 5 0 0 0
s 7 6 15 0 0
v 0 -1 1 a
v 1 0 0 b
v 2 1 0 c
v 3 2 8 d
x 0 0 0
x 0 1 1
x 1 2 1
x 2 3 1
x 3 3 0
x 4 1 0
x 4 2 0
x 5 0 0
====
rs @@ let-else @@ rule 8 and ruling 4: let-else is an if whose branch leaves; a closure's host names are captured; `?` is not lowered
fn f(o: Option<i32>) -> Result<i32, E> {
    let Some(v) = o else { return Err(E) };
    let k = 2;
    let g = |x: i32| x * k;
    let r = g(v)?;
    Ok(r)
}
----
s 0 -1 2 0 0
s 1 0 0 0 0
s 2 1 9 0 0
s 3 -1 1 0 0
s 4 -1 1 0 0
s 5 -1 1 0 0
s 6 -1 1 0 0
v 0 -1 1 o
v 1 0 0 v
v 2 3 2 k
v 3 4 0 g
v 4 5 0 r
x 0 0 0
x 0 1 1
x 3 2 1
x 4 3 1
x 5 3 0
x 5 1 0
x 5 4 1
x 6 4 0
====
rs @@ expression-loop @@ rule 9 and ruling 4: a loop in expression position reads its names again after itself; a write in an if expression's branch is a read-write
fn f(mut i: u32) -> u32 {
    let mut last = 0;
    let found = loop {
        if i > 10 { break last; }
        last = i;
        i += 1;
    };
    let y = if found > 3 { last = 1; 2 } else { 3 };
    y + last
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 1 0 0
v 0 -1 1 i
v 1 0 0 last
v 2 1 0 found
v 3 2 0 y
x 0 1 1
x 1 0 0
x 1 1 0
x 1 0 0
x 1 1 1
x 1 0 2
x 1 0 0
x 1 1 0
x 1 2 1
x 2 2 0
x 2 1 2
x 2 3 1
x 3 3 0
x 3 1 0
"#;

#[test]
fn rust_lowers_as_the_table_says() {
    run_table(TABLE);
}
