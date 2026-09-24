//! Sonar Cognitive Complexity whitepaper v1.7 (2023-08-29) worked
//! examples in the language the whitepaper writes them in — Java, once
//! plan v2.30 step 3 made it a judged language. Quoted from the PDF
//! (sonarsource.com/docs/CognitiveComplexity.pdf) with two repairs
//! only: the typeset quotes around its char literals are Java's `'`
//! again, and the elided bodies `{ ... }` are empty (the ellipsis is
//! no Java). The two p.8 condition fragments sit in a method of their
//! own, since a condition is no unit. Every expected value is the
//! margin annotation at the cited page, never re-derived, and `cc`
//! appears only where the page states it; getWords is the one row
//! whose `cc` departs from its page, by the register entry it cites.
//! Left out, with no Java form: the `#if DEBUG` variant of myMethod2
//! (a C# preprocessor) and Appendix C's model.js `save` (JavaScript).

use crate::common;

const ROWS: &str = r#"
java @@ coc=7 cc=4 @@ p.10 margin: for +1, for +2, if +3, continue OUT +1; p.5 margin: cyclomatic 4
int sumOfPrimes(int max) {
   int total = 0;
   OUT: for (int i = 1; i <= max; ++i) {
      for (int j = 2; j < i; ++j) {
         if (i % j == 0) {
            continue OUT;
         }
      }
      total += i;
   }
   return total;
}
====
java @@ coc=1 cc=5 @@ p.10 margin: the switch and all its cases are one increment; cc 5, not p.5's 4 — `default:` is a path (register D2, booklet §5)
String getWords(int number) {
   switch (number) {
      case 1:
         return "one";
      case 2:
         return "a couple";
      case 3:
         return "a few";
      default:
         return "lots";
   }
}
====
java @@ coc=4 @@ p.8 margin: if +1, then +1 for each new sequence of like operators (&&, ||, &&)
boolean sequences(boolean a, boolean b, boolean c, boolean d, boolean e, boolean f) {
   if (a
          && b && c
          || d || e
          && f)
      return true;
   return false;
}
====
java @@ coc=3 @@ p.8 margin: if +1, && +1, and the && inside the negated parentheses starts a sequence of its own +1
boolean negated(boolean a, boolean b, boolean c) {
   if (a
          &&
          !(b && c))
      return true;
   return false;
}
====
java @@ coc=9 @@ p.9 margin: if +1, for +2, while +3, catch +1, if +2 — try is no structure and nests nothing
void myMethod () {
   try {
      if (condition1) {
         for (int i = 0; i < 10; i++) {
            while (condition2) { }
         }
      }
   } catch (ExcepType1 | ExcepType2 e) {
      if (condition2) { }
   }
}
====
java @@ coc=2 @@ p.9 margin: the lambda increments nothing but raises the nesting level, so its if pays +2
void myMethod2 () {
   Runnable r = () -> {
       if (condition1) { }
   };
}
====
java @@ coc=19 @@ p.17 margin total (Appendix C, JavaSymbol.java in SonarJava)
@Nullable
private MethodJavaSymbol overriddenSymbolFrom(ClassJavaType classType) {
  if (classType.isUnknown()) {
    return Symbols.unknownMethodSymbol;
  }
  boolean unknownFound = false;
  List<JavaSymbol> symbols = classType.getSymbol().members().lookup(name);
  for (JavaSymbol overrideSymbol : symbols) {
    if (overrideSymbol.isKind(JavaSymbol.MTH)
        && !overrideSymbol.isStatic()) {
      MethodJavaSymbol methodJavaSymbol = (MethodJavaSymbol)overrideSymbol;
      if (canOverride(methodJavaSymbol)) {
        Boolean overriding = checkOverridingParameters(methodJavaSymbol,
            classType);
        if (overriding == null) {
          if (!unknownFound) {
            unknownFound = true;
          }
        } else if (overriding) {
          return methodJavaSymbol;
        }
      }
    }
  }
  if (unknownFound) {
    return Symbols.unknownMethodSymbol;
  }
  return null;
}
====
java @@ coc=35 @@ p.18 margin total (Appendix C, TimelyResource.java in sonar-persistit): synchronized, like try, is no structure
private void addVersion(final Entry entry, final Transaction txn)
    throws PersistitInterruptedException, RollbackException {
  final TransactionIndex ti = _persistit.getTransactionIndex();
  while (true) {
    try {
      synchronized (this) {
        if (frst != null) {
          if (frst.getVersion() > entry.getVersion()) {
            throw new RollbackException();
          }
          if (txn.isActive()) {
            for
                (Entry e = frst; e != null; e = e.getPrevious()) {
              final long version = e.getVersion();
              final long depends = ti.wwDependency(version,
                  txn.getTransactionStatus(), 0);
              if (depends == TIMED_OUT) {
                throw new WWRetryException(version);
              }
              if (depends != 0
                  && depends != ABORTED) {
                throw new RollbackException();
              }
            }
          }
        }
        entry.setPrevious(frst);
        frst = entry;
        break;
      }
    } catch (final WWRetryException re) {
      try {
        final long depends = _persistit.getTransactionIndex()
            .wwDependency(re.getVersionHandle(),txn.getTransactionStatus(),
            SharedResource.DEFAULT_MAX_WAIT_TIME);
        if (depends != 0
            && depends != ABORTED) {
          throw new RollbackException();
        }
      } catch (final InterruptedException ie) {
        throw new PersistitInterruptedException(ie);
      }
    } catch (final InterruptedException ie) {
      throw new PersistitInterruptedException(ie);
    }
  }
}
====
java @@ coc=20 @@ p.19 margin total (Appendix C, WildcardPattern.java in SonarQube)
private static String toRegexp(String antPattern,
    String directorySeparator) {
  final String escapedDirectorySeparator = '\\' + directorySeparator;
  final StringBuilder sb = new StringBuilder(antPattern.length());
  sb.append('^');
  int i = antPattern.startsWith("/") ||
      antPattern.startsWith("\\") ? 1 : 0;
  while (i < antPattern.length()) {
    final char ch = antPattern.charAt(i);
    if (SPECIAL_CHARS.indexOf(ch) != -1) {
      sb.append('\\').append(ch);
    } else if (ch == '*') {
      if (i + 1 < antPattern.length()
          && antPattern.charAt(i + 1) == '*') {
        if (i + 2 < antPattern.length()
            && isSlash(antPattern.charAt(i + 2))) {
          sb.append("(?:.*")
              .append(escapedDirectorySeparator).append("|)");
          i += 2;
        } else {
          sb.append(".*");
          i += 1;
        }
      } else {
        sb.append("[^").append(escapedDirectorySeparator).append("]*?");
      }
    } else if (ch == '?') {
      sb.append("[^").append(escapedDirectorySeparator).append("]");
    } else if (isSlash(ch)) {
      sb.append(escapedDirectorySeparator);
    } else {
      sb.append(ch);
    }
    i++;
  }
  sb.append('$');
  return sb.toString();
}
"#;

#[test]
fn whitepaper_worked_examples_in_java() {
    common::assert_metric_table(ROWS);
}
