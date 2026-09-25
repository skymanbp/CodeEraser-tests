//! K20's same-file legs, one line per case: `path name ⇒ +` when the
//! file's own exception regions spell the name, `⇒ -` when they do
//! not — the source follows after `|`, its newlines spelled `\n` so
//! the whole table is ONE text literal (an array of case strings is
//! the token shape the clone gate pairs across test files). The
//! negative cases are the rule's edges: plain comments and prose, the
//! code inside a TS `${…}`, a `... ` line whose indent never opened a
//! doctest, a `>>>` without its trailing space, a doc run's prose
//! outside any fence, a `////` non-doc comment, a doc run broken by a
//! blank line, a Doxygen command or a `{@code}` in a plain comment, a
//! `<pref>` tag that is not `<pre>`, an indented line that wraps a
//! paragraph, a run whose every line shares one indent, an indented
//! haddock line, the text a Lua or R doc run holds outside its
//! `@usage` / `@examples` section, a plain R comment.

use super::SelfText;
use crate::testutil::scratch;

const CASES: &str = r#"
# Go: template actions inside strings; a plain comment does not count
a.go M ⇒ + | package p\nvar t = "{{.M}} and {{ .N | html }}"\nfunc (T) M() {}\n
a.go N ⇒ + | package p\nvar t = "{{ .N | html }}"\n
a.go Eq ⇒ - | package p\n// Eq is here\nfunc Eq() {}\n
a.go Eq ⇒ - | package p\nvar s = "Eq called"\n
# TS: strings and templates, minus substitution code, plus the
# strings nested in it at any depth
a.ts Foo ⇒ - | const x = `see ${Foo} now`;\nexport function Foo() {}\n
a.ts KeyName ⇒ + | const x = `${reg("KeyName")}`;\nexport function KeyName() {}\n
a.ts ZodMiniEmail ⇒ + | const x = `${a(b(c('ZodMiniEmail')))}`;\n
a.ts x ⇒ - | const t = `a${ `b${x}c` }d`;\n
a.ts b ⇒ + | const t = `a${ `b${x}c` }d`;\n
a.ts d ⇒ + | const t = `a${ `b${x}c` }d`;\n
a.ts plain ⇒ + | invoke('plain');\n
a.ts Bar ⇒ - | // Bar in a comment\nexport function Bar() {}\n
# Python: doctest lines; PS2 gate on the same indent inside the same
# string; prose never; `>>>` needs its trailing space
a.py from_key_val_list ⇒ + | def f():\n    """\n        >>> from_key_val_list([])\n    """\n
a.py cont ⇒ + | def f():\n    """\n      >>> x = 1\n      ... cont(x)\n    """\n
a.py cont ⇒ - | def f():\n    """\n      >>> x = 1\n    ... cont(x)\n    """\n
a.py cont ⇒ - | def f():\n    """\n    ... cont or POST:\n    """\n
a.py far ⇒ + | def f():\n    """\n    >>> x = 1\n    2\n    prose\n    ... far(x)\n    """\n
a.py blockA ⇒ + | def f():\n    """\n    >>> blockA()\n        >>> blockB()\n        ... blockC()\n    ... blockD()\n    """\n
a.py blockC ⇒ + | def f():\n    """\n    >>> blockA()\n        >>> blockB()\n        ... blockC()\n    ... blockD()\n    """\n
a.py blockD ⇒ + | def f():\n    """\n    >>> blockA()\n        >>> blockB()\n        ... blockC()\n    ... blockD()\n    """\n
a.py nospace ⇒ - | def f():\n    """\n    >>>nospace()\n    """\n
a.py prose ⇒ - | def f():\n    """prose here"""\n
a.py lit ⇒ + | x = 'lit'\nx = '>>> lit()'\n
# Rust: macro bodies, fenced code in a doc run; prose in the run, a
# `////` line, a plain `//` and a run split by a blank line do not
a.rs helper ⇒ + | macro_rules! m { () => { helper() }; }\n
a.rs fenced ⇒ + | /// Example:\n/// ```\n/// fenced();\n/// ```\npub fn fenced() {}\n
a.rs inner ⇒ + | //! ```\n//! inner();\n//! ```\n
a.rs blocky ⇒ + | /** ```\n * blocky();\n * ``` */\npub fn blocky() {}\n
a.rs prose ⇒ - | /// prose() mentioned in words\npub fn prose() {}\n
a.rs quad ⇒ - | //// ```\n//// quad();\n//// ```\n
a.rs plain ⇒ - | // ```\n// plain();\n// ```\n
a.rs split ⇒ - | /// ```\n\n/// split();\n/// ```\n
# Rust: an indented block is a doctest too (rustdoc compiles it) —
# after a blank line or a heading; a wrapped paragraph line and a run
# whose one indent rustdoc strips are prose
a.rs indented ⇒ + | /// Example:\n///\n///     indented();\npub fn indented() {}\n
a.rs head ⇒ + | /// # Examples\n///     head();\npub fn head() {}\n
a.rs wrapped ⇒ - | /// Wraps a line\n///     wrapped() onto the next\npub fn wrapped() {}\n
a.rs flat ⇒ - | ///     flat();\npub fn flat() {}\n
# Haskell: haddock runs with fences, `@` blocks and bird tracks; an
# ordinary comment does not count
a.hs fenced ⇒ + | -- | Example:\n--\n-- ```\n-- fenced 1\n-- ```\nfenced :: Int -> Int\nfenced = id\n
a.hs atb ⇒ + | -- | Use it:\n--\n-- @\n-- atb 1\n-- @\natb = id\n
a.hs bird ⇒ + | -- |\n-- > bird 1\nbird = id\n
a.hs plain ⇒ - | -- plain 1\nplain = id\n
a.hs prose ⇒ - | -- | prose 1 in words\nprose = id\n
a.hs indent ⇒ - | -- | Example:\n--\n--     indent 1\nindent = id\n
# C / C++: every string literal, a concatenated string's pieces and a
# raw string included; a comment does not count
a.c helper ⇒ + | void f(void) { dl("helper"); }\n
a.c cat ⇒ + | const char *s = "a" "cat";\n
a.cpp raw ⇒ + | auto s = R"(raw)";\n
a.c plain ⇒ - | /* plain */\nvoid plain(void) {}\n
# C / C++ Doxygen: `@code` and `\code` blocks in a doc run; the same
# command in a plain comment and a doc run's prose do not
a.cpp dox ⇒ + | /// @code\n/// dox();\n/// @endcode\nvoid dox();\n
a.c bs ⇒ + | /**\n * \code\n * bs();\n * \endcode\n */\nvoid bs(void);\n
a.c nod ⇒ - | /* @code\n nod();\n @endcode */\nvoid nod(void);\n
a.cpp prose ⇒ - | /// prose() in words\nvoid prose();\n
a.cpp bg ⇒ + | /**\n * Example:\n *\n *     bg(1);\n */\nvoid bg(int);\n
a.c wrap ⇒ - | /**\n * @param x the x, which\n *     wrap() mentions\n */\nvoid wrap(int x);\n
# Java: every string literal, a text block included; Javadoc `<pre>`
# (either case), `{@code}` and `{@snippet}` spans, a Markdown doc
# comment's fences; prose, a plain comment and `<pref>` do not
a.java run ⇒ + | class A { void f() throws Exception { A.class.getMethod("run"); } }\n
a.java blk ⇒ + | class A { String s = """\n    blk\n    """; }\n
a.java pre ⇒ + | /**\n * <pre>\n * pre();\n * </pre>\n */\nclass A {}\n
a.java up ⇒ + | /** <PRE class="x">up();</PRE> */\nclass A {}\n
a.java code ⇒ + | /** Use {@code code(new int[] {1})} here. */\nclass A {}\n
a.java snip ⇒ + | /**\n * {@snippet :\n * snip();\n * }\n */\nclass A {}\n
a.java md ⇒ + | /// ```\n/// md();\n/// ```\nclass A {}\n
a.java mdind ⇒ + | /// Example:\n///\n///     mdind();\nclass A {}\n
a.java ind ⇒ + | /**\n * Example:\n *\n *     ind();\n */\nclass A {}\n
a.java prose ⇒ - | /** prose() in words */\nclass A {}\n
a.java plain ⇒ - | /* {@code plain()} */\nclass A {}\n
a.java pref ⇒ - | /** <pref>pref()</pref> */\nclass A {}\n
# Lua / R: every string (a Lua long string and an R raw one
# included); LDoc's `@usage` section — on its tag line or below it,
# in a `---` run or a long comment, to the next tag — and roxygen's
# `@examples` and `@examplesIf`; prose, the text past the section's
# next tag and a plain R comment do not
a.lua dyn ⇒ + | local f = _G["dyn"]\n
a.lua long ⇒ + | local s = [==[long()]==]\n
a.lua plain ⇒ - | -- plain()\nlocal function plain() end\n
a.lua use ⇒ + | --- Summary.\n-- @usage use(1)\nfunction M.use() end\n
a.lua below ⇒ + | --- Summary.\n-- @usage\n--   below(1)\n-- @param x\nfunction M.below(x) end\n
a.lua after ⇒ - | --- Summary.\n-- @usage x()\n-- @param after the thing\nfunction M.after() end\n
a.lua blk ⇒ + | --[[-- Summary.\n@usage blk()\n]]\nfunction M.blk() end\n
a.lua prose ⇒ - | --- prose() in words\nfunction M.prose() end\n
a.R get ⇒ + | x <- get("get")\n
a.R raw ⇒ + | x <- r"(raw)"\n
a.R ex ⇒ + | #' Title\n#' @examples\n#' ex(1)\n#' @export\nex <- function(x) x\n
a.R cond ⇒ + | #' @examplesIf interactive()\n#' cond(1)\ncond <- function(x) x\n
a.R tagged ⇒ - | #' @examples\n#' y()\n#' @param tagged the x\nf <- function(x) x\n
a.R plain ⇒ - | # @examples\n# plain()\nplain <- function() 1\n
a.R prose ⇒ - | #' prose() in words\nprose <- function() 1\n
"#;

/// A stray byte decodes lossily, as the index side decoded the file
/// whose declarations are being judged: the fenced block still
/// counts, and a vanished file still reads as nothing.
#[test]
fn a_stray_byte_does_not_void_the_files_own_regions() {
    let dir = scratch("selfref-bytes");
    std::fs::write(
        dir.join("a.rs"),
        b"// \xff\n/// ```\n/// stray();\n/// ```\npub fn stray() {}\n",
    )
    .expect("a.rs");
    assert!(SelfText::read(&dir, "a.rs").mentions("stray"));
    assert!(!SelfText::read(&dir, "gone.rs").mentions("stray"));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn only_the_named_regions_count_as_self_mentions() {
    let dir = scratch("selfref");
    for case in CASES
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let (head, src) = case.split_once(" | ").expect("case has ` | `");
        let (input, want) = head.rsplit_once(" ⇒ ").expect("case has ` ⇒ `");
        let (rel, name) = input.split_once(' ').expect("path then name");
        std::fs::write(dir.join(rel), src.replace("\\n", "\n")).expect(rel);
        let mut file = SelfText::read(&dir, rel);
        assert_eq!(file.mentions(name), want == "+", "{case:?}");
    }
    std::fs::remove_dir_all(&dir).ok();
}
