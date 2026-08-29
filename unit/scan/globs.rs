use super::*;

/// The one dialect, table-pinned: the three legacy entry_globs forms
/// (exact path, `dir/`, `*.ext`), the bare basename (any depth — the
/// gitignore no-slash rule, which is what `glob == base` meant), and
/// the forms the hand-rolled matcher could not spell (`**`, `{a,b}`,
/// `?`, a `[*]` class). A negative rides beside every positive, so a
/// matcher that says yes to everything cannot pass. One flat table:
/// a per-glob nested array repeats its own shape row after row and
/// the suite's dedup gate reads that as a clone.
#[test]
fn inclusions_speak_gitignore_and_keep_the_legacy_forms() {
    let root = Path::new(".");
    let table = [
        ("src/**/*.ts", "src/root.ts", true),
        ("src/**/*.ts", "src/n/root.ts", true),
        ("src/**/*.ts", "src/x.rs", false),
        ("src/**/*.ts", "lib/a.ts", false),
        ("src/n/root.ts", "src/n/root.ts", true),
        ("src/n/root.ts", "other/src/n/root.ts", false),
        ("main.ts", "main.ts", true),
        ("main.ts", "a/b/main.ts", true),
        ("main.ts", "main.tsx", false),
        ("cmd/", "cmd/x.go", true),
        ("cmd/", "cmd/deep/y.go", true),
        ("cmd/", "cmdx/y.go", false),
        ("cmd/", "a/cmd/z.go", false),
        ("*.ts", "a/b/c.ts", true),
        ("*.ts", "c.rs", false),
        ("{a,b}.ts", "b.ts", true),
        ("{a,b}.ts", "c.ts", false),
        ("?.ts", "a.ts", true),
        ("?.ts", "ab.ts", false),
        ("literal[*].rs", "literal*.rs", true),
        ("literal[*].rs", "literal/x.rs", false),
        ("literal[*].rs", "literalx.rs", false),
    ];
    for (glob, path, want) in table {
        let set = compile_inclusions(root, &[glob.to_string()], "test")
            .unwrap_or_else(|e| panic!("{glob}: {e}"));
        assert_eq!(selected(&set, path), want, "{glob} vs {path}");
    }
}

/// What the dialect would silently misread is refused by name, in
/// both directions, with the ce.toml key in the message.
#[test]
fn the_dialect_refuses_what_it_would_silently_misread() {
    let root = Path::new(".");
    let refusals = [
        ("!x.rs", "'!'"),
        ("#root.ts", "comment"),
        ("src\\gen\\*.rs", "escape"),
    ];
    for (glob, names) in refusals {
        let err =
            compile_inclusions(root, &[glob.to_string()], "[graph] entry_globs").expect_err(glob);
        assert!(
            err.contains(names) && err.contains("[graph] entry_globs"),
            "{glob}: {err}"
        );
        let mut b = OverrideBuilder::new(root);
        let err = add_user_glob(&mut b, glob, true, "exclude").expect_err(glob);
        assert!(
            err.contains(names) && err.contains("exclude"),
            "{glob}: {err}"
        );
    }
    // an exclusion keeps `dir/` as written: the walker prunes the
    // directory itself, so no `/**` is appended there
    let mut b = OverrideBuilder::new(root);
    add_user_glob(&mut b, "vendor/", true, "exclude").expect("dir exclusion");
    let set = b.build().expect("build");
    assert!(matches!(
        set.matched(Path::new("vendor"), true),
        ignore::Match::Ignore(_)
    ));
}
