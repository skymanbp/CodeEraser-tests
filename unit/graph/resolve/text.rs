//! The string readers' legs: the two character classes over every
//! scalar value; `lines` / `trim` / `split_whitespace`; the three
//! command tokenizers and the program test (cmdline.rs); the lexical
//! placement (compdb.rs `relativize`); the include chain of an argv
//! (compdb_flags.rs `chain`).

use super::{agree, check, link, questions, table};
use crate::graph::{cmdline, compdb, compdb_flags};
use serde_json::{Value, json};

/// Inclusive scalar ranges a predicate holds for (the surrogate gap
/// bridged, as the core reports them).
fn ranges(p: fn(char) -> bool) -> Vec<[u32; 2]> {
    let mut out: Vec<[u32; 2]> = Vec::new();
    for c in (0..=0x10FFFF_u32)
        .filter_map(char::from_u32)
        .filter(|c| p(*c))
    {
        let u = c as u32;
        match out.last_mut() {
            Some(r) if r[1] + 1 == u || (r[1] == 0xD7FF && u == 0xE000) => r[1] = u,
            _ => out.push([u, u]),
        }
    }
    out
}

#[test]
#[ignore = "needs a core: the differential gate"]
fn chars_agree_on_every_scalar_value() {
    let body = json!({ "inspect": { "chars": true } });
    let reply = crate::corelink::judged::ask(&mut link(), "resolve/1", "9.0.0", "resolve", body)
        .expect("core");
    let got = &reply["inspected"]["chars"];
    let want =
        json!({ "white": ranges(char::is_whitespace), "alnum": ranges(char::is_alphanumeric) });
    agree(
        "chars",
        &[json!("every scalar value")],
        &[want],
        std::slice::from_ref(got),
    );
}

/// What the tokenizers and the line readers see: separators, quotes,
/// escapes, drive letters, the odd spaces (the space twice: weight).
const TEXT: &str = "  \t\n\r\0\"\"'\\\\ab/:.Clé\u{a0}\u{85}\u{2028}\u{3000}@-Ix";

/// Up to `max` characters of TEXT.
fn text(d: &mut super::draw::Draw, max: usize) -> String {
    let alphabet: Vec<char> = TEXT.chars().collect();
    d.text(&alphabet, max)
}

#[test]
#[ignore = "needs a core: the differential gate"]
fn text_readers_agree() {
    let qs = questions(1, |d, _| json!(text(d, 24)));
    check("text", &qs, |q| {
        let t = s(q);
        let lines: Vec<&str> = t.lines().collect();
        json!([lines, t.trim(), t.split_whitespace().collect::<Vec<_>>()])
    });
}

/// Program spellings the tokenizers branch on.
const PROGRAMS: &str = "gcc¦cl.exe¦CL¦clang-cl¦C:\\tools\\cc.exe¦\"C:\\Program Files\\cl.exe\"¦/usr/bin/clang¦x.BAT¦é:cc¦";

#[test]
#[ignore = "needs a core: the differential gate"]
fn tokenizers_agree() {
    let (programs, modes) = (
        table(PROGRAMS),
        table("command¦gnuJson¦gnu¦windows¦windowsProgram¦shaped"),
    );
    let qs = questions(2, |d, i| {
        let head = if d.chance(70) { d.one(&programs) } else { "" };
        json!([modes[i % modes.len()], format!("{head}{}", text(d, 30))])
    });
    check("split", &qs, split_oracle);
}

fn split_oracle(q: &Value) -> Value {
    let (mode, t) = (s(&q[0]), s(&q[1]));
    match mode {
        "command" => json!(cmdline::split_command(t)),
        "gnuJson" => json!(cmdline::split_gnu_json(t)),
        "gnu" => json!(cmdline::split_gnu(t)),
        "windows" => json!(cmdline::split_windows(t, false)),
        "windowsProgram" => json!(cmdline::split_windows(t, true)),
        _ => json!([cmdline::windows_shaped(t), compdb_flags::is_msvc(t)]),
    }
}

/// Roots, directories and paths the placement reads.
const ROOTS: &str = "/r¦C:/r¦c:/R¦/¦¦é¦/r/é";
const PLACES: &str = "¦a¦a/b¦..¦../x¦./y¦/r¦/r/a¦/R/a¦C:\\r\\a¦c:/r/b¦C:/rx¦/r/¦a//b¦é/ü¦:¦x:";

#[test]
#[ignore = "needs a core: the differential gate"]
fn placement_agrees() {
    let (roots, places) = (table(ROOTS), table(PLACES));
    let qs = questions(3, |d, _| {
        let mut path = d.one(&places).to_string();
        if d.chance(40) {
            path = format!("{path}/{}", d.one(&places));
        }
        json!([d.one(&roots), d.one(&places), path])
    });
    check("relativize", &qs, |q| {
        json!(compdb::relativize(s(&q[0]), s(&q[1]), s(&q[2])))
    });
}

fn s(v: &Value) -> &str {
    v.as_str().expect("a string")
}

/// The words an include chain's argv is drawn from.
pub(super) const ARGS: &str = "-I¦-Ia¦-I-¦-iquote¦-isystem¦-isystem/r/s¦-idirafter¦-isystem-after¦-iprefix¦-iprefixp/¦-iwithprefix¦-iwithprefixbefore¦-F¦-Ffw¦-iframework¦--sysroot¦--sysroot=/r/sys¦-isysroot¦-include¦--include¦--include=h.h¦-imacros¦--imacros¦-include-pch¦-o¦-D¦-DX¦-x¦-Xclang¦-Xpreprocessor¦-Xlinker¦-Xassembler¦/I¦-I¦/external:I¦/external:Iext¦/imsvc¦/FI¦-FIf.h¦/Tc¦/D¦a¦../b¦=inc¦$SYSROOT/inc¦=¦/r/abs¦C:\\r\\win¦inc/¦x.h¦é¦";
pub(super) const ARGV0: &str = "clang¦cl.exe¦C:\\vs\\CL.EXE¦gcc¦clang-cl";

#[test]
#[ignore = "needs a core: the differential gate"]
fn chains_agree() {
    let (roots, places, args, argv0) = (table(ROOTS), table(PLACES), table(ARGS), table(ARGV0));
    let qs = questions(4, |d, _| {
        let n = d.under(12);
        let argv: Vec<&str> = std::iter::once(d.one(&argv0))
            .chain((0..n).map(|_| d.one(&args)))
            .collect();
        json!([d.one(&roots), d.one(&places), argv])
    });
    check("chain", &qs, |q| {
        let argv: Vec<String> = serde_json::from_value(q[2].clone()).expect("argv");
        let (base, dir) = (s(&q[0]), s(&q[1]));
        chain_json(&compdb_flags::chain(&argv, &|p| {
            compdb::relativize(base, dir, p)
        }))
    });
}

/// A chain as the core's inspection spells it.
pub(super) fn chain_json(c: &compdb_flags::Chain) -> Value {
    let places = |v: &[compdb_flags::Search]| -> Vec<Value> {
        v.iter()
            .map(|p| match p {
                compdb_flags::Search::Dir(d) => json!([0, d]),
                compdb_flags::Search::Framework(d) => json!([1, d]),
            })
            .collect()
    };
    json!([
        c.msvc,
        c.own_dir,
        places(&c.quote),
        places(&c.bracket),
        places(&c.system),
        c.forced
    ])
}
