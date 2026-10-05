//! The TS readers' texts (plan v2.33 W2-text stage E): JSON values over
//! the edges serde_json 1 (float_roundtrip) draws its accept set at —
//! numbers at the overflow line, lone and paired surrogates, raw control
//! characters, a duplicate key, nesting around the depth limit, white
//! space JSON does not count as such — dressed the JSONC way (comments,
//! trailing commas); tsconfig and package.json files filled in from
//! skeletons, whose `extends` runs through arrays, cycles, bare packages
//! and out-of-tree targets.

use super::draw::Draw;
use super::table;

/// What may sit between tokens: JSON's four white space characters and
/// the comments `clean` strips.
const GAPS: &str = " ¦  ¦\t¦\n¦\r\n¦\r¦// c\n¦/* c */¦/**/¦/* , ] */¦// ,}\n¦/*/ x */";

/// Rarer: white space JSON does not count, a lone slash, an open comment.
const RARE: &str = "\u{a0}¦\u{c}¦\u{feff}¦\u{2028}¦\u{85}¦/¦/* open¦,¦// to the end";

fn gap(d: &mut Draw, out: &mut String) {
    if d.chance(30) {
        out.push_str(d.one(&table(GAPS)));
    }
    if d.chance(1) {
        out.push_str(d.one(&table(RARE)));
    }
}

/// Literals, the edge of the number grammar and of the overflow line
/// (MAX = 1.7976931348623157e308; the halfway point to 2^1024 is
/// 1.797693134862315807937…e308, a tie rounding up).
const ATOMS: &str = "0¦-0¦1¦-1¦1.5¦1.5e3¦1E+2¦1e-2¦1e308¦1e309¦-1e400¦1e-400¦1.7976931348623157e308¦1.7976931348623158e308¦1.79769313486231580793e308¦1.797693134862315808e308¦-1.7976931348623158e308¦18446744073709551615¦18446744073709551616¦-9223372036854775809¦0.0000000000000000000001e330¦01¦1.¦.5¦-¦+1¦1e¦0x10¦NaN¦Infinity¦true¦false¦null¦nul¦tru¦1_0";

/// String literals: escapes, surrogates paired and not, a comment or a
/// comma inside; the raw control characters (DEL is allowed) after `‖`.
const STRINGS: &str = r#""a"¦"\u00e9"¦"\ud83d\ude00"¦"\uD83D\uDE00"¦"\ud800"¦"\udc00x"¦"\ud800\u0041"¦"\ud800\ud800"¦"\/"¦"\q"¦"//not"¦"/* x */"¦"a, }"¦"\""¦"\\"¦"é"¦"\u0000"¦"\u12"¦"unterminated"#;

/// An atom: a literal, a string, a raw control character in a string, a
/// long run of digits.
fn atom(d: &mut Draw) -> String {
    match d.under(20) {
        0..9 => d.one(&table(ATOMS)).to_string(),
        9..17 => d.one(&table(STRINGS)).to_string(),
        17 => format!("\"x{}y\"", d.one(&["\t", "\u{1f}", "\u{7f}", "\n"])),
        18 => "9".repeat(300 + d.under(12)),
        _ => format!("0.{}1e{}", "0".repeat(d.under(400)), 300 + d.under(20)),
    }
}

/// Object keys, a few that are not strings, one twice.
const KEYS: &str = r#""a"¦"a"¦"extends"¦"compilerOptions"¦"paths"¦""¦"é"¦"\u0061"¦a¦'a'"#;

/// A value: an atom, or (shallower than five) an array or an object of
/// up to four members, a trailing comma now and then.
pub(super) fn value(d: &mut Draw, depth: usize, out: &mut String) {
    gap(d, out);
    let (open, close) = match d.under(if depth > 4 { 2 } else { 6 }) {
        0 | 1 => {
            out.push_str(&atom(d));
            gap(d, out);
            return;
        }
        2 | 3 => ('[', ']'),
        _ => ('{', '}'),
    };
    out.push(open);
    for i in 0..d.under(5) {
        if i > 0 {
            out.push(',');
        }
        if open == '{' {
            gap(d, out);
            out.push_str(d.one(&table(KEYS)));
            gap(d, out);
            out.push(':');
        }
        value(d, depth + 1, out);
    }
    if d.chance(15) {
        out.push(',');
    }
    gap(d, out);
    out.push(close);
    gap(d, out);
}

