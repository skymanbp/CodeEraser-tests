//! The TS legs' texts (plan v2.33 W2-text stage E): tsconfig files and
//! package.json files written as JSONC the way projects write them — a
//! comment here and there, a trailing comma, CRLF, a byte-order mark now
//! and then, a text that does not read at all — and the specifiers each
//! rung is aimed with. Every word comes from the `ts` rows of tables.rs.

use super::rng::Rng;
use super::tables::t;

/// A JSON string literal of a word.
fn quoted(word: &str) -> String {
    serde_json::to_string(word).expect("a string encodes")
}

/// An object of rendered members, JSONC-dressed: a comment before a
/// member, a trailing comma, a line ending.
fn object(rng: &mut Rng, members: &[(String, String)]) -> String {
    let nl = if rng.chance(20) { "\r\n" } else { "\n" };
    let mut out = String::from("{");
    for (i, (key, value)) in members.iter().enumerate() {
        if rng.chance(10) {
            out.push_str(&format!("{nl}  // {key}{nl}"));
        } else if rng.chance(5) {
            out.push_str(" /* note */ ");
        }
        let comma = if i + 1 < members.len() || rng.chance(20) {
            ","
        } else {
            ""
        };
        out.push_str(&format!("{nl}  {}: {value}{comma}", quoted(key)));
    }
    out.push_str(nl);
    out.push('}');
    out
}

/// A list of one to `max` words of a table, rendered.
fn list(rng: &mut Rng, table: &'static str, max: usize) -> String {
    let n = 1 + rng.below(max);
    let words: Vec<String> = (0..n).map(|_| quoted(rng.pick(table))).collect();
    format!("[{}]", words.join(", "))
}

/// The `extends` value: one target, a list, or something that is no
/// target.
fn extends(rng: &mut Rng) -> String {
    match rng.below(10) {
        0..6 => quoted(rng.pick(t("ts EXTENDS"))),
        6..8 => list(rng, t("ts EXTENDS"), 3),
        8 => format!("[{}, 3]", quoted(rng.pick(t("ts EXTENDS")))),
        _ => rng.pick("7|null|{}|true").to_string(),
    }
}

/// `compilerOptions`: a baseUrl, a paths map, or neither; now and then a
/// value of the wrong kind.
fn compiler(rng: &mut Rng) -> String {
    let mut members = Vec::new();
    if rng.chance(55) {
        let base = if rng.chance(8) {
            "5".to_string()
        } else {
            quoted(rng.pick(t("ts BASE_URLS")))
        };
        members.push(("baseUrl".to_string(), base));
    }
    if rng.chance(65) {
        let mut paths = Vec::new();
        for _ in 0..rng.below(5) {
            let targets = match rng.below(12) {
                0 => "\"notarray\"".to_string(),
                1 => format!("[{}, 3]", quoted(rng.pick(t("ts TARGETS")))),
                _ => list(rng, t("ts TARGETS"), 2),
            };
            paths.push((rng.pick(t("ts PATTERNS")).to_string(), targets));
        }
        members.push(("paths".to_string(), object(rng, &paths)));
    }
    if rng.chance(5) {
        return rng.pick("null|\"x\"|[]").to_string();
    }
    object(rng, &members)
}

/// One tsconfig's text.
pub(super) fn tsconfig(rng: &mut Rng) -> String {
    let mut members = Vec::new();
    if rng.chance(55) {
        members.push(("extends".to_string(), extends(rng)));
    }
    if rng.chance(85) {
        members.push(("compilerOptions".to_string(), compiler(rng)));
    }
    let doc = object(rng, &members);
    dressed(rng, doc)
}

/// An exports value: a target, a condition map, a subpath map with
/// patterns and conditions inside, null, a number.
fn exports(rng: &mut Rng, depth: usize) -> String {
    match rng.below(if depth > 1 { 3 } else { 10 }) {
        0 | 1 => quoted(rng.pick(t("ts EXPORT_TARGETS"))),
        2 => rng.pick("null|4|[]").to_string(),
        n => {
            // a condition map (up to three) or a subpath map (up to five)
            let (keys, most) = if n < 5 {
                ("ts CONDITIONS", 3)
            } else {
                ("ts EXPORT_KEYS", 5)
            };
            let members: Vec<(String, String)> = (0..1 + rng.below(most))
                .map(|_| (rng.pick(t(keys)).to_string(), exports(rng, depth + 1)))
                .collect();
            object(rng, &members)
        }
    }
}

