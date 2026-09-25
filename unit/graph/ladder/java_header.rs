//! The Java header reader pinned (plan v2.30 step 3): what the walk
//! reads off every Java file for the ladder — the declared package and
//! the imports — and where a header it cannot finish stops.

use super::{Header, Import, read};
use crate::testutil::blocks;

/// Each block: `java @@ package @@ imports @@ why` over a compilation
/// unit. `-` is the unnamed package or no import; imports are `, `-
/// joined, `static ` and a trailing `.*` spelling their two flags.
const CASES: &str = r#"
java @@ a.b @@ java.util.List, static a.b.C.m, x.y.*, static a.b.C.* @@ the four import forms, in document order
package a.b;
import java.util.List;
import static a.b.C.m;
import x.y.*;
import static a.b.C.*;
class K {}
====
java @@ a.b @@ java.util.List, static a.b.C.m @@ comments and whitespace anywhere between the tokens
/* licence */ // header
package   a . b /* c */ ;
import java . util . List ; // tail
import static
    a.b.C.m;
====
java @@ a.b @@ - @@ package-info: annotations with arguments come first, a `)` inside a string included
@Deprecated @a.b.Ann(value = "x)(", other = @In(1), c = ')') package a.b;
====
java @@ p @@ q.R @@ a text block inside an annotation hides what looks like a declaration
@Ann("""
  ) ; import x.Y;
""") package p;
import q.R;
====
java @@ - @@ a.B, static c.D.e @@ module-info: no package, the imports still read
import a.B;
import static c.D.e;
module m.x { requires java.base; }
====
java @@ - @@ - @@ a package without its `;` ends the header before it
package a.b
import c.D;
====
java @@ a @@ b.C @@ a stray token inside an import ends the header there
package a;
import b.C;
import d.*.e;
import f.G;
====
java @@ a @@ - @@ the first type declaration ends the header
package a;
class K {}
import b.C;
====
java @@ a @@ staticx.Y @@ a keyword must end at a non-identifier character; a digit cannot start a name
package a;
import staticx.Y;
import 9a.B;
====
java @@ café.ü @@ ñ.Ö, a$b._C @@ identifiers of any script, `$` and `_` included
package café.ü;
import ñ.Ö;
import a$b._C;
"#;

fn expected(package: &str, imports: &str) -> Header {
    let import = |spelled: &str| {
        let (is_static, name) = match spelled.strip_prefix("static ") {
            Some(name) => (true, name),
            None => (false, spelled),
        };
        let (name, star) = match name.strip_suffix(".*") {
            Some(name) => (name, true),
            None => (name, false),
        };
        Import {
            name: name.to_string(),
            star,
            is_static,
        }
    };
    Header {
        package: if package == "-" { "" } else { package }.to_string(),
        imports: imports
            .split(", ")
            .filter(|i| *i != "-")
            .map(import)
            .collect(),
    }
}

#[test]
fn every_header_reads_as_its_row_states() {
    for (_, cols, src) in blocks(CASES) {
        let [package, imports, why]: [&str; 3] = cols;
        assert_eq!(
            read(src),
            expected(package, imports),
            "{why}\n--- source ---\n{src}"
        );
    }
}

/// A byte-order mark is no token: the header after it still reads.
#[test]
fn a_byte_order_mark_is_skipped() {
    assert_eq!(
        read("\u{feff}package a;\nimport b.C;"),
        expected("a", "b.C")
    );
}
