//! The Java header reader pinned (plan v2.30 step 3): what the walk
//! reads off every Java file for the ladder — the declared package and
//! the imports, each with the line of its keyword (step 5b) — and
//! where a header it cannot finish stops. The unit's type declarations
//! are the child java_types.rs's own battery.

use super::{Header, read};
use crate::testutil::blocks;

/// Each block: `java @@ package @@ imports @@ why` over a compilation
/// unit. `-` is the unnamed package or no import; imports are `, `-
/// joined, `static ` and a trailing `.*` spelling their two flags and
/// `@line` the line of the `import` keyword.
const CASES: &str = r#"
java @@ a.b @@ java.util.List@2, static a.b.C.m@3, x.y.*@4, static a.b.C.*@5 @@ the four import forms, in document order
package a.b;
import java.util.List;
import static a.b.C.m;
import x.y.*;
import static a.b.C.*;
class K {}
====
java @@ a.b @@ java.util.List@3, static a.b.C.m@4 @@ comments and whitespace anywhere between the tokens; a folded import keeps its keyword's line
/* licence */ // header
package   a . b /* c */ ;
import java . util . List ; // tail
import static
    a.b.C.m;
====
java @@ a.b @@ - @@ package-info: annotations with arguments come first, a `)` inside a string included
@Deprecated @a.b.Ann(value = "x)(", other = @In(1), c = ')') package a.b;
====
java @@ p @@ q.R@4 @@ a text block inside an annotation hides what looks like a declaration
@Ann("""
  ) ; import x.Y;
""") package p;
import q.R;
====
java @@ - @@ a.B@1, static c.D.e@2 @@ module-info: no package, the imports still read
import a.B;
import static c.D.e;
module m.x { requires java.base; }
====
java @@ - @@ - @@ a package without its `;` ends the header before it
package a.b
import c.D;
====
java @@ a @@ b.C@2 @@ a stray token inside an import ends the header there
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
java @@ a @@ staticx.Y@2 @@ a keyword must end at a non-identifier character; a digit cannot start a name
package a;
import staticx.Y;
import 9a.B;
====
java @@ café.ü @@ ñ.Ö@2, a$b._C@3 @@ identifiers of any script, `$` and `_` included
package café.ü;
import ñ.Ö;
import a$b._C;
"#;

/// A header's package and imports as the table spells them.
fn spelled(header: &Header) -> (String, String) {
    let imports: Vec<String> = header
        .imports
        .iter()
        .map(|i| {
            format!(
                "{}{}{}@{}",
                if i.is_static { "static " } else { "" },
                i.name,
                if i.star { ".*" } else { "" },
                i.line
            )
        })
        .collect();
    let dash = |s: String| if s.is_empty() { "-".to_string() } else { s };
    (dash(header.package.clone()), dash(imports.join(", ")))
}

#[test]
fn every_header_reads_as_its_row_states() {
    for (_, cols, src) in blocks(CASES) {
        let [package, imports, why]: [&str; 3] = cols;
        let got = spelled(&read(src));
        assert_eq!(
            (got.0.as_str(), got.1.as_str()),
            (package, imports),
            "{why}\n--- source ---\n{src}"
        );
    }
}

/// A byte-order mark is no token: the header after it still reads.
#[test]
fn a_byte_order_mark_is_skipped() {
    let got = spelled(&read("\u{feff}package a;\nimport b.C;"));
    assert_eq!((got.0.as_str(), got.1.as_str()), ("a", "b.C@2"));
}
