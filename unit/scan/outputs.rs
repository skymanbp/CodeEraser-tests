//! scan/outputs.rs: a build-output name is an output beside one of its
//! own row's project files, and nowhere else.

use super::{OUTPUTS, is_output};
use crate::testutil::{scratch, write_tree};

/// Every row, each of its project files alone beside the directory:
/// the directory is an output — before it exists too — and with the
/// project file gone it is an ordinary directory again. A suffix
/// entry is met by any file of that suffix.
#[test]
fn each_project_file_makes_its_row_an_output() {
    let root = scratch("outputs-rows");
    for row in OUTPUTS.lines() {
        let mut words = row.split_ascii_whitespace();
        let name = words.next().expect("a directory name");
        let dir = root.join(name);
        for file in words.map(|w| w.replace('*', "pkg")) {
            assert!(!is_output(&dir), "{name} with nothing beside it");
            write_tree(&root, &[(&file, "")]);
            assert!(is_output(&dir), "{name} beside {file}, not yet created");
            std::fs::remove_file(root.join(&file)).expect("rm");
        }
    }
}

/// What does not make an output: another row's project file, one a
/// level up, a directory spelled like a project file, and a name no
/// row holds — luarocks' `src/luarocks/build/` beside its sibling
/// modules is the case the any-depth globs got wrong.
#[test]
fn nothing_else_makes_an_output() {
    let root = scratch("outputs-not");
    write_tree(
        &root,
        &[
            ("Cargo.toml", ""),
            ("src/luarocks/cmd.lua", ""),
            ("src/luarocks/build/builtin.lua", ""),
            ("hs/pkg.cabal/keep", ""),
            ("web/package.json", ""),
        ],
    );
    for dir in [
        "build",
        "out",
        "src/target",
        "src/luarocks/build",
        "hs/dist",
        "web/target",
        "web/build",
    ] {
        assert!(!is_output(&root.join(dir)), "{dir} is no build output");
    }
    assert!(is_output(&root.join("target")) && is_output(&root.join("web/dist")));
}