/// One text for the JSONC reader: a value, or nesting around the depth
/// limit, or a value cut short; now and then a byte-order mark.
pub(super) fn document(d: &mut Draw) -> String {
    let mut out = String::new();
    match d.under(12) {
        0 => {
            let n = 124 + d.under(8);
            let (o, c) = if d.chance(50) {
                ("[", "]")
            } else {
                ("{\"a\":", "}")
            };
            out = format!("{}0{}", o.repeat(n), c.repeat(n));
        }
        1 => {
            value(d, 0, &mut out);
            let cut = d.under(out.chars().count() + 1);
            out = out.chars().take(cut).collect();
        }
        _ => value(d, 0, &mut out),
    }
    if d.chance(3) {
        out.insert(0, '\u{feff}');
    }
    out
}

fn quoted(word: &str) -> String {
    serde_json::to_string(word).expect("a string encodes")
}

/// tsconfig skeletons: `@E` an extends value, `@B` a baseUrl, `@P` a
/// paths map, each filled anew; values of the wrong kind among them.
const TSCONFIGS: &str = r#"{"extends": @E, "compilerOptions": {"baseUrl": @B, "paths": @P}}¦{"extends": @E}¦{"compilerOptions": {"paths": @P}}¦{"compilerOptions": {"baseUrl": @B}}¦{"extends": @E, "compilerOptions": {"paths": @P}}¦{"extends": @E, "compilerOptions": {"baseUrl": @B}}¦{"compilerOptions": {"baseUrl": @B, "paths": @P}}¦{}¦{"compilerOptions": {}}¦{"extends": @E, "compilerOptions": null}¦{"compilerOptions": "x"}¦{"compilerOptions": {"baseUrl": 5}}¦null¦[]"#;

/// package.json skeletons: `@N` a name, `@X` an exports value, `@D` a
/// dependency map (one key twice: the last stands).
const PACKAGES: &str = r#"{"name": @N, "exports": @X, "dependencies": @D}¦{"name": @N}¦{"name": @N, "exports": @X}¦{"exports": @X, "devDependencies": @D, "peerDependencies": @D}¦{"name": @N, "optionalDependencies": @D, "dependencies": @D}¦{"name": @N, "dependencies": @D, "dependencies": @D}¦{"name": 5, "dependencies": []}¦{}¦null"#;

/// `extends` targets: relative ones (with and without `.json`, up and
/// out of the tree, back into a cycle), bare packages, the dot forms.
const EXTENDS: &str = "./tsconfig.base.json¦./tsconfig.base¦../tsconfig.json¦../../tsconfig.json¦./shared/base.json¦../shared/base¦./a/tsconfig.json¦./b/../shared/base.json¦../../../out.json¦@tsconfig/node18/tsconfig.json¦pkg¦./missing.json¦.¦./¦..¦./tsconfig.json¦../a/tsconfig.json¦./base.json¦../b/tsconfig¦./shared/x";

/// Every other hole's words, by its letter: baseUrls, paths patterns and
/// targets, names, dependencies, exports targets and keys.
fn words_for(hole: char) -> &'static str {
    match hole {
        'B' => "¦.¦./src¦src¦..¦../..¦../../../x¦/abs¦src/../lib¦é",
        'K' => "*¦@app/*¦@lib¦@a/*/x¦*/*¦@app/*¦",
        'T' => "src/*¦./src/*¦lib/index.ts¦../*¦*¦/abs/*¦src/*/index¦",
        'N' => "@scope/p1¦p2¦¦é¦@scope/¦a/b",
        'D' => "react¦@types/node¦zod¦¦react¦é",
        'G' => "./src/index.ts¦./src/*.ts¦./src/*¦../x/*¦./lib/*.js",
        _ => ".¦./sub¦./lib/*¦./*¦./x/*/y¦./null¦import¦types¦default",
    }
}

