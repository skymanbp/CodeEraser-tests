use super::*;

fn live(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_string()).collect()
}

/// One scratch tree holding every probe shape: a root `build/`
/// database, a nested JSON database, a flags file, a response file a
/// database names, and a Markdown file no probe climbs from.
fn tree(tag: &str) -> std::path::PathBuf {
    let root = crate::testutil::scratch(tag);
    let db = "[{\"directory\": \"src\", \"file\": \"a.c\", \
              \"arguments\": [\"cc\", \"@flags.rsp\", \"-c\", \"a.c\"]}]";
    crate::testutil::write_tree(
        &root,
        &[
            ("src/a.c", "int x;\n"),
            ("src/flags.rsp", "-I inc\n"),
            ("build/compile_commands.json", db),
            ("lib/x/compile_commands.json", "[]"),
            ("lib/x/y/b.cpp", "\n"),
            ("src/compile_flags.txt", "-Iinc\n"),
            ("other/README.md", "# x\n"),
        ],
    );
    root
}

/// clangd's probe: each C-family file's directory and its ancestors,
/// the three names in clangd's order per directory; a tree whose
/// files are no C climbs from nowhere, whatever the disk holds.
#[test]
fn probes_climb_from_each_c_file_in_clangd_order() {
    let root = tree("compdb-find");
    let files = live(&["src/a.c", "lib/x/y/b.cpp", "other/README.md"]);
    let dirs: Vec<String> = probe_dirs(files.iter()).into_iter().collect();
    assert_eq!(dirs, ["", "lib", "lib/x", "lib/x/y", "src"]);
    let got: Vec<(String, usize, String)> = found(&root, files.iter())
        .into_iter()
        .map(|f| (f.dir, f.probe, f.rel))
        .collect();
    let want = [
        ("", 1, "build/compile_commands.json"),
        ("lib/x", 0, "lib/x/compile_commands.json"),
        ("src", 2, "src/compile_flags.txt"),
    ];
    let want: Vec<(String, usize, String)> = want
        .iter()
        .map(|(d, p, r)| ((*d).to_string(), *p, (*r).to_string()))
        .collect();
    assert_eq!(got, want);
    assert!(
        found(&root, live(&["other/README.md"]).iter()).is_empty(),
        "no C-family file, no probe"
    );
    std::fs::remove_dir_all(&root).ok();
}

/// The key inputs: every database found under its `c:db:` label, and
/// each in-tree response file a JSON database names under `c:rsp:`;
/// an edit to either moves the hash, so the sweep re-fires.
#[test]
fn facts_name_each_database_and_response_file_with_its_bytes() {
    let root = tree("compdb-find-facts");
    let files = live(&["src/a.c"]);
    let labels = |root: &Path| -> Vec<(String, u64)> { facts(root, files.iter()) };
    let before = labels(&root);
    let names: Vec<&str> = before.iter().map(|(l, _)| l.as_str()).collect();
    assert_eq!(
        names,
        [
            "c:db:build/compile_commands.json",
            "c:rsp:src/flags.rsp",
            "c:db:src/compile_flags.txt"
        ]
    );
    std::fs::write(root.join("src/flags.rsp"), "-I other\n").expect("edit the response file");
    let after = labels(&root);
    assert_eq!(before[0], after[0], "the database itself did not change");
    assert_ne!(
        before[1], after[1],
        "the response file's bytes are the fact"
    );
    std::fs::remove_dir_all(&root).ok();
}
