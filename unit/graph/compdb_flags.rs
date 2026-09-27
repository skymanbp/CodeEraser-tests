use super::*;

/// Render classes without hiding framework identity in directory comparisons.
fn searches(entries: &[Search]) -> String {
    entries
        .iter()
        .map(|entry| match entry {
            Search::Dir(dir) => dir.clone(),
            Search::Framework(dir) => format!("@{dir}"),
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Columns are quote, bracket, system, forced, own-directory, and MSVC.
fn check(table: &str, place: &dyn Fn(&str) -> Option<String>) {
    for row in table.trim().lines() {
        let (command, want) = row.split_once(" => ").expect("case separator");
        let argv = crate::graph::cmdline::split_command(command);
        let got = chain(&argv, place);
        let actual = format!(
            "{} | {} | {} | {} | {} | {}",
            searches(&got.quote),
            searches(&got.bracket),
            searches(&got.system),
            got.forced.join(","),
            got.own_dir,
            got.msvc
        );
        assert_eq!(actual, want, "{command}");
    }
}

/// Classification cases keep their directory spelling unchanged.
fn identity(dir: &str) -> Option<String> {
    Some(dir.to_string())
}

/// The identity-placed cases as one literal, blocks split by `====`
/// and each headed by what its rows pin. One call in a loop rather
/// than a test per block: six same-shaped test bodies read as clones
/// of one another.
const CASES: &str = r#"
Classes reorder the invocation, and the barrier moves every earlier bracket entry.
cc -isystem s -I b -iquote q -idirafter a => q | b | s,a |  | true | false
cc -I q1 -I q2 -I- -I b => q1,q2 | b |  |  | false | false
cc -iquote q -F f -I x -I- -I y -I- -I z => q,@f,x,y | z |  |  | false | false
cc -idirafter a -isystem s -isystem-after b -isystem t =>  |  | s,t,a,b |  | true | false
cc -nostdinc -nostdinc++ -nobuiltininc -I a =>  | a |  |  | true | false
cc -Ia -I b -iquoteq -iquote r -isystems -isystem t => q,r | a,b | s,t |  | true | false
====
GNU system duplicates displace user entries of the angled group; the quoted group is deduplicated alone (clang Realize); other repeats keep the first origin.
cc -I x -isystem x =>  |  | x |  | true | false
cc -I x -I x =>  | x |  |  | true | false
cc -iquote x -I x => x | x |  |  | true | false
cc -iquote x -iquote x -I y -I y => x | y |  |  | true | false
cc -iquote x -I x -idirafter x -isystem y -isystem x => x |  | y,x |  | true | false
cc -F x -I x -iframework x =>  | x | @x |  | true | false
====
Last explicit sysroot wins even after directories; unrelated operands are opaque.
cc --sysroot=/sr -I=inc =>  | /sr/inc |  |  | true | false
cc -I=inc -I$SYSROOT/inc =>  |  |  |  | true | false
cc --sysroot /sr -I$SYSROOT/inc =>  | /sr/inc |  |  | true | false
cc -I=inc --sysroot /old -isysroot/new =>  | /new/inc |  |  | true | false
cc -isysroot /sr/ -iquote=/q -isystem=/s -idirafter=a -isystem-after $SYSROOT/b -F=f -iframework=g => /sr/q | @/sr/f | /sr/s,@/sr/g,/sr/a,/sr/b |  | true | false
cc --sysroot=/sr -D --sysroot=/wrong -I=inc =>  | /sr/inc |  |  | true | false
cc -iprefix /p/ -iwithprefix a -iwithprefixbefore b =>  | /p/b | /p/a |  | true | false
cc -iwithprefix a -iwithprefixbefore b =>  |  |  |  | true | false
cc -iprefix /p -iwithprefix a -iprefix /q/ -iwithprefixbefore b =>  | /q/b | /pa |  | true | false
====
Forwarding and skipped operands cannot manufacture include options.
cc -Xclang -include -Xclang pch.h =>  |  |  | pch.h | true | false
cc -Xpreprocessor -I -Xpreprocessor inc =>  | inc |  |  | true | false
cc -include-pch x.pch -include-pchjoined =>  |  |  |  | true | false
cc --include=cfg.h -imacros m.h --includenext.h --imacroslast.h -includeone.h =>  |  |  | cfg.h,m.h,next.h,last.h,one.h | true | false
cc -o -I -MF -I -MT -I -MQ -I -x -I -arch -I -target -I -mllvm -I -D -I -U -I -L -I -l -I -z -I -u -I -e -I -T -I -B -I -b -I -V -I =>  |  |  |  | true | false
cc -Xassembler -Ihidden -Xlinker -includehidden -Xanalyzer -Ihidden -I kept =>  | kept |  |  | true | false
cc -unknown -I kept -iquote =>  | kept |  |  | true | false
cc -I =>  |  |  |  | true | false
====
Framework entries retain their positions beside ordinary entries in each class.
cc -F f -iframework g =>  | @f | @g |  | true | false
cc -I a -Ff -I b -iframeworkg -isystem s =>  | a,@f,b | @g,s |  | true | false
====
MSVC accepts slash or dash options, defers imsvc, and preserves first duplicates.
cl /I a /external:I b /imsvc c /FI f.h /X -I d =>  | a,d | b,c | f.h | true | true
clang-cl -imsvc last /external:Ifirst /Ia -FIforced.h =>  | a | first,last | forced.h | true | true
CL.EXE /Ix /external:Ix /imsvc x =>  | x |  |  | true | true
cl /I=inc /I$SYSROOT/inc -I- =>  | =inc,$SYSROOT/inc,- |  |  | true | true
cl /Tc /I /Tp /I /D /I /U /I /Ikept =>  | kept |  |  | true | true
cl /Tcx.c /Tpx.cpp /DX /UX /FI =>  |  |  |  | true | true
clang-cl-wrapper -I x -isystem x =>  |  | x |  | true | false
"#;

#[test]
fn identity_placed_cases_classify_as_the_preprocessor_does() {
    for (_, rows) in crate::testutil::sections(CASES) {
        check(rows, &identity);
    }
}

/// Placement can drop external paths and collapse different spellings before dedup.
#[test]
fn placement_precedes_duplicate_removal() {
    check(
        r#"
cc -I/usr/include -iquote kept -I./inc -isystem inc => kept |  | inc |  | true | false
"#,
        &|dir| (!dir.starts_with("/usr/")).then(|| dir.trim_start_matches("./").to_string()),
    );
    assert_eq!(chain(&[], &identity).bracket, Vec::new());
}
