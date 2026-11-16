//! A docstring gives the clone stream no token (TOKENIZER_REV 6): the
//! node docdup reads as one (the package's `docstring_hosts`: the
//! body's first statement, comments before it not being statements)
//! whose string holds no code leaves the stream as a comment does. A
//! string statement that is not a docstring, an implicitly concatenated
//! first statement and an f-string holding an interpolation keep the
//! tokens they had at rev 5, every other language's stream is what it
//! was, and at a file head the reported block starts at the first
//! copied statement, not at a docstring the tool never compared.

use crate::dedup_core::toks as hashes;
use crate::dedup_imports::blocks;
use codeeraser::dedup::tokens;
use codeeraser::scan::lang::Lang;
use std::path::Path;

/// Cases split by `====`; the first line of each says what it shows.
/// A line written `|...` is a docstring line: the source with the bars
/// dropped and the source without those lines give one stream.
const GONE: &str = r#"a module docstring over three lines, the code after it
|"""The module.
|
|Says what it does."""
x = compute(1, 2)
====
a function docstring, a raw one with an escape
def f(a, b):
|    r"""Adds a\tb."""
    return a + b
====
a class docstring above a method that has its own
class Meter(Base):
|    '''One meter.'''
    def read(self, rows):
|        """Sum the rows."""
        return sum(rows)
====
a module docstring, then a function whose docstring is in single quotes
|'''Top.'''
def g():
|    'one line'
    return 2
====
a coding line above the module docstring
# -*- coding: utf-8 -*-
|"""The module, under its coding line."""
import os
x = os.sep
====
a shebang above the module docstring
#!/usr/bin/env python3
|"""A script.
|
|Run it."""
y = main(1)
====
a function body whose docstring follows a comment line
def h(n):
    # the caller checked n
|    """Halve n."""
    return n // 2
====
a class body whose docstring follows two comment lines
class Gauge(Meter):
    # one per dial
    # never shared
|    """A gauge."""
    limit = 3
"#;

/// The `====` cases of a table, each as (its first line, the rest).
fn cases(table: &str) -> impl Iterator<Item = (&str, &str)> {
    table
        .split("====\n")
        .map(|case| case.split_once('\n').expect("a why line"))
}

fn py(src: &str) -> Vec<u64> {
    hashes(Lang::Python, src)
}

#[test]
fn a_docstring_leaves_the_stream_as_if_absent() {
    for (why, src) in cases(GONE) {
        let (mut with, mut without) = (String::new(), String::new());
        for line in src.lines() {
            let doc = line.strip_prefix('|');
            with.push_str(doc.unwrap_or(line));
            with.push('\n');
            if doc.is_none() {
                without.push_str(line);
                without.push('\n');
            }
        }
        assert_eq!(py(&with), py(&without), "{why}");
    }
}

/// A body whose only statement is its docstring: no token on the
/// docstring's line (`def`, the name, the two parentheses and the colon
/// are all on the first).
#[test]
fn a_docstring_only_body_gives_no_token_for_it() {
    let stream =
        tokens::stream(b"def f():\n    \"\"\"Only this.\"\"\"\n", Lang::Python).expect("stream");
    assert_eq!(stream.len(), 5, "{stream:?}");
    assert!(stream.iter().all(|t| t.end_line == 1), "{stream:?}");
}

/// The string statements that are not a docstring of the rule (cases
/// as in GONE, with no bars), with the token counts their streams had
/// at TOKENIZER_REV 5. The last two pin the skip to comments: past a
/// comment the first statement is an assignment, so the string after it
/// (the second statement) stays.
const STAYS: &str = r#"a bare string that is not the first statement
def f():
    x = 1
    "not first"
====
an attribute docstring after an assignment
class A:
    x = 1
    """About x."""
====
an implicitly concatenated first statement of a module
"a" "b"
x = 1
====
an implicitly concatenated first statement of a function
def g():
    "a" "b"
    return 1
====
an f-string first statement with an interpolation
def h(n):
    f"doc {n}"
    return n
====
a module f-string first statement with an interpolation
f"{x} rows"
y = 2
====
a function's second statement after a comment and an assignment
def k():
    # set up
    x = 1
    """Second."""
====
a module's second statement after a comment and an assignment
# -*- coding: utf-8 -*-
x = 1
"""Second."""
"#;
const STAYS_REV5: [usize; 8] = [9, 7, 5, 9, 13, 8, 9, 4];

