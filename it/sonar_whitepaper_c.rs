//! Sonar Cognitive Complexity whitepaper v1.7 (2023-08-29) worked
//! examples in C (plan v2.30 step 6: the whitepaper table widened from
//! the launch languages to every judged language that parses, booklet
//! §11). Each row is sonar_whitepaper_java.rs's quoted original spelled
//! in C with the construct the whitepaper scores kept as is: the
//! labeled `continue OUT` is a `goto` to a label at the end of the
//! outer loop's body (p.8: a jump to a label is a fundamental +1, the
//! same point the labeled continue pays), a `boolean` is an `int`, a
//! for-each is a pointer walk, a nullable `Boolean` is a pointer that
//! may be NULL. Every expected value is the margin annotation at the
//! cited page, never re-derived, and `cc` appears only where the page
//! states it. Registered as having no C form: myMethod (try / catch),
//! myMethod2 (a lambda; C has none), addVersion (try / catch). Already
//! pinned in coc_c.rs: getWords (C, register D2 for its `cc`) and
//! myMethod2 as a C++ lambda.

use crate::common;

const ROWS: &str = r#"
c @@ coc=7 cc=4 @@ p.10 margin: for +1, for +2, if +3, the jump to the label +1 (`goto OUT` where Java continues OUT); p.5 margin: cyclomatic 4, a goto is no decision
int sumOfPrimes(int max) {
   int total = 0;
   for (int i = 1; i <= max; ++i) {
      for (int j = 2; j < i; ++j) {
         if (i % j == 0) {
            goto OUT;
         }
      }
      total += i;
   OUT:;
   }
   return total;
}
====
c @@ coc=4 @@ p.8 margin: if +1, then +1 for each new sequence of like operators (&&, ||, &&)
int sequences(int a, int b, int c, int d, int e, int f) {
   if (a
          && b && c
          || d || e
          && f)
      return 1;
   return 0;
}
====
c @@ coc=3 @@ p.8 margin: if +1, && +1, and the && inside the negated parentheses starts a sequence of its own +1
int negated(int a, int b, int c) {
   if (a
          &&
          !(b && c))
      return 1;
   return 0;
}
====
c @@ coc=19 @@ p.17 margin total (Appendix C, JavaSymbol.java in SonarJava): the for-each is a pointer walk and the nullable Boolean a pointer, the structure is the page's
static MethodJavaSymbol *overriddenSymbolFrom(ClassJavaType *classType) {
  if (classType_isUnknown(classType)) {
    return unknownMethodSymbol;
  }
  int unknownFound = 0;
  JavaSymbolList *symbols = members_lookup(classType_getSymbol(classType), name);
  for (JavaSymbol *overrideSymbol = symbols->first; overrideSymbol != NULL; overrideSymbol = overrideSymbol->next) {
    if (symbol_isKind(overrideSymbol, MTH)
        && !symbol_isStatic(overrideSymbol)) {
      MethodJavaSymbol *methodJavaSymbol = (MethodJavaSymbol *)overrideSymbol;
      if (canOverride(methodJavaSymbol)) {
        Boolean *overriding = checkOverridingParameters(methodJavaSymbol,
            classType);
        if (overriding == NULL) {
          if (!unknownFound) {
            unknownFound = 1;
          }
        } else if (*overriding) {
          return methodJavaSymbol;
        }
      }
    }
  }
  if (unknownFound) {
    return unknownMethodSymbol;
  }
  return NULL;
}
====
c @@ coc=20 @@ p.19 margin total (Appendix C, WildcardPattern.java in SonarQube): the StringBuilder is a buffer, the chars are chars, the ternary and the else-if chain are the page's
static char *toRegexp(const char *antPattern, const char *directorySeparator) {
  char escapedDirectorySeparator[8];
  strcpy(escapedDirectorySeparator, "\\");
  strcat(escapedDirectorySeparator, directorySeparator);
  Buffer sb = buffer_new();
  buffer_append(&sb, "^");
  size_t i = startsWith(antPattern, "/") ||
      startsWith(antPattern, "\\") ? 1 : 0;
  while (i < strlen(antPattern)) {
    char ch = antPattern[i];
    if (strchr(SPECIAL_CHARS, ch) != NULL) {
      buffer_append(&sb, "\\");
      buffer_append_char(&sb, ch);
    } else if (ch == '*') {
      if (i + 1 < strlen(antPattern)
          && antPattern[i + 1] == '*') {
        if (i + 2 < strlen(antPattern)
            && isSlash(antPattern[i + 2])) {
          buffer_append(&sb, "(?:.*");
          buffer_append(&sb, escapedDirectorySeparator);
          buffer_append(&sb, "|)");
          i += 2;
        } else {
          buffer_append(&sb, ".*");
          i += 1;
        }
      } else {
        buffer_append(&sb, "[^");
        buffer_append(&sb, escapedDirectorySeparator);
        buffer_append(&sb, "]*?");
      }
    } else if (ch == '?') {
      buffer_append(&sb, "[^");
      buffer_append(&sb, escapedDirectorySeparator);
      buffer_append(&sb, "]");
    } else if (isSlash(ch)) {
      buffer_append(&sb, escapedDirectorySeparator);
    } else {
      buffer_append_char(&sb, ch);
    }
    i++;
  }
  buffer_append(&sb, "$");
  return buffer_string(&sb);
}
"#;

#[test]
fn whitepaper_worked_examples_in_c() {
    common::assert_metric_table(ROWS);
}
