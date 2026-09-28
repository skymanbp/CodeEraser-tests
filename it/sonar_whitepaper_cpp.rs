//! The whitepaper's worked examples in C++, on the row form
//! sonar_whitepaper_c.rs states: the C table's rows again (the C++
//! grammar is a parser of its own) plus the three C could not spell —
//! myMethod's try / catch, where the page's multi-catch is one handler
//! (C++ has no `|` in a handler and the page scores each catch +1),
//! addVersion's `synchronized (this)` as a block holding a lock guard
//! (a compound statement is no structure, as the page says of
//! synchronized), and getWords, whose `cc` departs from its page by
//! the register entry it cites (D2), as in the Java table. Java's
//! for-each is a range for here and the nullable Boolean an optional.
//! myMethod2 as a lambda is already pinned in coc_c.rs.

use crate::common;

const ROWS: &str = r#"
cpp @@ coc=7 cc=4 @@ p.10 margin: for +1, for +2, if +3, the jump to the label +1 (`goto OUT` where Java continues OUT); p.5 margin: cyclomatic 4, a goto is no decision
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
cpp @@ coc=1 cc=5 @@ p.10 margin: the switch and all its cases are one increment; cc 5, not p.5's 4 — `default:` is a path (register D2, the C family's cc_kinds)
std::string getWords(int number) {
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
cpp @@ coc=4 @@ p.8 margin: if +1, then +1 for each new sequence of like operators (&&, ||, &&)
bool sequences(bool a, bool b, bool c, bool d, bool e, bool f) {
   if (a
          && b && c
          || d || e
          && f)
      return true;
   return false;
}
====
cpp @@ coc=3 @@ p.8 margin: if +1, && +1, and the && inside the negated parentheses starts a sequence of its own +1
bool negated(bool a, bool b, bool c) {
   if (a
          &&
          !(b && c))
      return true;
   return false;
}
====
cpp @@ coc=9 @@ p.9 margin: if +1, for +2, while +3, catch +1, if +2 — try is no structure and nests nothing; the page's multi-catch is one handler here
void myMethod() {
   try {
      if (condition1) {
         for (int i = 0; i < 10; i++) {
            while (condition2) { }
         }
      }
   } catch (const ExcepType1&) {
      if (condition2) { }
   }
}
====
cpp @@ coc=19 @@ p.17 margin total (Appendix C, JavaSymbol.java in SonarJava): the for-each is a range for and the nullable Boolean an optional, the structure is the page's
MethodJavaSymbol* overriddenSymbolFrom(ClassJavaType& classType) {
  if (classType.isUnknown()) {
    return Symbols::unknownMethodSymbol;
  }
  bool unknownFound = false;
  std::vector<JavaSymbol*> symbols = classType.getSymbol().members().lookup(name);
  for (JavaSymbol* overrideSymbol : symbols) {
    if (overrideSymbol->isKind(JavaSymbol::MTH)
        && !overrideSymbol->isStatic()) {
      MethodJavaSymbol* methodJavaSymbol = (MethodJavaSymbol*)overrideSymbol;
      if (canOverride(methodJavaSymbol)) {
        std::optional<bool> overriding = checkOverridingParameters(methodJavaSymbol,
            classType);
        if (!overriding) {
          if (!unknownFound) {
            unknownFound = true;
          }
        } else if (*overriding) {
          return methodJavaSymbol;
        }
      }
    }
  }
  if (unknownFound) {
    return Symbols::unknownMethodSymbol;
  }
  return nullptr;
}
====
cpp @@ coc=35 @@ p.18 margin total (Appendix C, TimelyResource.java in sonar-persistit): synchronized is a block with a lock guard and, like try, no structure
void addVersion(const Entry& entry, const Transaction& txn) {
  const TransactionIndex& ti = _persistit.getTransactionIndex();
  while (true) {
    try {
      {
        std::lock_guard<std::mutex> guard(_lock);
        if (frst != nullptr) {
          if (frst->getVersion() > entry.getVersion()) {
            throw RollbackException();
          }
          if (txn.isActive()) {
            for
                (Entry* e = frst; e != nullptr; e = e->getPrevious()) {
              const long version = e->getVersion();
              const long depends = ti.wwDependency(version,
                  txn.getTransactionStatus(), 0);
              if (depends == TIMED_OUT) {
                throw WWRetryException(version);
              }
              if (depends != 0
                  && depends != ABORTED) {
                throw RollbackException();
              }
            }
          }
        }
        entry.setPrevious(frst);
        frst = &entry;
        break;
      }
    } catch (const WWRetryException& re) {
      try {
        const long depends = _persistit.getTransactionIndex()
            .wwDependency(re.getVersionHandle(), txn.getTransactionStatus(),
            SharedResource::DEFAULT_MAX_WAIT_TIME);
        if (depends != 0
            && depends != ABORTED) {
          throw RollbackException();
        }
      } catch (const InterruptedException& ie) {
        throw PersistitInterruptedException(ie);
      }
    } catch (const InterruptedException& ie) {
      throw PersistitInterruptedException(ie);
    }
  }
}
====
cpp @@ coc=20 @@ p.19 margin total (Appendix C, WildcardPattern.java in SonarQube): the StringBuilder is a std::string, the ternary and the else-if chain are the page's
static std::string toRegexp(const std::string& antPattern,
    const std::string& directorySeparator) {
  const std::string escapedDirectorySeparator = "\\" + directorySeparator;
  std::string sb;
  sb.append("^");
  size_t i = startsWith(antPattern, "/") ||
      startsWith(antPattern, "\\") ? 1 : 0;
  while (i < antPattern.length()) {
    const char ch = antPattern.at(i);
    if (SPECIAL_CHARS.find(ch) != std::string::npos) {
      sb.append("\\");
      sb.push_back(ch);
    } else if (ch == '*') {
      if (i + 1 < antPattern.length()
          && antPattern.at(i + 1) == '*') {
        if (i + 2 < antPattern.length()
            && isSlash(antPattern.at(i + 2))) {
          sb.append("(?:.*");
          sb.append(escapedDirectorySeparator).append("|)");
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
      sb.push_back(ch);
    }
    i++;
  }
  sb.append("$");
  return sb;
}
"#;

#[test]
fn whitepaper_worked_examples_in_cpp() {
    common::assert_metric_table(ROWS);
}