#[test]
fn the_strings_that_are_not_docstrings_keep_their_tokens() {
    let lit = tokens::fnv1a(b"\x02LIT");
    for (i, (why, src)) in cases(STAYS).enumerate() {
        let stream = py(src);
        assert_eq!(stream.len(), STAYS_REV5[i], "{why}");
        assert!(stream.contains(&lit), "{why}: a LIT stays");
    }
}

/// Sources of the other languages, some opening on a string expression
/// where the grammar has one, with their token counts at TOKENIZER_REV
/// 5: the bump moves Python's stream only.
const OTHER_EXT: [&str; 4] = ["rs", "ts", "go", "lua"];
const OTHER_SRC: [&str; 4] = [
    "fn f() {\n    \"doc\";\n    let x = 1;\n}\n",
    "\"use strict\";\nfunction f() {\n  \"doc\";\n  return 1;\n}\n",
    "package main\n\nfunc f() string {\n\treturn \"doc\"\n}\n",
    "local s = \"doc\"\nlocal function f()\n  return s\nend\n",
];
const OTHER_REV5: [usize; 4] = [13, 13, 9, 12];

#[test]
fn every_other_language_keeps_its_stream() {
    for i in 0..OTHER_EXT.len() {
        let lang = Lang::from_path(Path::new(&format!("x.{}", OTHER_EXT[i]))).expect("known");
        assert_eq!(
            hashes(lang, OTHER_SRC[i]).len(),
            OTHER_REV5[i],
            "{}",
            OTHER_EXT[i]
        );
    }
}

const SETUP: &str = "ROOT = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(ROOT, \"lib\"))
SPEC = importlib.util.spec_from_file_location(\"helper\", os.path.join(ROOT, \"helper.py\"))
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
DATA = os.path.join(ROOT, \"data\", \"rows.json\")
LIMIT = MODULE.limit(DATA, 12)
";

/// Two module heads under `lead` that differ in docstring and imports
/// above the same setup statements: the block is the setup, from its
/// first line.
fn head_block_is_the_setup(name: &str, lead: &str) {
    let a = format!(
        "{lead}\"\"\"Module one.\n\nSays what it does.\n\"\"\"\nimport os\nimport sys\n\n{SETUP}"
    );
    let b = format!(
        "{lead}\"\"\"Another module, two lines\nof text.\"\"\"\nfrom pathlib import Path\nimport importlib.util\nimport json\n\n{SETUP}"
    );
    let found = blocks(name, &[("a.py", &a), ("b.py", &b)]);
    assert_eq!(found.len(), 1, "{found:?}");
    let first = |text: &str| {
        text.lines()
            .position(|l| l.starts_with("ROOT"))
            .expect("ROOT")
            + 1
    };
    assert_eq!(
        (found[0].a_start, found[0].b_start),
        (first(&a), first(&b)),
        "{found:?}"
    );
    assert_eq!(found[0].tokens, py(SETUP).len(), "{found:?}");
}

#[test]
fn a_block_under_different_docstrings_starts_at_the_first_copied_statement() {
    head_block_is_the_setup("docstring-head", "");
}

/// The same with a coding line above both docstrings (the shape of the
/// trigger pair of this change).
#[test]
fn a_block_under_a_coding_line_and_docstrings_starts_at_the_first_copied_statement() {
    head_block_is_the_setup("docstring-coding-head", "# -*- coding: utf-8 -*-\n");
}

const CLASS: &str = "class Meter(Base):
    def read(self, rows):
        total = 0
        for row in rows:
            total += row.value * 2
        return total

    def reset(self):
        self.count = 0
        self.total = 0
        self.seen = []
";

/// The same class in two files, a class docstring on one side only: the
/// body alone is under the 50-token line, the class with its header is
/// over it, so the docstring used to split the only run long enough.
#[test]
fn a_class_with_a_docstring_on_one_side_is_a_clone() {
    let whole = tokens::stream(CLASS.as_bytes(), Lang::Python).expect("stream");
    let header = whole.iter().filter(|t| t.start_line == 1).count();
    assert!(
        whole.len() - header < 50 && whole.len() >= 50,
        "{}",
        whole.len()
    );
    let with = CLASS.replacen(":\n", ":\n    \"\"\"One side only.\"\"\"\n", 1);
    let found = blocks("docstring-class", &[("a.py", &with), ("b.py", CLASS)]);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].tokens, whole.len(), "{found:?}");
    assert_eq!((found[0].a_start, found[0].b_start), (1, 1), "{found:?}");
}
