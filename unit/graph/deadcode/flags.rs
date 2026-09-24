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

/// Plan v2.30 steps 2 and 3 (register D18): a compilation unit carries
/// the unit role by its extension alone, a header and a Java class
/// carry none (a class is reached by its name), `main.c` and
/// `Main.java` are named entries, and the test-runner basenames — the
/// C-family `_test` suffix, Maven Surefire's classes — are the test
/// convention, read off the table the convention word reads too (so a
/// file named exactly `_test.c` is nobody's test there either).
#[test]
fn compiled_language_roles_read_the_file_name() {
    const MASK: i64 = ROLE_UNIT | ROLE_ENTRY_NAMED | ROLE_TEST;
    let rows = [
        ("src/a.c", ROLE_UNIT),
        ("src/a.h", 0),
        ("src/main.cpp", ROLE_UNIT | ROLE_ENTRY_NAMED),
        ("src/a_test.cc", ROLE_UNIT | ROLE_TEST),
        ("src/b.cxx", ROLE_UNIT),
        ("src/_test.c", ROLE_UNIT),
        ("src/Main.java", ROLE_ENTRY_NAMED),
        ("src/FooTest.java", ROLE_TEST),
        ("src/Foo.java", 0),
    ];
    let root = crate::testutil::scratch("dc-c-roles");
    for (path, want) in rows {
        assert_eq!(roles_at(&root, path) & MASK, want, "{path}");
    }
    std::fs::remove_dir_all(&root).ok();
}
