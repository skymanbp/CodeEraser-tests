//! The TS readers' legs (plan v2.33 W2-text stage E): the JSONC reader
//! (jsonc.rs `clean` before serde_json 1's `from_str`), the tsconfig
//! chain (roots_ts.rs `ts_options`, `ts_extends_files`) and the
//! package.json surface (roots.rs `package`). Numbers are compared as
//! zero: the core reads one for its acceptance only, no rung reads its
//! value. The frozen readers read files, so each question's tree is
//! written under its own scratch root first; the core is sent the tree
//! as texts (a null text: a file that does not read as UTF-8).

use super::draw::{Draw, real_texts};
use super::files::{real_or, scratch};
use super::ts_gen::{document, package, tsconfig};
use super::{check, questions, table};
use crate::graph::{jsonc, roots, roots_ts};
use serde_json::{Value, json};
use std::path::Path;

/// Numbers zeroed, everywhere in a document.
fn zeroed(v: Value) -> Value {
    match v {
        Value::Number(_) => json!(0),
        Value::Array(a) => Value::Array(a.into_iter().map(zeroed).collect()),
        Value::Object(m) => Value::Object(m.into_iter().map(|(k, v)| (k, zeroed(v))).collect()),
        other => other,
    }
}

/// A file of a question's tree: its text, or bytes that do not read.
fn write(root: &Path, rel: &str, text: &Value) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().expect("a parent")).expect("dirs");
    match text.as_str() {
        Some(t) => std::fs::write(p, t),
        None => std::fs::write(p, b"{\"a\": \"\xff\xfe\"}"),
    }
    .expect("write");
}

/// The real tsconfig and package.json texts of the trees.
fn real_configs() -> Vec<String> {
    let mut real = real_texts("tsconfig.json");
    real.extend(real_texts("package.json"));
    println!("real tsconfig / package.json texts: {}", real.len());
    real
}

#[test]
#[ignore = "needs a core: the differential gate"]
fn jsonc_agrees() {
    let real = real_configs();
    let qs = questions(31, |d, i| json!(real_or(d, &real, i, document)));
    let read =
        |q: &Value| serde_json::from_str::<Value>(&jsonc::clean(q.as_str().expect("a text"))).ok();
    let read_n = qs.iter().filter(|q| read(q).is_some()).count();
    println!("documents that read: {read_n} of {}", qs.len());
    check("jsoncAccepts", &qs, |q| json!(read(q).is_some()));
    // the documents themselves, those shallow enough to come back inside a
    // reply (serde_json reads 128 levels; the reply wraps four)
    let shallow: Vec<Value> = qs
        .iter()
        .filter(|q| read(q).is_none_or(|d| depth(&d) < 100))
        .cloned()
        .collect();
    check("jsonc", &shallow, |q| {
        read(q).map_or(Value::Null, |doc| json!([zeroed(doc)]))
    });
}

/// A document's nesting: a scalar 0, a container one more than its
/// deepest member.
fn depth(v: &Value) -> usize {
    match v {
        Value::Array(a) => 1 + a.iter().map(depth).max().unwrap_or(0),
        Value::Object(m) => 1 + m.values().map(depth).max().unwrap_or(0),
        _ => 0,
    }
}

/// Where configs sit: nested, beside each other, in a shared directory.
const CFG_PATHS: &str = "tsconfig.json¦a/tsconfig.json¦a/b/tsconfig.json¦shared/base.json¦tsconfig.base.json¦shared/tsconfig.json¦a/b/base.json¦b/tsconfig.json¦shared/x.json¦a/shared/base.json¦a/base.json¦out.json";

const DIRS: &str = "¦a¦a/b¦a/b/c¦b¦shared¦c¦é";

/// A tree of configs: each place filled with a chance, its text drawn,
/// a real one mutated, or no text at all.
fn tree(d: &mut Draw, real: &[String], i: usize) -> Value {
    let mut rows = Vec::new();
    for rel in table(CFG_PATHS) {
        if !d.chance(40) {
            continue;
        }
        let text = if d.chance(5) {
            Value::Null
        } else {
            json!(real_or(d, real, i, tsconfig))
        };
        rows.push(json!([rel, text]));
    }
    json!(rows)
}

/// A leg over trees: each question's tree written under its own root,
/// then read there by `read`.
fn on_trees(leg: &str, qs: &[Value], mut read: impl FnMut(&Path, &str) -> Value) {
    let base = scratch(leg);
    let mut n = 0;
    check(leg, qs, |q| {
        n += 1;
        let root = base.join(format!("q{n}"));
        std::fs::create_dir_all(&root).expect("root");
        for row in q[1].as_array().expect("a tree") {
            write(&root, row[0].as_str().expect("a path"), &row[1]);
        }
        read(&root, q[0].as_str().expect("a path"))
    });
}

#[test]
#[ignore = "needs a core: the differential gate"]
fn tsconfig_chains_agree() {
    let real = real_texts("tsconfig.json");
    println!("real tsconfig texts: {}", real.len());
    let qs = questions(32, |d, i| json!([d.one(&table(DIRS)), tree(d, &real, i)]));
    let mut kinds = [0; 3];
    on_trees("tsconfig", &qs, |root, dir| {
        match roots_ts::ts_options(root, dir) {
            roots_ts::TsChain::None => tally(&mut kinds, 0, json!([0, null, [], ""])),
            roots_ts::TsChain::Broken => tally(&mut kinds, 2, json!([2, null, [], ""])),
            roots_ts::TsChain::Ok(o) => tally(
                &mut kinds,
                1,
                json!([1, o.base_dir, o.paths, o.paths_anchor]),
            ),
        }
    });
    println!("chains none / usable / refused: {kinds:?}");
    let qs = questions(33, |d, i| {
        json!([d.one(&table(CFG_PATHS)), tree(d, &real, i)])
    });
    on_trees("tsReached", &qs, |root, start| {
        json!(roots_ts::ts_extends_files(root, start))
    });
}

fn tally(kinds: &mut [usize; 3], k: usize, answer: Value) -> Value {
    kinds[k] += 1;
    answer
}

const PKG_RELS: &str = "package.json¦a/package.json¦a/b/package.json¦é/package.json";

#[test]
#[ignore = "needs a core: the differential gate"]
fn packages_agree() {
    let real = real_texts("package.json");
    println!("real package.json texts: {}", real.len());
    let qs = questions(34, |d, i| {
        let text = if d.chance(4) {
            Value::Null
        } else {
            json!(real_or(d, &real, i, package))
        };
        json!([d.one(&table(PKG_RELS)), text])
    });
    let root = scratch("package");
    check("package", &qs, |q| {
        let rel = q[0].as_str().expect("a path");
        write(&root, rel, &q[1]);
        roots::package(&root, rel).map_or(Value::Null, |p| {
            let exports: Vec<Value> = p.exports.into_iter().map(zeroed).collect();
            json!([p.dir, p.name, exports, p.deps])
        })
    });
}
