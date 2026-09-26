//! Java ladder fixtures (plan v2.30 step 3): a class is the file named
//! for it in the package its header declares — wherever that file sits
//! — so every row reads the headers the fixture read the way the walk
//! does. The rungs: an import's own file (R1), a shorter prefix — a
//! nested type, a static member, a star over a package's one directory
//! or a type's members (R2), a type reference in the JLS 6.4.1 order
//! (R3), and External for what the JDK exports (R4). Ambiguity rows MUST
//! refuse unless the declared `[graph.search_roots] java` holds exactly
//! one candidate: a row that picks one of two is the red condition.

use codeeraser::scan::lang::Lang;

use crate::common::text_ladder;

/// The habitat, `==== path` over each file, then the rows
/// (common::text_ladder). Package a.b lives in three directories (src,
/// gen, test); package p twice under two roots; s1 and s2 each hold an
/// S; Default.java sits in the unnamed package. Under
/// `[graph.search_roots] java = ["src"]` (the `@rooted` rows) each tie
/// the declaration holds exactly one side of resolves to it. The
/// standard layout (java_sets.rs, the second `@cases` run): package
/// o.nodes is split between the main and the test source set. A main
/// file sees main code only, so its star import has one directory to
/// answer and a test-only class is out of its reach; a test file sees
/// both and answers its own part. A name the file declares itself —
/// its own nested type, itself — is own_unit, never an edge back to
/// the file.
const LADDER: &str = "
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
==== @cases
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
==== @rooted src
import @@ src/a/b/Main.java @@ p.Q @@ ok src/p/Q.java 1
import_star @@ src/a/b/Main.java @@ p @@ pkg src/p 2
type_ref @@ test/a/b/MainTest.java @@ Helper @@ ok src/a/b/Helper.java 3
type_ref @@ src/a/b/Main.java @@ S @@ no ambiguous_paths
==== src/main/java/o/nodes/Node.java
package o.nodes; public class Node {}
==== src/test/java/o/nodes/NodeTest.java
package o.nodes; class NodeTest {}
==== src/main/java/o/parser/Parser.java
package o.parser;
import o.nodes.*;
public class Parser { static class Inner { static int X; } }
==== src/test/java/o/parser/ParserTest.java
package o.parser;
import o.nodes.*;
class ParserTest {}
==== src/test/java/o/parser/Fixture.java
package o.parser; class Fixture {}
==== @cases
import_star @@ src/main/java/o/parser/Parser.java @@ o.nodes @@ pkg src/main/java/o/nodes 2
import_star @@ src/test/java/o/parser/ParserTest.java @@ o.nodes @@ pkg src/test/java/o/nodes 2
import @@ src/main/java/o/parser/Parser.java @@ o.nodes.NodeTest @@ no out_of_scope
import @@ src/test/java/o/parser/ParserTest.java @@ o.nodes.NodeTest @@ ok src/test/java/o/nodes/NodeTest.java 1
type_ref @@ src/main/java/o/parser/Parser.java @@ Fixture @@ no out_of_scope
type_ref @@ src/test/java/o/parser/ParserTest.java @@ Fixture @@ ok src/test/java/o/parser/Fixture.java 3
type_ref @@ src/test/java/o/parser/ParserTest.java @@ Parser @@ ok src/main/java/o/parser/Parser.java 3
import_star @@ src/main/java/o/parser/Parser.java @@ static o.parser.Parser.Inner @@ no own_unit
import @@ src/main/java/o/parser/Parser.java @@ o.parser.Parser @@ no own_unit
type_ref @@ src/main/java/o/parser/Parser.java @@ o.parser.Parser @@ no own_unit
";

#[test]
fn java_rungs_resolve_and_refuse() {
    text_ladder(Lang::Java, LADDER);
}
