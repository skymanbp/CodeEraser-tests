//! Build outputs are decided beside the directory (scan/outputs.rs):
//! the walk prunes `target/` beside a `Cargo.toml`, `dist/` beside a
//! `package.json` and `build/` beside a `pyproject.toml`, and walks the
//! same names in a source tree — the luarocks module directory
//! `src/luarocks/build/`, a KOReader plugin's `target/` — which the
//! any-depth globs this replaced hid from every measurement. The
//! guard's one-path question (`walk::in_scope`) answers as the walk
//! does, for files that exist and for ones not yet created.

use crate::common;
use codeeraser::scan::walk::in_scope;
use std::path::Path;

const TREE: &str = "\
--- Cargo.toml
[package]
--- target/debug/gen.rs
fn gen() {}
--- web/package.json
{}
--- web/dist/app.html
<p>built</p>
--- web/build/task.html
<p>a build script's page, not a build's output</p>
--- src/luarocks/build/builtin.lua
return {}
--- plugins/exporter.koplugin/target/html.lua
return {}
--- py/pyproject.toml
--- py/build/lib/pkg.py
x = 1
";

/// `in` / `out` per path: the files above as the walk must read them,
/// then paths no file holds yet, as the guard asks before a write.
const SCOPE: &str = "\
out target/debug/gen.rs
out web/dist/app.html
in web/build/task.html
in src/luarocks/build/builtin.lua
in plugins/exporter.koplugin/target/html.lua
out py/build/lib/pkg.py
out target/new.rs
out web/dist/deep/new.html
in src/luarocks/build/new.lua
in py/new/build/x.py";

#[test]
fn outputs_are_pruned_beside_their_project_file_only() {
    let dir = common::doc_tree("walk-outputs", TREE);
    let walked: Vec<String> = common::walked(&dir)
        .into_iter()
        .map(|(rel, _)| rel)
        .collect();
    for line in SCOPE.lines() {
        let (verdict, rel) = line.split_once(' ').expect("verdict path");
        let inside = verdict == "in";
        assert_eq!(
            in_scope(&dir, Path::new(rel), &[]),
            inside,
            "{rel}: the guard's scope"
        );
        if dir.join(rel).exists() {
            assert_eq!(walked.iter().any(|w| w == rel), inside, "{rel}: the walk");
        }
    }
}
