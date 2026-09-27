//! The Java type scanner pinned (plan v2.30 step 5b): the declarations
//! a compilation unit holds — names, supertypes as written, member
//! types, lines — and the bodies that are no declaration of anything.

use super::{TypeDecl, read_types};
use crate::testutil::blocks;

/// Each block: `java @@ types @@ why` over a compilation unit. A
/// declaration spells `Name:Sup,Sup@first-last{members}` — the supers
/// `,`-joined (nothing between `:` and `@` = none), the members in the
/// same spelling `,`-joined inside the braces — top-level ones `; `-
/// joined, `-` for none.
const CASES: &str = r#"
java @@ A:B,C,D.E@1-1{N:@1-1{}} @@ generics dropped, extends and implements read, a dotted super kept, a member type nested
class A<T extends Number> extends B<T> implements C, D.E { static class N {} }
====
java @@ I:J,K@1-1{} @@ an interface's extends list holds several supers
interface I extends J, K {}
====
java @@ E:F@1-3{N:@2-2{}} @@ an enum: a constant's body is no member, a nested type is
enum E implements F { A { void f() {} }, B;
    static class N {}
}
====
java @@ R:S@1-1{Q:@1-1{}} @@ a record header is skipped whole; `record` is a keyword only before a name and `(`
record R(int record) implements S { record Q() {} }
====
java @@ A:@1-1{Bar:@1-1{}} @@ `Foo.class` is no declaration; a class inside one is
class A { Object o = Foo.class; class Bar {} }
====
java @@ A:@1-3{} @@ a local class in a method body is no member
class A { void f() {
    class Local {} }
}
====
java @@ A:@1-3{} @@ an anonymous class body is no member and holds none
class A { Runnable r = new Runnable() { public void run() {}
    class Inner {} };
}
====
java @@ A:@1-3{N:@2-2{}} @@ braces inside literals and comments are no braces
class A { String s = "{"; char c = '}'; /* { */ // }
    class N {}
}
====
java @@ A:@1-6{N:@5-5{}} @@ a text block's lines still count
class A { String s = """
    { not a brace
    }
    """;
    class N {}
}
====
java @@ Ann:@1-1{} @@ an annotation type is a type
@interface Ann { int v(); }
====
java @@ A:@1-1{}; B:A@2-2{} @@ two top-level types in one unit, each its own
class A {}
class B extends A {}
====
java @@ S:@1-1{} @@ a permits list names no super
sealed class S permits T, U {}
====
java @@ A:B,C@1-1{} @@ annotated and generic supers read to their names
class A extends @Tag B<@Tag String> implements @x.Y(v = "{") C {}
====
java @@ A:@2-2{} @@ a package annotation's braces come before any type
@Ann(v = "{") package p;
class A {}
====
java @@ O:@1-4{M:Base.Inner@2-3{D:@3-3{}}} @@ a dotted super two deep, members two deep
class O {
    class M extends Base.Inner {
        class D {} }
}
====
java @@ - @@ a header alone declares nothing
package a;
import b.C;
"#;

fn spell(decls: &[TypeDecl], sep: &str) -> String {
    let one = |d: &TypeDecl| {
        format!(
            "{}:{}@{}-{}{{{}}}",
            d.name,
            d.supers.join(","),
            d.lines.0,
            d.lines.1,
            spell(&d.members, ",")
        )
    };
    decls.iter().map(one).collect::<Vec<_>>().join(sep)
}

#[test]
fn every_unit_reads_as_its_row_states() {
    for (_, [types, why], src) in blocks(CASES) {
        let got = spell(&read_types(src), "; ");
        let got = if got.is_empty() { "-".to_string() } else { got };
        assert_eq!(got, types, "{why}\n--- source ---\n{src}");
    }
}
