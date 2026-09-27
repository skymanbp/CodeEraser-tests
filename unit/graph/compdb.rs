use super::*;
use crate::graph::compdb_flags::Search;

/// Write a single-literal file table, expanding only explicit fixture markers.
fn fixture(tag: &str, table: &str) -> std::path::PathBuf {
    let root = crate::testutil::scratch(tag);
    let files: Vec<(&str, String)> = table
        .trim()
        .lines()
        .map(|row| {
            let (path, text) = row.split_once(" => ").expect("fixture separator");
            let text = crate::testutil::unmarked(&text.replace("{ROOT}", &root_text(&root)));
            (path, text)
        })
        .collect();
    let files: Vec<(&str, &str)> = files
        .iter()
        .map(|(path, text)| (*path, text.as_str()))
        .collect();
    crate::testutil::write_tree(&root, &files);
    root
}

/// Show every public entry field so database cases exercise the complete new shape.
fn describe(entry: &Entry) -> String {
    format!(
        "{} => {:?}; {:?}; {:?}; {:?}; {:?}; {}; {}",
        entry.unit,
        entry.dir,
        entry.chain.quote,
        entry.chain.bracket,
        entry.chain.system,
        entry.chain.forced,
        entry.chain.msvc,
        entry.chain.own_dir
    )
}

/// Common database assertions and scratch cleanup for each independent file table.
fn database_case(tag: &str, table: &str, expected: &str, responses: &str) {
    let root = fixture(tag, table);
    let db = parse(&root, "cc.json").expect("database parses");
    assert_eq!(
        db.entries
            .iter()
            .map(describe)
            .collect::<Vec<_>>()
            .join("\n"),
        expected.trim(),
        "{tag}"
    );
    assert_eq!(
        db.responses.into_iter().collect::<Vec<_>>().join("|"),
        responses,
        "{tag}"
    );
    std::fs::remove_dir_all(&root).ok();
}

/// Compare a path table without asking the host to interpret any spelling.
fn check_paths(cases: &str) {
    for row in cases.trim_matches('\n').lines() {
        let (input, want) = row.split_once(" => ").expect("path case");
        let cols: Vec<&str> = input.split(" | ").collect();
        let want = (want != "~").then_some(if want == "{}" { "" } else { want });
        assert_eq!(
            relativize(cols[0], cols[1], cols[2]).as_deref(),
            want,
            "{input}"
        );
    }
}

/// Absolute, relative, and Windows spellings place lexically without a filesystem.
#[test]
fn paths_relativize_lexically_against_the_root() {
    check_paths(
        r#"
/opt/proj | /opt/proj/build | /opt/proj/src/a.c => src/a.c
/opt/proj | /opt/proj/build | ../src/a.c => src/a.c
/opt/proj | build | ../src/a.c => src/a.c
/opt/proj | /opt/proj/build | /usr/include => ~
/opt/proj | /opt/project/build | a.c => ~
D:/proj | d:\proj\build | ..\src\a.c => src/a.c
D:/proj | D:\proj | d:\PROJ\inc => inc
/opt/proj | /elsewhere | /opt/proj/a.c => a.c
/opt/proj |  | ../a.c => ~
/opt/proj | build/../src | ./a.c => src/a.c
/opt/proj | ../outside | a.c => ~
/opt/proj | /opt/proj |  => {}
"#,
    );
}

