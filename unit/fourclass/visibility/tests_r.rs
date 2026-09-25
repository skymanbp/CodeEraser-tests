//! R legs (plan v2.30 step 4; design booklet §7 row R, register D16):
//! a roxygen file's `@export` tags, else the leading-dot convention,
//! and the enclosing function, through the extractor's own root. Every
//! shape here was probed on tree-sitter-r 1.3.0 before the reading was
//! written (the scripts/tsprobe transcripts). Two text tables through
//! the shared runner (tests.rs run_tables).

use super::super::tests::run_tables;

/// `R <source> ⇒ <name:letters …>` per line, newlines spelled `\n`, `#`
/// lines commentary — measured over the functions the extractor sees.
const FNS: &str = r#"
# Without roxygen, R's own convention: a name starting with a dot is
# hidden (`ls()` skips it); a function inside another closes the scope
# bit; an anonymous function value names nothing to export.
R f <- function() 1\n.g <- function() 1\nh <- function() {\n  k <- function() 1\n  k()\n}\nlapply(xs, function(v) v) ⇒ f:ES .g:- h:ES k:E (anonymous):-
# A roxygen file exports exactly what its blocks tag — `@export`, and
# `@exportS3Method` beside it — whatever the name spells, across the
# blank lines and plain comments roxygen skips; a chained assignment is
# one statement, so its block reaches the value.
R #' @export\nf <- function() 1\n#' Title\ng <- function() 1\n#' @exportS3Method\nprint.foo <- function(x) 1\n#' @export\n\n# plain\n.dot <- function() 1\n#' @export\nh <- k <- function() 1 ⇒ f:ES g:- print.foo:ES .dot:ES k:ES
"#;

/// The same line shape over every unit the register keys, spelled as
/// stored (`name/arity`).
const ALL: &str = r#"
R f <- function(a, b = 1, ...) a ⇒ f/3:ES
"#;

#[test]
fn r_words_follow_the_tables() {
    run_tables(FNS, ALL);
}
