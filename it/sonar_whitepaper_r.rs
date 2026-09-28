//! The whitepaper's worked examples in R (sonar_whitepaper_c.rs says
//! what a row is and where its expected value comes from). `!` and
//! the short-circuit `&&` / `||` are R's own, the else-if chain is an
//! `if` in the alternative (the flat hybrid), a nullable Boolean is
//! NULL. R has no conditional operator and `ifelse` is a call
//! (register D8); toRegexp therefore starts with an `if` at nesting 0,
//! which pays the ternary's +1, and its `||` pays the second point
//! the page gives that line. No R form (the booklet's "no switch /
//! goto" register, Haskell D4 alike): sumOfPrimes (`next` and `break`
//! name no label, register D5), getWords (`switch` is a call, D8),
//! myMethod and addVersion (`tryCatch` is a call, D8), myMethod2 (D3
//! makes the function value a unit of its own, so no single unit
//! carries the page's host-plus-lambda total; coc_r.rs's units row
//! pins the split).

use crate::common;

const ROWS: &str = r#"
R @@ coc=4 @@ p.8 margin: if +1, then +1 for each new sequence of like operators (&&, ||, &&)
sequences <- function(a, b, c, d, e, f) {
   if (a
          && b && c
          || d || e
          && f)
      return(TRUE)
   FALSE
}
====
R @@ coc=3 @@ p.8 margin: if +1, && +1, and the && inside the negated parentheses starts a sequence of its own +1
negated <- function(a, b, c) {
   if (a
          &&
          !(b && c))
      return(TRUE)
   FALSE
}
====
R @@ coc=19 @@ p.17 margin total (Appendix C, JavaSymbol.java in SonarJava): the for-each is R's for and the nullable Boolean a NULL, the structure is the page's
overriddenSymbolFrom <- function(classType) {
  if (isUnknown(classType)) {
    return(Symbols$unknownMethodSymbol)
  }
  unknownFound <- FALSE
  symbols <- lookup(members(getSymbol(classType)), name)
  for (overrideSymbol in symbols) {
    if (isKind(overrideSymbol, JavaSymbol$MTH)
        && !isStatic(overrideSymbol)) {
      methodJavaSymbol <- overrideSymbol
      if (canOverride(methodJavaSymbol)) {
        overriding <- checkOverridingParameters(methodJavaSymbol,
            classType)
        if (is.null(overriding)) {
          if (!unknownFound) {
            unknownFound <- TRUE
          }
        } else if (overriding) {
          return(methodJavaSymbol)
        }
      }
    }
  }
  if (unknownFound) {
    return(Symbols$unknownMethodSymbol)
  }
  NULL
}
====
R @@ coc=20 @@ p.19 margin total (Appendix C, WildcardPattern.java in SonarQube): the ternary is an if at nesting 0 (its +1 beside the ||'s +1), the else-if chain is the page's
toRegexp <- function(antPattern, directorySeparator) {
  escapedDirectorySeparator <- paste0("\\", directorySeparator)
  sb <- "^"
  i <- 0
  if (startsWith(antPattern, "/") ||
      startsWith(antPattern, "\\")) i <- 1
  while (i < nchar(antPattern)) {
    ch <- substr(antPattern, i + 1, i + 1)
    if (grepl(ch, SPECIAL_CHARS, fixed = TRUE)) {
      sb <- paste0(sb, "\\", ch)
    } else if (ch == "*") {
      if (i + 1 < nchar(antPattern)
          && substr(antPattern, i + 2, i + 2) == "*") {
        if (i + 2 < nchar(antPattern)
            && isSlash(substr(antPattern, i + 3, i + 3))) {
          sb <- paste0(sb, "(?:.*", escapedDirectorySeparator, "|)")
          i <- i + 2
        } else {
          sb <- paste0(sb, ".*")
          i <- i + 1
        }
      } else {
        sb <- paste0(sb, "[^", escapedDirectorySeparator, "]*?")
      }
    } else if (ch == "?") {
      sb <- paste0(sb, "[^", escapedDirectorySeparator, "]")
    } else if (isSlash(ch)) {
      sb <- paste0(sb, escapedDirectorySeparator)
    } else {
      sb <- paste0(sb, ch)
    }
    i <- i + 1
  }
  paste0(sb, "$")
}
"#;

#[test]
fn whitepaper_worked_examples_in_r() {
    common::assert_metric_table(ROWS);
}
