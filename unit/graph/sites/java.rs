//! Java's `type_ref` pass pinned (plan v2.30 step 3): which spellings
//! name a type, and which names a file's own declarations take out.

use super::type_refs;
use crate::scan::ast;
use crate::testutil::blocks;

/// Each block: `java @@ specs @@ why` over one compilation unit — the
/// type_ref specs in document order, `|`-joined, `-` for none.
const CASES: &str = r#"
java @@ - @@ the file's own types at every depth and its type parameters reach no other file
class O<T> {
    class In {}
    interface I {}
    <U> U f(T t, In i, O.In j, I k) { return null; }
}
====
java @@ Outer|String|a.b.Outer|K|java.util.Map.Entry|K|V|a.b.@Tag C|Tag|java.util.List|X @@ a qualified type is one site cut at its type arguments, which are walked, as is a type annotation inside it
class G {
    Outer<String>.Inner a;
    a.b.Outer<K>.Inner b;
    java.util.Map.Entry<K, V> e;
    a.b.@Tag C c;
    java.util.List<? extends X> w;
    int[] n;
}
====
java @@ Logger|Fn|Util|Runnable|Util|Mode|Fn|$Gen @@ a receiver spelled like a type is a site unless a variable of the file obscures it: a field, an enum constant, a parameter, a lambda parameter
class V {
    static final Logger LOG = null;
    enum E { RED }
    void f(Fn X) {
        LOG.info();
        RED.name();
        X.go();
        Util.run();
        Runnable r = Util::g;
        Mode.FAST.name();
        Fn g = Y -> Y.go();
        $Gen.i();
    }
}
====
java @@ Cfg|a.b.C|java.lang.System|Runnable|a.b.D @@ a class named through its package in expression position; a variable heading the chain is no package
class Q {
    void f(Cfg config) {
        a.b.C.D.g();
        java.lang.System.out.println();
        config.Mode.x();
        Runnable r = a.b.D::h;
    }
}
====
java @@ A|B|Object|C|Point|Box|String @@ an annotation's name and a class literal are sites, `var` is none, a record pattern names its type
@A(B.class)
class L {
    boolean f(Object o) {
        var x = C.class;
        return o instanceof Point(int p, var q) && o instanceof Box<String>(var s);
    }
}
====
java @@ a.b.Service|a.b.Service|a.b.Impl|c.Other @@ a module's service types are sites (a provider is often named nowhere else); required and exported names are not
module m.x {
    requires java.base;
    exports a.b;
    uses a.b.Service;
    provides a.b.Service with a.b.Impl, c.Other;
}
====
java @@ - @@ `this`, `super` and an own class's `T.this` name no other file
class T {
    int x;
    void f() { this.x = 1; super.f(); T.this.x = 2; }
}
"#;

#[test]
fn every_file_yields_exactly_the_type_refs_its_row_states() {
    for (lang, [want, why], src) in blocks(CASES) {
        let tree = ast::parse_lang(src, lang).expect("a Java parse");
        assert!(
            !tree.root_node().has_error(),
            "{why}: the sample must parse"
        );
        let got: Vec<String> = type_refs(tree.root_node(), src.as_bytes())
            .into_iter()
            .map(|site| site.spec)
            .collect();
        let want: Vec<&str> = want.split('|').filter(|s| *s != "-").collect();
        assert_eq!(got, want, "{why}\n--- source ---\n{src}");
    }
}
