use super::*;
use crate::testutil::unmarked;

/// Every table uses a single literal; a tilde denotes no words, braces
/// an empty word; control markers expand without interpreting the
/// backslashes under test.
fn check(table: &str, split: impl Fn(&str) -> Vec<String>) {
    for row in table.trim_matches('\n').lines() {
        let (input, expected) = row.split_once(" => ").expect("case separator");
        let expected: Vec<String> = expected
            .split('|')
            .filter(|word| *word != "~")
            .map(|word| unmarked(if word == "{}" { "" } else { word }))
            .collect();
        assert_eq!(split(&unmarked(input)), expected, "{input}");
    }
}

/// JSON quoting concatenates pieces and stops with its partial argument at EOF.
#[test]
fn json_words_follow_clang() {
    check(
        r#"
cc -I/a -c f.c => cc|-I/a|-c|f.c
cc "-I/my dir" f.c => cc|-I/my dir|f.c
cc -DX=\"y\" f.c => cc|-DX="y"|f.c
cc -I'/q dir' f.c => cc|-I/q dir|f.c
cc a"b c"d f.c => cc|ab cd|f.c
cc a{TAB}b{LF}c => cc|a{TAB}b{LF}c
cc -I/a\ => cc|-I/a
cc "unterminated => cc|unterminated
cc 'a\b' "c\d" => cc|a\b|cd
cc "" '' => cc|{}|{}
cc "tail\ => cc|tail
cc 'tail\ => cc|tail\
 => ~
   cc   x   => cc|x
"#,
        split_gnu_json,
    );
}

/// Response GNU whitespace and escapes differ from JSON's POSIX reader.
#[test]
fn gnu_response_words_follow_llvm() {
    check(
        r#"
-I/a{LF}{TAB}-I"/b c"{LF}-D'X=1' => -I/a|-I/b c|-DX=1
a\ b => a b
'a\ b' "c\ d" => a b|c d
x{CR}{LF}y => x|y
a\ => a\
'a\ => a\
"" x => {}|x
a"b c"d => ab cd
"open => open
 => ~
"#,
        split_gnu,
    );
}

/// A mode field distinguishes Windows response arguments from a leading program.
#[test]
fn windows_words_follow_llvm() {
    check(
        r#"
program cl.exe /I"C:\p q" /c a.cpp => cl.exe|/IC:\p q|/c|a.cpp
program "C:\Program Files\LLVM\bin\clang-cl.exe" -IC:\x a.cpp => C:\Program Files\LLVM\bin\clang-cl.exe|-IC:\x|a.cpp
response a\\\"b => a\"b
response a\\\\"b c" => a\\b c
response "a""b" => a"b
response x\y\z => x\y\z
response a{TAB}b{CR}{LF}c{NUL}d => a|b|c|d
response "" "open => {}|open
program "C:\dir\" /Iinc => C:\dir\|/Iinc
response 'a b' => 'a|b'
response  => ~
"#,
        |row| {
            let (mode, input) = row.split_once(' ').expect("Windows mode");
            split_windows(input, mode == "program")
        },
    );
}

/// Program spellings choose syntax without any host-platform branch.
#[test]
fn program_shapes_and_command_selection() {
    check(
        r#"
cl => true
cl.exe => true
clang-cl => true
C:/x/clang.exe => true
C:\x\gcc => true
c:mingw\gcc => true
CC.BAT => true
CC.CmD => true
/opt/CLANG-CL.EXE => true
cc => false
/usr/bin/clang++ => false
clang-cl-wrapper => false
g++-13 => false
1:cc => false
 => false
"#,
        |program| vec![windows_shaped(program).to_string()],
    );
    check(
        r#"
cl /I x a.cpp => cl|/I|x|a.cpp
cc -I x a.c => cc|-I|x|a.c
cl /Ia\ b a.cpp => cl|/Ia\|b|a.cpp
cc -Ia\ b a.c => cc|-Ia b|a.c
"C:\Program Files\cl.exe" /I"a b" => C:\Program Files\cl.exe|/Ia b
"#,
        split_command,
    );
}