/// One package.json's text.
pub(super) fn package(rng: &mut Rng) -> String {
    let mut members = Vec::new();
    if rng.chance(85) {
        members.push(("name".to_string(), quoted(rng.pick(t("ts PKG_NAMES")))));
    }
    if rng.chance(70) {
        members.push(("exports".to_string(), exports(rng, 0)));
    }
    for key in [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ] {
        if rng.chance(30) {
            let deps: Vec<(String, String)> = (0..rng.below(4))
                .map(|_| (rng.pick(t("ts DEPS")).to_string(), "\"^1\"".to_string()))
                .collect();
            let value = if rng.chance(5) {
                "[]".to_string()
            } else {
                object(rng, &deps)
            };
            members.push((key.to_string(), value));
        }
    }
    let doc = object(rng, &members);
    dressed(rng, doc)
}

/// A document's whole text: now and then a byte-order mark, a leading
/// comment, a cut that leaves it unreadable.
fn dressed(rng: &mut Rng, doc: String) -> String {
    match rng.below(40) {
        0 => format!("\u{feff}{doc}"),
        1 => doc.chars().take(rng.below(doc.len().max(1))).collect(),
        2 => format!("// header\n{doc}\n/* tail"),
        _ => doc,
    }
}

/// The path from directory `from` to `target`, `./` or `../` first.
fn relative(from: &str, target: &str) -> String {
    let a: Vec<&str> = from.split('/').filter(|s| !s.is_empty()).collect();
    let b: Vec<&str> = target.split('/').collect();
    let common = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let ups = a.len() - common;
    let rest = b[common..].join("/");
    if ups == 0 {
        format!("./{rest}")
    } else {
        format!("{}{rest}", "../".repeat(ups))
    }
}

/// A relative specifier to a walked file from `from`'s directory: the
/// extension kept, dropped, or rewritten to its JavaScript twin's (always,
/// when `esm`), an index file named by its directory; else an odd one.
fn relative_spec(rng: &mut Rng, files: &[&String], from: &str, esm: bool) -> String {
    if files.is_empty() || rng.chance(15) {
        return rng.pick(t("ts REL_ODD")).to_string();
    }
    let target = files[rng.below(files.len())];
    let dir = from.rfind('/').map_or("", |i| &from[..i]);
    let rel = relative(dir, target);
    let stem = [".d.ts", ".ts", ".tsx", ".mts", ".cts"]
        .iter()
        .find_map(|e| rel.strip_suffix(e));
    let pick = if esm { 2 } else { rng.below(6) };
    match (stem, pick) {
        (Some(s), 0 | 1) => s.trim_end_matches("/index").to_string(),
        (Some(s), 2) => format!("{s}{}", rng.pick(".js|.mjs|.cjs")),
        (Some(s), 3) => s.to_string(),
        _ => rel,
    }
}

/// A specifier spelled from a walked file under one of a table's
/// `dir>prefix` pairs: the file's path below `dir`, its extension
/// dropped, behind `prefix`; else (or with none there) a word of `words`.
fn spelled(rng: &mut Rng, files: &[&String], pairs: &str, words: &str) -> String {
    let (dir, prefix) = rng
        .pick(t(pairs))
        .split_once('>')
        .expect("a dir>prefix pair");
    let under: Vec<&&String> = files.iter().filter(|f| f.starts_with(dir)).collect();
    if under.is_empty() || rng.chance(30) {
        return rng.pick(t(words)).to_string();
    }
    let rest = &under[rng.below(under.len())][dir.len()..];
    let stem = [".d.ts", ".ts", ".tsx", ".mts", ".cts"]
        .iter()
        .find_map(|e| rest.strip_suffix(e))
        .unwrap_or(rest);
    format!("{prefix}{stem}")
}

/// A specifier aimed at rung `rung` (0 relative, 1 tsconfig paths and
/// baseUrl, 2 workspace members, 3 bare names, 4 the ESM rewrite), one in
/// four aimed at another rung.
pub(super) fn spec(rng: &mut Rng, rung: usize, files: &[&String], from: &str) -> String {
    let aim = if rng.chance(75) { rung } else { rng.below(5) };
    match aim {
        0 | 4 => relative_spec(rng, files, from, aim == 4),
        1 => spelled(rng, files, "ts SPELL_PATHS", "ts ALIAS"),
        2 => spelled(rng, files, "ts SPELL_MEMBERS", "ts WORKSPACE"),
        _ => rng.pick(t("ts BARE")).to_string(),
    }
}
