//! The file readers' legs: go.mod (gomod.rs), the root pyproject.toml
//! (roots.rs `pyproject`), a compile_commands.json with its response
//! files (compdb.rs `parse`) and a compile_flags.txt (compdb.rs
//! `parse_flags`). The frozen readers read their files from disk, so each
//! case is written to a scratch tree first; the core is sent what the
//! production request sends — the text, the decoded document, and the
//! response files it names, read on demand (`wanted`).

use super::draw::{Draw, ODD, real_texts};
use super::text::{ARGS, ARGV0, chain_json};
use super::{agree, check, count, inspect, link, questions, table};
use crate::graph::{compdb, compdb_find, gomod, roots};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// A clean scratch directory for one leg.
fn scratch(leg: &str) -> PathBuf {
    let base = std::env::var("CE_RESOLVE_DIFF_SCRATCH")
        .map_or_else(|_| std::env::temp_dir(), PathBuf::from);
    let dir = base.join(format!("resolve-diff-{leg}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch");
    dir
}

fn put(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().expect("a parent")).expect("dirs");
    std::fs::write(p, text).expect("write");
}

/// Every fourth question a real text (mutated) when the trees hold any,
/// else one drawn.
fn real_or(
    d: &mut Draw,
    real: &[String],
    i: usize,
    drawn: impl FnOnce(&mut Draw) -> String,
) -> String {
    if real.is_empty() || !i.is_multiple_of(4) {
        return drawn(d);
    }
    let at = d.under(real.len());
    d.mutate(&real[at])
}

/// Lines a go.mod is drawn from.
const GO_LINES: &str = "module example.com/m¦module \"quoted/m\"¦module  spaced/m  ¦module¦go 1.22¦replace (¦)¦replace a/b => ./local¦replace a/b v1.0.0 => ../up v1.2.0¦\tx/y => z/w¦replace c => d // why¦require z v1¦// module commented¦replace e =>¦  ";

#[test]
#[ignore = "needs a core: the differential gate"]
fn go_mods_agree() {
    let (root, real, lines) = (scratch("gomod"), real_texts("go.mod"), table(GO_LINES));
    let seps = table("\n¦\r\n¦ ¦\n\u{85}");
    let qs = questions(5, |d, i| {
        let text = real_or(d, &real, i, |d| d.words(&lines, &seps, 9) + d.one(ODD));
        json!([d.one(&["go.mod", "a/go.mod", "a/é/go.mod"]), text])
    });
    println!("real go.mod texts: {}", real.len());
    on_disk("goMod", &qs, &root, 0, |root, rel| {
        let m = gomod::parse(root, rel).expect("readable");
        json!([m.dir, m.module, m.replaces])
    });
}

fn rel_of(v: &Value) -> &str {
    v.as_str().expect("a string")
}

/// A leg over files: each question's text (`[.., path, text]` from
/// `at`) written at its path, then read back by `read` (the frozen
/// reader, its answer spelled for the comparison).
fn on_disk(leg: &str, qs: &[Value], root: &Path, at: usize, read: impl Fn(&Path, &str) -> Value) {
    check(leg, qs, |q| {
        put(root, rel_of(&q[at]), rel_of(&q[at + 1]));
        read(root, rel_of(&q[at]))
    });
}

/// Fragments a pyproject.toml is drawn from.
const PY_PARTS: &str = "[tool.setuptools.package-dir]\n\"\" = \"src\"\npkg = \"lib/pkg\"\nn = 3\n¦[tool.setuptools]\npackage-dir = { a = \"x\", b = \"y/z\" }\n¦[tool.poetry]\npackages = [{ include = \"a\", from = \"src\" }, { include = \"b\" }, { from = 4 }]\n¦[[tool.poetry.packages]]\ninclude = \"c\"\nfrom = \"lib\"\n¦[project]\ndependencies = [\"requests>=2.31 ; extra\", \"  spaced  \", 7, \"na\u{345}me>1\", \"\u{2170}x\", \"é-name\", \"a.b_c-d[x]\", \"\u{85}lead\"]\n¦[project]\ndependencies = \"not a list\"\n¦[tool]\nsetuptools = 1\n¦when = 1979-05-27T07:32:00Z\n¦[project\nbroken¦";

#[test]
#[ignore = "needs a core: the differential gate"]
fn pyprojects_agree() {
    let (root, real, parts) = (
        scratch("pyproject"),
        real_texts("pyproject.toml"),
        table(PY_PARTS),
    );
    let seps = table("¦\n¦\r\n");
    let texts = questions(6, |d, i| {
        json!(real_or(d, &real, i, |d| d.words(&parts, &seps, 3)))
    });
    let qs: Vec<Value> = texts
        .iter()
        .map(|t| {
            let doc = rel_of(t)
                .parse::<toml::Table>()
                .ok()
                .and_then(|doc| serde_json::to_value(doc).ok());
            doc.unwrap_or(Value::Null)
        })
        .collect();
    let want: Vec<Value> = texts
        .iter()
        .map(|t| {
            put(&root, "pyproject.toml", rel_of(t));
            json!(roots::pyproject(&root).map(|p| (p.source_dirs, p.deps)))
        })
        .collect();
    println!("real pyproject texts: {}", real.len());
    agree(
        "pyproject",
        &qs,
        &want,
        &inspect(&mut link(), "pyproject", &qs),
    );
}

