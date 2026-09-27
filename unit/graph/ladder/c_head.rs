use super::*;
use crate::graph::sites::detect;
use crate::scan::lang::Lang;
use crate::testutil::unmarked;

/// A table row: `source => spec|spec` (tilde: none). A leading `+`
/// marks a source only the C++ grammar parses (raw strings:
/// tree-sitter-c has none); a leading `!` marks a row outside the twin
/// leg — ill-formed source (a directive off its line's start, an
/// unterminated literal or comment, an operand no header-name can be,
/// a backslash inside a q-char-sequence, undefined by C17 §6.4.7) or a
/// well-formed spelling the detector's lexer cannot read (a comment
/// between `#` and `include`, a line splice inside the directive name,
/// a sixteen-character raw delimiter) — where the reader follows the
/// standard and the detector answers as its grammar will.
fn row(line: &str) -> (Vec<Lang>, String, Vec<&str>) {
    let (text, want) = line.split_once(" => ").expect("case separator");
    let (twins, text) = match text.strip_prefix(['!', '+']) {
        Some(rest) if text.starts_with('!') => (vec![], rest),
        Some(rest) => (vec![Lang::Cpp], rest),
        None => (vec![Lang::C, Lang::Cpp], text),
    };
    let want = want.split('|').filter(|spec| *spec != "~").collect();
    (twins, unmarked(text), want)
}

/// Each row is one source and its pipe-separated ordered includes.
fn check(table: &str) {
    for line in table.trim_matches('\n').lines() {
        let (_, text, want) = row(line);
        assert_eq!(read(&text), want, "{line}");
    }
}

/// The measured tree-sitter acceptance file, including repeated and inactive sites.
const ACCEPTANCE: &str = r##"#include "a.h"
#if 0
#include "dead.h"
#endif
/* #include "incomment.h" */
// #include "linecomment.h"
#include_next <next.h>
#import "imported.h"
#include HEADER
#include <sys/types.h>
#  include   "spaced.h"
#include "a.h" // twice
#include "back\slash.h"
const char *s = "#include \"instring.h\"";
"##;

/// Logical line starts, phase-two splicing, and header-name delimiters are lexical.
const LOGICAL_LINES: &str = r##"
#include "a.h"{CR}{LF}#include <b.h>{CR}{LF} => a.h|<b.h>
{BOM}#include "bom.h"{LF} => bom.h
  {TAB}#  include "blank.h"{LF} => blank.h
!int x; #include "n.h"{LF} => ~
!int x = 1 #include "n.h"{LF} => ~
void f(void) {{LF}#include "body.h"{LF}}{LF} => body.h
#define INC #include "in_define.h"{LF}#include "yes.h"{LF} => yes.h
!#ifdef X /* c */ #include "in_ifdef.h"{LF}#include "yes.h"{LF}#endif{LF} => yes.h
#include "a.h" /* trailing */{LF} => a.h
#include \{LF} "z.h"{LF} => z.h
!#inc\{CR}{LF}lude "joined.h"{LF} => joined.h
// hidden\{LF}#include "n.h"{LF}#include "yes.h"{LF} => yes.h
/*{LF}#include "x.h"{LF}*/{LF}#include "yes.h"{LF} => yes.h
#include/**/"comments.h"{LF} => comments.h
!/* prefix */ #/**/include/**/"comments.h"{LF} => comments.h
int x; /*{LF}*/ #include "after.h"{LF} => after.h
#include_next "no.h"{LF}#include2 "no.h"{LF}#import <no.h>{LF} => ~
#include HEADER_2(x){LF}#include <sys/types.h>{LF} => HEADER_2(x)|<sys/types.h>
!#include "back\"{LF}#include "next.h"{LF} => back\|next.h
!#include "open{LF}#include <later.h>{LF} => ~
!#include 123{LF}#include "yes.h"{LF} => yes.h
!#include /* open{LF} => ~
 => ~
"##;

/// Literals hide apparent directives, including all raw prefixes and delimiter limits.
const LITERALS: &str = r##"
const char *s = "#include \"instring.h\"";{LF} => ~
'"'; '#'; '\'';{LF}#include "after.h"{LF} => after.h
+R"({LF}#include "y.h"{LF})"{LF}#include "yes.h"{LF} => yes.h
+u8R"tag({LF}#include "y.h"{LF})tag"{LF}#include "yes.h"{LF} => yes.h
!uR"x({LF}#include "y.h"{LF})x" #include "no.h"{LF} => ~
+UR"x({LF}#include "y.h"{LF})x"{LF}#include "yes.h"{LF} => yes.h
+LR"x({LF}#include "y.h"{LF})x"{LF}#include <yes.h>{LF} => <yes.h>
!LR"abcdefghijklmnop({LF}#include "y.h"{LF})abcdefghijklmnop"{LF}#include <yes.h>{LF} => <yes.h>
+R"tag({LF})other"{LF}#include "y.h"{LF})tag"{LF}#include "yes.h"{LF} => yes.h
!R"({LF}#include "unterminated.h"{LF} => ~
!"quoted\" text{LF}#include \"hidden.h\"";{LF}#include "yes.h"{LF} => yes.h
"##;

#[test]
fn detector_acceptance_file() {
    assert_eq!(
        read(ACCEPTANCE).join("|"),
        "a.h|dead.h|HEADER|<sys/types.h>|spaced.h|a.h|back\\slash.h"
    );
}

#[test]
fn logical_lines_and_header_spellings() {
    check(LOGICAL_LINES);
}

#[test]
fn strings_characters_and_raw_literals_are_opaque() {
    check(LITERALS);
}

/// On every well-formed source above the reader answers exactly what
/// the site detector answers, under each grammar that parses the row
/// (a `.h` parses as C++; the closure walks both) — the include closure
/// attributes headers along the sites the graph will draw, no more and
/// no fewer.
#[test]
fn reads_what_the_detector_detects() {
    let mut rows = vec![(vec![Lang::C, Lang::Cpp], ACCEPTANCE.to_string())];
    for table in [LOGICAL_LINES, LITERALS] {
        for line in table.trim_matches('\n').lines() {
            let (twins, text, _) = row(line);
            rows.push((twins, text));
        }
    }
    let mut apart = Vec::new();
    for (twins, text) in &rows {
        for lang in twins {
            let detected: Vec<String> = detect(text, *lang)
                .into_iter()
                .filter(|s| s.kind == "include")
                .map(|s| s.spec)
                .collect();
            if read(text) != detected {
                let reader = read(text);
                apart.push(format!(
                    "{lang:?} {text:?}: reader {reader:?}, detector {detected:?}"
                ));
            }
        }
    }
    assert!(apart.is_empty(), "{}", apart.join("\n"));
}
