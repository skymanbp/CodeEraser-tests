//! Java ladder fixtures (plan v2.30 step 3): a class is the file named
//! for it in the package its header declares — wherever that file sits
//! — so every row reads the headers the fixture read the way the walk
//! does. The rungs: an import's own file (R1), a shorter prefix — a
//! nested type, a static member, a star over a package's one directory
//! or a type's members (R2), a type reference in the JLS 6.4.1 order
//! (R3), and External for what the JDK exports (R4). Ambiguity rows MUST
//! refuse unless the declared `[graph.search_roots] java` holds exactly
//! one candidate: a row that picks one of two is the red condition.

use codeeraser::graph::ladder::{Outcome, Reason};
use codeeraser::scan::lang::Lang;

use crate::common::{Case, ext, fixture, no, ok, pkg, run_cases};

/// The habitat, `==== path` over each file. Package a.b lives in three
/// directories (src, gen, test); package p twice under two roots; s1
/// and s2 each hold an S; Default.java sits in the unnamed package.
const TREE: &str = "
==== src/a/b/Main.java
package a.b;
import x.y.Z;
import x.y.*;
import s1.*;
import s2.*;
import static x.y.Z.Inner;
import java.util.*;
class Main {}
==== src/a/b/Helper.java
package a.b; class Helper {}
==== gen/a/b/Helper.java
package a.b; class Helper {}
==== test/a/b/MainTest.java
package a.b; class MainTest {}
==== src/x/y/Z.java
package x.y; public class Z { public static class Inner {} public static void m() {} }
==== src/x/y/W.java
package x.y; class W {}
==== src/p/Q.java
package p;
import org.junit.*;
class Q {}
==== alt/p/Q.java
package p; class Q {}
==== src/s1/S.java
package s1; class S {}
==== src/s2/S.java
package s2; class S {}
==== Default.java
class Default {}
";

/// `kind @@ from @@ spec @@ outcome` per line, outcome `ok <path>
/// <rung>`, `pkg <dir> <rung>`, `ext <rung>` or `no <reason>`.
const CASES: &str = "
import @@ src/a/b/Main.java @@ x.y.Z @@ ok src/x/y/Z.java 1
import @@ src/a/b/Main.java @@ x.y.Z.Inner @@ ok src/x/y/Z.java 2
import @@ src/a/b/Main.java @@ static x.y.Z.m @@ ok src/x/y/Z.java 2
import @@ src/a/b/Main.java @@ static x.y.Z.Inner.f @@ ok src/x/y/Z.java 2
import_star @@ src/a/b/Main.java @@ x.y @@ pkg src/x/y 2
import_star @@ src/a/b/Main.java @@ x.y.Z @@ ok src/x/y/Z.java 2
import_star @@ src/a/b/Main.java @@ static x.y.Z @@ ok src/x/y/Z.java 2
import @@ src/a/b/Main.java @@ java.util.List @@ ext 4
import_star @@ src/a/b/Main.java @@ java.util @@ ext 4
import @@ src/a/b/Main.java @@ org.junit.Test @@ no out_of_scope
import @@ src/a/b/Main.java @@ Default @@ no out_of_scope
import @@ src/a/b/Main.java @@ a..b @@ no out_of_scope
import @@ src/a/b/Main.java @@ static Z @@ no out_of_scope
import @@ src/a/b/Main.java @@ p.Q @@ no ambiguous_root
import_star @@ src/a/b/Main.java @@ p @@ no ambiguous_root
import_star @@ src/a/b/Main.java @@ a.b @@ no ambiguous_root
type_ref @@ src/a/b/Main.java @@ Z @@ ok src/x/y/Z.java 3
type_ref @@ src/a/b/Main.java @@ Inner @@ ok src/x/y/Z.java 3
type_ref @@ src/a/b/Main.java @@ Z.Inner @@ ok src/x/y/Z.java 3
type_ref @@ src/a/b/Main.java @@ Helper @@ ok src/a/b/Helper.java 3
type_ref @@ test/a/b/MainTest.java @@ Helper @@ no ambiguous_root
type_ref @@ src/a/b/Main.java @@ W @@ ok src/x/y/W.java 3
type_ref @@ src/a/b/Main.java @@ S @@ no ambiguous_paths
type_ref @@ src/a/b/Main.java @@ x.y.W @@ ok src/x/y/W.java 3
type_ref @@ src/a/b/Main.java @@ x.y.@Tag W @@ ok src/x/y/W.java 3
type_ref @@ src/a/b/Main.java @@ String @@ ext 4
type_ref @@ src/a/b/Main.java @@ List @@ ext 4
type_ref @@ src/a/b/Main.java @@ java.util.Map.Entry @@ ext 4
type_ref @@ src/p/Q.java @@ Test @@ no out_of_scope
type_ref @@ src/gone/G.java @@ Z @@ no out_of_scope
";

/// The same habitat under `[graph.search_roots] java = ["src"]`: each
/// tie the declaration holds exactly one side of resolves to it.
const ROOTED: &str = "
import @@ src/a/b/Main.java @@ p.Q @@ ok src/p/Q.java 1
import_star @@ src/a/b/Main.java @@ p @@ pkg src/p 2
type_ref @@ test/a/b/MainTest.java @@ Helper @@ ok src/a/b/Helper.java 3
type_ref @@ src/a/b/Main.java @@ S @@ no ambiguous_paths
";

fn tree() -> Vec<(&'static str, &'static str)> {
    TREE.trim()
        .split("==== ")
        .filter(|block| !block.is_empty())
        .map(|block| block.split_once('\n').expect("a path over its text"))
        .collect()
}

fn rows(table: &'static str) -> Vec<Case> {
    table
        .trim()
        .lines()
        .map(|line| {
            let [kind, from, spec, want]: [&'static str; 4] = line
                .split(" @@ ")
                .collect::<Vec<_>>()
                .try_into()
                .expect("kind @@ from @@ spec @@ outcome");
            (Lang::Java, kind, from, spec, outcome(want))
        })
        .collect()
}

fn outcome(spelled: &str) -> Outcome {
    let words: Vec<&str> = spelled.split(' ').collect();
    let rung = |w: &str| w.parse().expect("a rung");
    match words.as_slice() {
        ["ok", path, r] => ok(path, rung(r)),
        ["pkg", dir, r] => pkg(dir, rung(r)),
        ["ext", r] => ext(rung(r)),
        ["no", why] => no(match *why {
            "ambiguous_root" => Reason::AmbiguousRoot,
            "ambiguous_paths" => Reason::AmbiguousPaths,
            "out_of_scope" => Reason::OutOfScope,
            _ => panic!("reason {why:?}"),
        }),
        _ => panic!("outcome {spelled:?}"),
    }
}

#[test]
fn java_rungs_resolve_and_refuse() {
    run_cases(&fixture("ladder-java", &tree()), rows(CASES));
}

#[test]
fn declared_search_roots_break_a_tie_they_hold_one_side_of() {
    let mut fx = fixture("ladder-java-rooted", &tree());
    fx.search_roots
        .insert("java".to_string(), ["src".to_string()].into());
    run_cases(&fx, rows(ROOTED));
}