/// The response files a database case may name — present, nested, a
/// cycle, a depth run, under a directory — one `path ⇒ text` per part.
const RESPONSES: &str = "a.rsp ⇒ -Iinc \"-I sp ace\" -DX @sub/b.rsp¦sub/b.rsp ⇒ \u{feff}-isystem sys\n-include f.h\r\n'q'\\¦loop.rsp ⇒ @loop.rsp -Iloop¦d0.rsp ⇒ @d1.rsp¦d1.rsp ⇒ @d2.rsp¦d2.rsp ⇒ @d0.rsp -Ideep¦win.rsp ⇒ /Iwin \"C:\\a b\\\" /FIx.h";
/// What a database's arguments name besides them: missing, outside,
/// absolute, bare.
const RSP_ARGS: &str =
    "@a.rsp¦@sub/b.rsp¦@loop.rsp¦@d0.rsp¦@win.rsp¦@missing.rsp¦@../out.rsp¦@/abs.rsp¦@";

fn db_row(d: &mut Draw, base: &str, words: [&[&str]; 3]) -> Value {
    let [rsp, args_list, argv0] = words;
    let dir = d.one(&["", "sub", "..", "/elsewhere", "@base", "@base/sub", "C:\\x"]);
    let file = d.one(&["a.c", "sub/b.cpp", "../out.c", "@base/abs.c", "x.c", ""]);
    let mut row =
        json!({ "directory": dir.replace("@base", base), "file": file.replace("@base", base) });
    let args: Vec<String> = (0..d.under(6))
        .map(|_| {
            let list = if d.chance(30) { rsp } else { args_list };
            d.one(list).to_string()
        })
        .collect();
    let program = d.one(argv0).to_string();
    if d.chance(50) {
        row["arguments"] = json!(std::iter::once(program).chain(args).collect::<Vec<_>>());
    } else {
        row["command"] = json!(format!("{program} {}", args.join(" ")));
    }
    if d.chance(5) {
        row = json!(["not", "an", "object"]);
    }
    row
}

#[test]
#[ignore = "needs a core: the differential gate"]
fn databases_agree() {
    let (mut d, root) = (Draw::seeded(7), scratch("db"));
    for part in table(RESPONSES) {
        let (rel, text) = part.split_once(" ⇒ ").expect("path ⇒ text");
        put(&root, rel, text);
    }
    let (rsp, args, argv0) = (table(RSP_ARGS), table(ARGS), table(ARGV0));
    let base = compdb_find::root_text(&root);
    let mut core = link();
    let (mut qs, mut want, mut got) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..count() {
        let rows = json!(
            (0..d.under(4) + 1)
                .map(|_| db_row(&mut d, &base, [&rsp, &args, &argv0]))
                .collect::<Vec<_>>()
        );
        put(&root, "compile_commands.json", &rows.to_string());
        let db = compdb::parse(&root, "compile_commands.json").expect("an array");
        let entries: Vec<Value> = db
            .entries
            .iter()
            .map(|e| json!([e.unit, e.dir, chain_json(&e.chain)]))
            .collect();
        want.push(json!([entries, db.responses, Vec::<String>::new()]));
        got.push(db_core(&mut core, &root, &base, &rows));
        qs.push(rows);
    }
    agree("db", &qs, &want, &got);
}

/// One database through the core, its wanted response files read until
/// none is missing.
fn db_core(core: &mut crate::corelink::Link, root: &Path, base: &str, rows: &Value) -> Value {
    let mut texts: Vec<(String, Option<String>)> = Vec::new();
    loop {
        let answer = inspect(core, "db", &[json!([base, rows, texts])]).remove(0);
        let wanted: Vec<String> = serde_json::from_value(answer[2].clone()).expect("wanted");
        if wanted.is_empty() {
            return answer;
        }
        for rel in wanted {
            let text = std::fs::read(root.join(&rel))
                .ok()
                .map(|b| String::from_utf8_lossy(&b).into_owned());
            texts.push((rel, text));
        }
    }
}

#[test]
#[ignore = "needs a core: the differential gate"]
fn flag_files_agree() {
    let root = scratch("flags");
    let base = compdb_find::root_text(&root);
    let (args, seps) = (table(ARGS), table("\n¦\r\n¦  \n¦ ¦\t\n¦\u{85}\n"));
    let rels = table("compile_flags.txt¦sub/compile_flags.txt¦é/compile_flags.txt");
    let qs = questions(8, |d, _| {
        let text = d.words(&args, &seps, 10);
        json!([base, d.one(&rels), text])
    });
    on_disk("flags", &qs, &root, 1, |root, rel| {
        let f = compdb::parse_flags(root, rel).expect("readable");
        json!([f.dir, chain_json(&f.chain)])
    });
}
