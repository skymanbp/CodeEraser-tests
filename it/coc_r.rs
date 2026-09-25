//! Plan v2.30 step 4 R metric battery: the launch-language rows of
//! metrics.rs re-spelled in R where the construct exists, plus one row
//! per stance the R table records (scan/spec_r.rs, the booklet's
//! register D0 / D8 / D9) — R has no cognitive oracle (D0), and
//! lizard's R reader, which checks CC, reads the D8 and D9 stances the
//! other way (contracts/fixtures/crosscheck/DIVERGENCES.md), so this
//! table is the stances' one executor — and a `units=` block pinning
//! how an assignment names a unit (scan/binding.rs) and what the table
//! reads as its parameters. Recursion is the core's and is asserted
//! through it (coc_recursion.rs).

use crate::common;

const ROWS: &str = r#"
R @@ cc=5 coc=7 nesting=3 lines=10 @@ metrics.rs's Python row in R: cc 1 + if + && + for + if; coc if +1, && +1, for +2, if +3; nesting if > for > if
f <- function(a, b) {
  if (a > 0 && b > 0) {
    for (i in 1:10) {
      if (i > 5) {
        return(i)
      }
    }
  }
  0
}
====
R @@ cc=5 coc=5 nesting=1 lines=9 @@ metrics.rs's TypeScript row in R: an if in the alternative is the flat else-if (p.7), and the trailing else pays +1 without nesting
g <- function(a, b) {
  if (a > 0 && b > 0) {
    a + b
  } else if (a > 0 || b > 0) {
    1
  } else {
    0
  }
}
====
R @@ cc=2 coc=2 nesting=1 lines=3 @@ R's else has no node of its own: the alternative IS the else expression, and a bare expression there pays the else +1 exactly as a braced one does
s <- function(a) {
  if (a > 0) 1 else 0
}
====
R @@ cc=5 coc=5 nesting=2 lines=7 @@ for, while and repeat are loops; the if inside the repeat nests once, and `break` names no label (register D5: R has none)
l <- function(n) {
  s <- 0
  for (i in seq_len(n)) s <- s + i
  while (s > 0) s <- s - 1
  repeat { s <- s + 1; if (s >= 3) break }
  s
}
====
R @@ cc=5 coc=3 nesting=1 lines=3 @@ a run of one operator pays +1 and each change of operator +1 more (p.5): if +1, the && run +1, || +1, while CC counts every short-circuit operator
k <- function(a, b, c, d) {
  if (a && b && c || d) 1
}
====
R @@ cc=1 coc=0 nesting=0 lines=5 @@ the vectorised `&` and `|` branch nothing (register D9), and `ifelse`, `switch` and `tryCatch` are calls, invisible to the syntax (register D8)
v <- function(x, y) {
  z <- ifelse(x > 0 & y > 0 | x < 0, 1, 0)
  switch(z, a = 1, b = 2)
  tryCatch(stop("e"), finally = cat("done"))
}
====
R @@ units=f/3, g/0, h/1, k/1, s/0, x$m/1, `back tick`/0, (anonymous)/1, (anonymous)/1 @@ how an assignment names a unit (booklet §4): `<-`, `=` and `<<-` by the left side, `->` by the right (through the parentheses R needs), a string target by its content, a member target and a backticked name as written; an argument value and a bare `\(w)` lambda are `(anonymous)`. The parameters count every formal, a default and `...` included, and never the commas tree-sitter-r keeps as named nodes
f <- function(a, b = 1, ...) a
g = function() 1
h <<- function(x) x
(function(y) y) -> k
"s" <- function() 1
x$m <- function(z) z
`back tick` <- function() 1
lapply(xs, function(v) v)
\(w) w
"#;

#[test]
fn r_metrics_follow_the_table() {
    common::assert_metric_table(ROWS);
}
