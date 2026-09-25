use super::*;
use std::collections::BTreeSet;

/// The roles of one path under no entry globs and no declared
/// targets — the arrange stanza both batteries share.
fn roles_at(root: &Path, name: &str) -> i64 {
    let entries = globs::compile_inclusions(root, &[], "[graph] entry_globs").expect("empty set");
    let none = Declared::gather(Path::new("."), &BTreeSet::new(), &BTreeSet::new());
    roles_of(root, name, &entries, &none)
}

/// The allow-claim role (batch-7 slice 3), table-driven — the
/// docdup discipline transplanted: only a why-bearing marker
/// claims; a bare marker and an absent file claim nothing.
#[test]
fn allow_claim_requires_the_why_tail() {
    let cases = [
        (
            "a.py",
            Some("# ce:allow(deadcode) -- loader-invoked\n"),
            true,
        ),
        ("b.py", Some("# ce:allow(deadcode)\n"), false),
        ("missing.py", None, false),
    ];
    let root = crate::testutil::scratch("dc-allow");
    for (name, text, want) in cases {
        if let Some(text) = text {
            std::fs::write(root.join(name), text).unwrap();
        }
        assert_eq!(allow_claim(&root, name), want, "{name}");
        assert_eq!(roles_at(&root, name) & ROLE_ALLOW != 0, want, "{name}");
    }
    std::fs::remove_dir_all(&root).ok();
}

/// `path ⇒ letters` per line over the roles a file's name and place
/// decide: `u` compilation unit, `n` named entry, `t` test, `d` entry
/// directory, `-` none. Plan v2.30 steps 2 and 3 (register D18): a
/// compilation unit carries the unit role by its extension alone, a
/// header and a Java class carry none (a class is reached by its name),
/// and the test-runner basenames are the test convention, read off
/// the table the convention word reads too (so a file named exactly
/// `_test.c` is nobody's test there either). Step 4: LÖVE's and
/// Shiny's files by name, Neovim's `init.lua` at the root alone, the
/// runtime directories Neovim sources a Lua file from by path (never
/// `autoload/`, which is Vim script's, nor a file of another
/// language there), an R package's script directories, busted's and
/// testthat's tests. One literal rather than rows of typed tuples:
/// rows of one shape repeat every dozen tokens under the clone gate.
const ROLES: &str = "\
src/a.c ⇒ u
src/a.h ⇒ -
src/main.cpp ⇒ un
src/a_test.cc ⇒ ut
src/b.cxx ⇒ u
src/_test.c ⇒ u
src/Main.java ⇒ n
src/FooTest.java ⇒ t
src/Foo.java ⇒ -
main.lua ⇒ n
game/conf.lua ⇒ n
init.lua ⇒ n
lua/foo/init.lua ⇒ -
plugin/foo.lua ⇒ d
after/ftplugin/lua.lua ⇒ d
lsp/clangd.lua ⇒ d
autoload/foo.lua ⇒ -
plugin/notes.md ⇒ -
spec/foo_spec.lua ⇒ t
app.R ⇒ n
inst/app/server.R ⇒ nd
data-raw/build.R ⇒ d
inst/x.lua ⇒ -
R/utils.R ⇒ -
tests/testthat/test-utils.R ⇒ t
x/test_utils.r ⇒ t";

#[test]
fn roles_read_the_file_name_and_place() {
    let letter = |c| match c {
        'u' => ROLE_UNIT,
        'n' => ROLE_ENTRY_NAMED,
        't' => ROLE_TEST,
        'd' => ROLE_ENTRY_DIR,
        _ => 0,
    };
    let mask = ROLE_UNIT | ROLE_ENTRY_NAMED | ROLE_TEST | ROLE_ENTRY_DIR;
    let root = crate::testutil::scratch("dc-name-roles");
    for row in ROLES.lines() {
        let (path, letters) = row.split_once(" ⇒ ").expect("path ⇒ letters");
        let want: i64 = letters.chars().map(letter).sum();
        assert_eq!(roles_at(&root, path) & mask, want, "{row}");
    }
    std::fs::remove_dir_all(&root).ok();
}
