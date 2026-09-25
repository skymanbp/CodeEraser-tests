//! The Java ladder's reading of a name pinned (plan v2.30 step 3): a
//! type annotation is no part of the name it annotates (JLS 9.7.4), and
//! its argument list goes with it — strings, text blocks, comments and
//! nested groups inside read as the header lexer reads them.

use super::unannotated;

/// `written => read` per line; the read form is whitespace-free, as the
/// ladder's own collapse after this step leaves it. A name cut short —
/// a spec is its line's first line, so a text block opened in an
/// annotation's arguments never closes — keeps its trailing dot, which
/// the ladder then refuses by name.
const CASES: &str = r#"
a.b.C => a.b.C
a.b.@Tag C => a.b.C
a.b.@x.y.Tag C => a.b.C
a.b.@Tag(x = (1), s = ")") C => a.b.C
a.b.@A @B(1) C => a.b.C
a.b.@Tag /* note */ C => a.b.C
a.b.@Tag(""" => a.b.
a.b.@Tag => a.b.
"#;

#[test]
fn a_type_annotation_is_no_part_of_the_name() {
    for line in CASES.trim().lines() {
        let (written, read) = line.split_once(" => ").expect("written => read");
        let got: String = unannotated(written).split_whitespace().collect();
        assert_eq!(got, read, "{written}");
    }
}
