use super::*;

/// The three flags in both spellings, in invocation order; `-include`
/// and a bare `-c` name no directory.
#[test]
fn include_dirs_read_both_spellings_in_order() {
    let argv: Vec<String> = "cc -I/a -I /b -iquote /c -isystem/d -include x.h -c f.c"
        .split(' ')
        .map(String::from)
        .collect();
    let dirs: Vec<&str> = include_dirs(&argv).collect();
    assert_eq!(dirs, ["/a", "/b", "/c", "/d"]);
}

/// A database path lands under the root as the walk spells it:
/// absolute and inside; relative to an absolute directory inside;
/// relative to a relative (root-anchored) directory; outside the root
/// — absolute, or relative to a directory outside — is None; a drive
/// letter compares without case and backslashes read as separators.
/// (The roots are sample spellings, no machine's directory: the
/// comparison is lexical.)
#[test]
fn paths_relativize_lexically_against_the_root() {
    let root = "/opt/proj";
    let rows = [
        ("/opt/proj/build", "/opt/proj/src/a.c", Some("src/a.c")),
        ("/opt/proj/build", "../src/a.c", Some("src/a.c")),
        ("build", "../src/a.c", Some("src/a.c")),
        ("/opt/proj/build", "/usr/include", None),
        ("/opt/project/build", "a.c", None),
    ];
    for (dir, path, want) in rows {
        assert_eq!(relativize(root, dir, path).as_deref(), want, "{dir} {path}");
    }
    let windows = relativize("D:/proj", "d:\\proj\\build", "..\\src\\a.c");
    assert_eq!(windows.as_deref(), Some("src/a.c"));
}

/// One file, two entries: `arguments` with an absolute attached `-I`,
/// and `command` with a relative file — the entry outside the root is
/// dropped whole.
#[test]
fn a_database_yields_its_in_tree_units_with_their_include_dirs() {
    let root = crate::testutil::scratch("compdb");
    let r = root.to_string_lossy().replace('\\', "/");
    let json = format!(
        "[{{\"directory\":\"{r}/build\",\"file\":\"../src/a.c\",\"arguments\":[\"cc\",\"-I{r}/inc\",\"-c\",\"../src/a.c\"]}},\
         {{\"directory\":\"/elsewhere\",\"file\":\"/elsewhere/b.c\",\"command\":\"cc b.c\"}}]"
    );
    std::fs::write(root.join("cc.json"), json).expect("write");
    let db = parse(&root, "cc.json").expect("parses");
    assert_eq!(
        db.entries,
        [("src/a.c".to_string(), vec!["inc".to_string()])]
    );
    std::fs::remove_dir_all(&root).ok();
}