/// A `[...]` of one to three quoted words, a number among them now and
/// then.
fn list(d: &mut Draw, words: &'static str) -> String {
    let mut items: Vec<String> = (0..1 + d.under(3))
        .map(|_| quoted(d.one(&table(words))))
        .collect();
    if d.chance(8) {
        items.insert(d.under(items.len() + 1), "3".to_string());
    }
    format!("[{}]", items.join(", "))
}

/// A `{...}` of up to `n` members `"key": value`.
fn map(d: &mut Draw, n: usize, key: char, member: fn(&mut Draw) -> String) -> String {
    let members: Vec<String> = (0..d.under(n + 1))
        .map(|_| format!("{}: {}", quoted(d.one(&table(words_for(key)))), member(d)))
        .collect();
    format!("{{{}}}", members.join(", "))
}

/// One hole's value.
fn hole(d: &mut Draw, letter: char) -> String {
    match letter {
        'E' => match d.under(10) {
            0..6 => quoted(d.one(&table(EXTENDS))),
            6..9 => list(d, EXTENDS),
            _ => d.one(&["7", "null", "{}", "true"]).to_string(),
        },
        'P' => map(d, 3, 'K', |d| {
            if d.chance(8) {
                "\"notarray\"".to_string()
            } else {
                list(d, words_for('T'))
            }
        }),
        'X' => exports(d, 0),
        'D' => map(d, 3, 'D', |_| "\"^1\"".to_string()),
        other => quoted(d.one(&table(words_for(other)))),
    }
}

/// An exports value: a target, a map of subpaths or conditions over
/// further values, null, a number, a value of the JSONC generator.
fn exports(d: &mut Draw, depth: usize) -> String {
    match d.under(if depth > 1 { 4 } else { 9 }) {
        0 | 1 => quoted(d.one(&table(words_for('G')))),
        2 => d.one(&["null", "4", "[]"]).to_string(),
        3 => {
            let mut out = String::new();
            value(d, 3, &mut out);
            out
        }
        _ => {
            let members: Vec<String> = (0..1 + d.under(4))
                .map(|_| {
                    format!(
                        "{}: {}",
                        quoted(d.one(&table(words_for('X')))),
                        exports(d, depth + 1)
                    )
                })
                .collect();
            format!("{{{}}}", members.join(", "))
        }
    }
}

/// A skeleton of `table` with its holes filled, JSONC-dressed.
fn filled(d: &mut Draw, table_of: &'static str) -> String {
    let skeleton = d.one(&table(table_of));
    let mut out = String::new();
    let mut rest = skeleton;
    while let Some(at) = rest.find('@') {
        out.push_str(&rest[..at]);
        let letter = rest[at + 1..].chars().next().expect("a hole letter");
        out.push_str(&hole(d, letter));
        rest = &rest[at + 2..];
    }
    out.push_str(rest);
    dressed(d, out)
}

/// A config's dressing: a leading comment, a line per member (CRLF now
/// and then), a trailing comma, a byte-order mark, a cut.
fn dressed(d: &mut Draw, doc: String) -> String {
    let mut out = doc;
    if d.chance(20) {
        let eol = if d.chance(30) { ",\r\n  " } else { ",\n  " };
        out = out.replace(", ", eol);
    }
    if d.chance(15) && out.starts_with('{') {
        out.insert_str(1, " // c\n");
    }
    if d.chance(15) && out.ends_with('}') && out.len() > 2 {
        out.insert(out.len() - 1, ',');
    }
    match d.under(40) {
        0 => format!("\u{feff}{out}"),
        1 => out
            .chars()
            .take(d.under(out.chars().count().max(1)))
            .collect(),
        _ => out,
    }
}

/// One tsconfig's text.
pub(super) fn tsconfig(d: &mut Draw) -> String {
    filled(d, TSCONFIGS)
}

/// One package.json's text.
pub(super) fn package(d: &mut Draw) -> String {
    filled(d, PACKAGES)
}
