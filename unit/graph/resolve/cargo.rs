//! The Cargo.toml reader's leg (plan v2.33 W2-text stage F): the frozen
//! 1324c927 reader (unit/graph/oracle_cfg/cargo.rs, mounted at
//! `graph::cargo` under cfg(test)) reads each text from disk as the Rust
//! rungs, the declared-target pass and the mounts table did, and answers
//! its crate roots, bin roots, lib root and — through the frozen
//! `RustTargets` (oracle_cfg/rust_targets.rs) — the files it keeps
//! private; the core reads the document this side decodes (the request's
//! TOML fact, facts.rs) through `inspect.cargo`. Ten thousand questions:
//! every fourth a real Cargo.toml of the trees CE_RESOLVE_DIFF_TREES
//! names, mutated, the rest drawn line by line — `[package]` names of
//! every kind (hyphenated, a number, inline), `[lib]` and `[[bin]]` paths
//! (some escaping the tree, some of the wrong kind), the three
//! dependency tables and keys of the wrong kind, a workspace, a redefined
//! table that does not read — each asked over a drawn set of walked
//! files under and beside the package.

use super::files::{bom_texts, put, scratch};
use super::{agree, inspect, link, table};
use crate::graph::cargo;
use crate::graph::oracle_cfg::rust_targets::RustTargets;
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Lines a Cargo.toml is drawn from.
const LINES: &str = r#"[package]¦name = "a"¦name = "a-b"¦name = 'x_y'¦name = 3¦version = "0.1.0"¦[lib]¦path = "src/x.rs"¦path = "lib/core.rs"¦path = 7¦path = "../out.rs"¦[[bin]]¦name = "gen"¦path = "src/tools/gen.rs"¦path = "src/main.rs"¦[dependencies]¦serde = "1"¦a-b = { path = "../a" }¦"quoted-dep" = "2"¦[dev-dependencies]¦tempfile = "3"¦[build-dependencies]¦cc = "1"¦[workspace]¦members = ["a", "b"]¦bin = 3¦lib = "x"¦dependencies = 5¦[package.metadata.x]¦# comment¦[target.'cfg(unix)'.dependencies]¦libc = "0.2"¦package = { name = "inline" }¦[dependencies.nested]¦version = "1"¦broken = ¦bin = [{ path = "src/b.rs" }, { name = "n" }, 4]¦lib = { path = "src/l.rs" }"#;

/// Walked files a question's package may hold, below its directory.
const FILES: &str = "src/lib.rs¦src/main.rs¦build.rs¦src/bin/x.rs¦src/bin/y/main.rs¦src/bin/y/part.rs¦src/bin/z/w/main.rs¦tests/t.rs¦tests/it/main.rs¦examples/e.rs¦benches/b.rs¦src/tools/gen.rs¦src/x.rs¦lib/core.rs¦src/a/mod.rs¦src/l.rs¦src/b.rs¦src/bin/notes.md¦../out.rs¦../other/src/lib.rs";

/// The frozen reader's answer, spelled as the core's.
fn frozen(pkg: Option<cargo::Package>, files: &BTreeSet<String>) -> Value {
    let Some(p) = pkg else {
        return Value::Null;
    };
    let kept: Vec<&String> = {
        let targets = RustTargets::of(Some(p.clone()), files);
        files.iter().filter(|f| targets.keeps(f)).collect()
    };
    json!([
        p.dir,
        p.name,
        p.deps,
        p.crate_roots(files),
        p.bin_roots(files),
        p.lib_root(files),
        kept
    ])
}

#[test]
#[ignore = "needs a core: the differential gate"]
fn cargo_manifests_agree() {
    let texts = bom_texts(
        15,
        "Cargo.toml",
        LINES,
        "Cargo.toml¦a/Cargo.toml¦crates/x/Cargo.toml¦\u{e9}/Cargo.toml",
        14,
    );
    let (root, files_table) = (scratch("cargo"), table(FILES));
    let mut d = super::draw::Draw::seeded(16);
    let (mut qs, mut want) = (Vec::new(), Vec::new());
    for q in &texts {
        let (rel, text) = (q[0].as_str().expect("rel"), q[1].as_str().expect("text"));
        put(&root, rel, text);
        let dir = crate::graph::roots::parent_dir(rel);
        let files: BTreeSet<String> = (0..d.under(9))
            .filter_map(|_| crate::graph::roots::join_rel(&dir, d.one(&files_table)))
            .collect();
        let doc = super::super::facts::fact(&root, 3, rel, "")[4].clone();
        qs.push(json!([rel, doc, files]));
        want.push(frozen(cargo::package(&root, rel), &files));
    }
    let got = inspect(&mut link(), "cargo", &qs);
    agree("cargo", &qs, &want, &got);
}