/// Every JSON database case as one literal: blocks split by `====`,
/// each headed `tag @@ what its rows pin`, the body a file table, a
/// `----` line, the expected entries, a `----` line and the expected
/// response files. One call in a loop rather than a test per case:
/// five same-shaped test bodies read as clones of one another.
const DATABASES: &str = r#"
compdb-order @@ Both flag spellings preserve order within their own class.
cc.json => [{"file":"a.c","arguments":["cc","-I/a","-I","lib","-iquote","q","-isystems","-include","x.h","-c","a.c"]}]
----
a.c => Some(""); [Dir("q")]; [Dir("lib")]; [Dir("s")]; ["x.h"]; false; true
----
====
compdb-rows @@ In-tree absolute operands survive an outside working directory; outside units do not.
cc.json => [{"directory":"{ROOT}/build","file":"../src/a.c","arguments":["cc","-I{ROOT}/inc"]},{"directory":"/elsewhere","file":"/elsewhere/b.c","command":"cc @hidden.rsp"},{"command":"cc @also-hidden.rsp"},{"directory":"/elsewhere","file":"{ROOT}/src/b.c","arguments":["cc","-Irelative","-I{ROOT}/shared","-include","cfg.h"]}]
----
src/a.c => Some("build"); []; [Dir("inc")]; []; []; false; true
src/b.c => None; []; [Dir("shared")]; []; ["cfg.h"]; false; true
----
====
compdb-command @@ JSON commands keep quoted directory spaces; an arguments array overrides command.
cc.json => [{"file":"a.c","command":"cc \"-I/my dir\" -I'local dir' -DX=\\\"y\\\" a.c"},{"file":"b.c","arguments":["cl","/I","inc",9,"/external:I","ext","/imsvc","sdk","/FI","cfg.h"],"command":"cc -Iwrong"},{"file":"c.c","arguments":[],"command":"cc @ignored.rsp"}]
----
a.c => Some(""); []; [Dir("local dir")]; []; []; false; true
b.c => Some(""); []; [Dir("inc")]; [Dir("ext"), Dir("sdk")]; ["cfg.h"]; true; true
c.c => Some(""); []; []; []; []; false; true
----
====
compdb-response @@ Nested responses resolve against the entry directory and keep missing inputs visible.
cc.json => [{"directory":"build","file":"../src/a.c","arguments":["cc","@flags.rsp","@missing.rsp","@/elsewhere/x.rsp"]}]
build/flags.rsp => {BOM}-I../inc{LF}-DX @sub/nested.rsp
build/sub/nested.rsp => -iquote "../quoted dir" @leaf.rsp
build/leaf.rsp => -isystem ../system
----
src/a.c => Some("build"); [Dir("quoted dir")]; [Dir("inc")]; [Dir("system")]; []; false; true
----
build/flags.rsp|build/leaf.rsp|build/missing.rsp|build/sub/nested.rsp
====
compdb-win-response @@ An absolute response and a Windows-shaped non-MSVC compiler use Windows tokenization.
cc.json => [{"directory":"build","file":"../a.cpp","command":"clang-cl @flags.rsp"},{"directory":"build","file":"../b.c","arguments":["C:/tools/gcc.exe","@{ROOT}/build/gnu.rsp"]}]
build/flags.rsp => /I"..\inc dir" /FI"config file.h"
build/gnu.rsp => -I..\inc
----
a.cpp => Some("build"); []; [Dir("inc dir")]; []; ["config file.h"]; true; true
b.c => Some("build"); []; [Dir("inc")]; []; []; false; true
----
build/flags.rsp|build/gnu.rsp
"#;

/// Each database block reads as clang's JSONCompilationDatabase reads it.
#[test]
fn databases_read_as_clang_reads_them() {
    for (head, body) in crate::testutil::sections(DATABASES) {
        let tag = head.split(" @@ ").next().expect("a tag");
        let [table, expected, responses]: [&str; 3] = body
            .split("\n----")
            .map(str::trim)
            .collect::<Vec<_>>()
            .try_into()
            .expect("files ---- entries ---- responses");
        database_case(tag, table, expected, responses);
    }
}

