//! R lowered as the table says (plan v2.31 step 4 A2): each block one
//! rule of design booklet §5.1 on the R grammar, its rows the first
//! unit's statements, variables and accesses (unit/flow/shape.rs).

use crate::flow::lower::shape::run_table;

const TABLE: &str = r#"
R @@ if-return @@ rules 2, 3, 8: an else-if nests; `return(x)` is kind 9 and `stop()` kind 15 by callee name; a first write declares
f <- function(x) {
  if (x > 0) {
    y <- 1
  } else if (x < 0) {
    stop("neg")
  } else {
    return(NULL)
  }
  y
}
----
s 0 -1 2 1 0
s 1 0 0 0 0
s 2 1 1 0 0
s 3 0 2 1 0
s 4 3 0 0 0
s 5 4 15 0 0
s 6 3 0 0 0
s 7 6 9 0 0
s 8 -1 1 0 0
v 0 -1 1 x
v 1 2 0 y
x 0 0 0
x 2 1 1
x 3 0 0
x 8 1 0
====
R @@ loops @@ rules 3, 5, 8: for declares its variable; `next` continues, break leaves; repeat and `while (TRUE)` are infinite
f <- function(xs) {
  s <- 0
  for (x in xs) {
    if (is.na(x)) next
    s <- s + x
  }
  repeat {
    s <- s - 1
    if (s < 0) break
  }
  while (TRUE) {
    s <- s + 1
  }
}
----
s 0 -1 1 0 0
s 1 -1 3 0 0
s 2 1 0 0 0
s 3 2 2 0 0
s 4 3 12 0 1
s 5 2 1 0 0
s 6 -1 3 2 0
s 7 6 0 0 0
s 8 7 1 0 0
s 9 7 2 0 0
s 10 9 11 0 6
s 11 -1 3 2 0
s 12 11 0 0 0
s 13 12 1 0 0
v 0 -1 1 xs
v 1 0 0 s
v 2 1 0 x
x 0 1 1
x 1 0 0
x 1 2 1
x 3 2 0
x 5 1 0
x 5 2 0
x 5 1 1
x 8 1 0
x 8 1 1
x 9 1 0
x 13 1 0
x 13 1 1
====
R @@ assign-ops @@ rules 8, 9: `=` and `->` write, `<<-` writes no local; `x$a <- v` and `names(x) <- v` read their base
f <- function(a) {
  b = a * 2
  a * 3 -> c
  total <<- c
  b$field <- 1
  names(b) <- "x"
  b
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 1 0 0
s 4 -1 1 0 0
s 5 -1 1 0 0
v 0 -1 1 a
v 1 0 0 b
v 2 1 0 c
x 0 0 0
x 0 1 1
x 1 0 0
x 1 2 1
x 2 2 0
x 3 1 0
x 4 1 0
x 5 1 0
====
R @@ glue @@ rules 3, 9, 10: a default argument is read first; a `{name}` in a string is read; a function value captures; cli::cli_abort never returns
f <- function(x, n = length(x)) {
  msg <- "bad"
  g <- function() msg
  cli::cli_abort("{msg} with {n}")
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 15 0 0
v 0 -1 1 x
v 1 -1 1 n
v 2 1 2 msg
v 3 2 0 g
x 0 0 0
x 1 2 1
x 2 3 1
x 3 2 0
x 3 1 0
====
R @@ dynamic @@ rules 7, 9: a write in an `&&` right operand or an if expression's branch is a read-write; assign() makes the unit dynamic
f <- function(env) {
  ok <- FALSE
  isTRUE(env) && (ok <- TRUE)
  y <- if (ok) (ok <- 2) else 3
  assign("x", y)
  ok
}
----
s 0 -1 1 0 0
s 1 -1 1 0 0
s 2 -1 1 0 0
s 3 -1 1 8 0
s 4 -1 1 0 0
v 0 -1 1 env
v 1 0 0 ok
v 2 2 0 y
x 0 1 1
x 1 0 0
x 1 1 2
x 2 1 0
x 2 1 2
x 2 2 1
x 3 2 0
x 4 1 0
====
R @@ expression-body @@ rule 1: an expression body is one return
f <- function(a, b = 2) a + b
----
s 0 -1 1 0 0
s 1 -1 9 0 0
v 0 -1 1 a
v 1 -1 1 b
x 1 0 0
x 1 1 0
"#;

#[test]
fn r_lowers_as_the_table_says() {
    run_table(TABLE);
}
