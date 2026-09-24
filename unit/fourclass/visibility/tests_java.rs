//! Java legs (plan v2.30 step 3; design booklet §7 row Java): the
//! spelled access, the access a holder implies, and the enclosing
//! chain, through the extractor's own root. Every shape here was
//! probed on tree-sitter-java 0.23.5 before the reading was written
//! (the scripts/tsprobe transcripts). Two text tables through the
//! shared runner (tests.rs run_tables), as the C family's are.

use super::super::tests::run_tables;

/// `java <source> ⇒ <name:letters …>` per line, newlines spelled `\n`,
/// `#` lines commentary — measured over the functions the extractor
/// sees.
const FNS: &str = r#"
# The spelled access on a public top-level class's members: package
# access (nothing spelled) and `protected` are exported and restricted.
java public class T {\n  public void pub() {}\n  protected void prot() {}\n  void pkg() {}\n  private void priv() {}\n} ⇒ pub:ES prot:ESR pkg:ESR priv:-
# An interface's member is public unless spelled private, an enum's
# constructor is private, and a type that is not public closes the
# scope of what it holds, whatever the member itself spells.
java public class T {\n  static class Nest { public void n() {} }\n  public interface I { default void d() {} private void ip() {} }\n  interface J { default void j() {} }\n  public enum E { A; E() {} public void em() {} }\n} ⇒ n:E d:ES ip:- j:E E:- em:ES
# A body closes the scope: a local class's method, an anonymous class's
# method and an enum constant's body keep their own bit 0 only; an
# abstract signature is no unit at all.
java public class T {\n  void host() { class L { public void lm() {} } Runnable r = new Runnable() { public void run() {} }; }\n  public enum E { A { void body() {} }; }\n  public abstract void sig();\n} ⇒ host:ESR lm:E run:E body:ER
# A package-access top-level type closes its members' scope; a record's
# compact constructor reads its own spelled access.
java class P { public void pp() {} }\npublic record R(int x) { public R {} } ⇒ pp:E R:ES
"#;

/// The same line shape over EVERY unit the register keys — the five
/// type declarations beside the functions, spelled as stored
/// (`name/arity` for a function).
const ALL: &str = r#"
# Each named type form is a unit read by the same rule; a local class
# keeps its package bit 0 and loses the scope bit; an anonymous class
# declares no type at all. (Units of one line read in key order.)
java public class T {\n  interface I {}\n  public enum E { A }\n  protected record R(int x) {}\n  private @interface Ann {}\n  void f() { class L {} Object o = new Object() {}; }\n} ⇒ T:ES I:ESR E:ES R:ESR Ann:- L:ER f/0:ESR
"#;

#[test]
fn java_words_follow_the_tables() {
    run_tables(FNS, ALL);
}