/// Cycles remain literal, whereas a later visit after stack unwinding expands again.
#[test]
fn response_cycles_and_external_paths_remain_literal() {
    let root = fixture(
        "compdb-cycle",
        r#"
a.rsp => -include a.h @b.rsp
b.rsp => -include b.h @a.rsp
"#,
    );
    let argv = [
        "cc",
        "@a.rsp",
        "@/elsewhere/x.rsp",
        "@missing.rsp",
        "@a.rsp",
    ]
    .map(String::from);
    let mut responses = BTreeSet::new();
    let expanded = expand(&root, "", &argv, false, &mut responses, &mut Vec::new());
    assert_eq!(
        expanded.join("|"),
        "cc|-include|a.h|-include|b.h|@a.rsp|@/elsewhere/x.rsp|-include|a.h|-include|b.h|@a.rsp"
    );
    assert_eq!(
        responses.into_iter().collect::<Vec<_>>(),
        ["a.rsp", "b.rsp", "missing.rsp"]
    );
    std::fs::remove_dir_all(&root).ok();
}

/// The seventeenth response is tracked but unopened, so only sixteen add directories.
#[test]
fn response_depth_is_bounded_and_limit_inputs_are_recorded() {
    let root = crate::testutil::scratch("compdb-depth");
    for at in 0..18 {
        std::fs::write(
            root.join(format!("r{at}.rsp")),
            format!("-I inc{at} @r{}.rsp", at + 1),
        )
        .unwrap();
    }
    let mut responses = BTreeSet::new();
    let argv = ["cc", "@r0.rsp"].map(String::from);
    let expanded = expand(&root, "", &argv, false, &mut responses, &mut Vec::new());
    assert_eq!(expanded.last().map(String::as_str), Some("@r16.rsp"));
    assert_eq!(responses.len(), 17);
    assert!(!responses.contains("r17.rsp"));
    assert_eq!(
        compdb_flags::chain(&expanded, &|p| Some(p.to_string()))
            .bracket
            .len(),
        16
    );
    std::fs::remove_dir_all(&root).ok();
}

/// Response byte decoding is lossy, with its leading UTF-8 BOM removed.
#[test]
fn response_bytes_are_lossy_utf8() {
    let root = crate::testutil::scratch("compdb-lossy");
    std::fs::write(root.join("bytes.rsp"), [0xef, 0xbb, 0xbf, b'-', b'I', 0xff]).unwrap();
    let argv = ["cc", "@bytes.rsp"].map(String::from);
    let expanded = expand(
        &root,
        "",
        &argv,
        false,
        &mut BTreeSet::new(),
        &mut Vec::new(),
    );
    assert_eq!(expanded, ["cc", "-I\u{fffd}"]);
    std::fs::remove_dir_all(&root).ok();
}

/// Flags belong to their own directory; response-looking words are never opened.
#[test]
fn fixed_flags_place_against_their_parent() {
    let root = fixture(
        "compdb-fixed",
        r#"
pkg/compile_flags.txt =>   -I  {LF}include{LF}-Ilib{LF}{LF}-std=c11{LF}@ignored.rsp{LF}-include{LF}cfg.h{LF}-Iwith space{LF}-I include
pkg/ignored.rsp => -Iwrong
"#,
    );
    let flags = parse_flags(&root, "pkg/compile_flags.txt").expect("fixed flags");
    assert_eq!(flags.dir, "pkg");
    let dirs: Vec<_> = ["pkg/include", "pkg/lib", "pkg/with space", "pkg/ include"]
        .map(|dir| Search::Dir(dir.to_string()))
        .into();
    assert_eq!(flags.chain.bracket, dirs);
    assert_eq!(flags.chain.forced, ["cfg.h"]);
    assert!(!flags.chain.msvc);
    std::fs::remove_dir_all(&root).ok();
}

/// Missing files and non-array JSON refuse without consulting any other file.
#[test]
fn missing_and_invalid_databases_are_absent() {
    let root = fixture(
        "compdb-invalid",
        r#"
cc.json => {"file":"a.c"}
"#,
    );
    assert!(parse(&root, "cc.json").is_none());
    assert!(parse(&root, "missing.json").is_none());
    assert!(parse_flags(&root, "missing.txt").is_none());
    std::fs::remove_dir_all(&root).ok();
}
