//! `[graph] crate_roots` (plan v2.18 step #12): a tree whose manifest
//! lives elsewhere — the suite is a slice of the `cli` package, its
//! test binaries cargo targets only in the superproject's Cargo.toml
//! — declares its Rust crate roots in ce.toml. A declared root is
//! everything a manifest target is: the ladder mounts its `mod`
//! children in its own directory and anchors `crate::` paths there,
//! and the entry role sees a declared target. The control tree is the
//! same files undeclared: the name convention (role 0) still keeps the
//! main, but it mounts nothing and anchors nothing, so its child dies.

use crate::common;
use codeeraser::graph::deadcode;

const FILES: &str = "--- it/main.rs\nmod helper;\nuse crate::helper::h;\nfn main() {\n    h();\n}\n\
                     --- it/helper.rs\npub fn h() {}\n";
const DECLARATION: &str = "--- ce.toml\n[graph]\ncrate_roots = [\"it/main.rs\"]\n";

fn judged(tag: &str, doc: &str) -> deadcode::Report {
    let dir = common::fixtures::doc_tree(tag, doc);
    deadcode::run(&dir, None, &common::core_bin()).expect("run")
}

fn dead_paths(report: &deadcode::Report) -> Vec<&str> {
    let mut dead: Vec<&str> = report.dead.iter().map(|d| d.path.as_str()).collect();
    dead.sort();
    dead
}

#[test]
fn a_declared_root_mounts_anchors_and_is_a_target() {
    let report = judged("crate-roots-declared", &format!("{FILES}{DECLARATION}"));
    assert_eq!(
        dead_paths(&report),
        Vec::<&str>::new(),
        "the declared root is a target and its `mod` child is mounted"
    );
    assert!(report.kept >= 1, "the mod edge resolved: {report:?}");
    let report = judged("crate-roots-undeclared", FILES);
    assert_eq!(
        dead_paths(&report),
        ["it/helper.rs"],
        "undeclared, the name convention still keeps main.rs (role 0) but it is no root: nothing mounts, helper dies"
    );
    assert_eq!(report.kept, 0, "and the mod edge falls out of scope");
}

/// A declaration the walk cannot honour is refused by name, never
/// dropped: a root that is no Rust file would make a Markdown page a
/// deadcode target, and a missing one would put the tree back in the
/// false-dead shape the knob exists to end (codex review of step #12).
#[test]
fn a_root_that_is_not_a_walked_rust_file_is_refused_by_name() {
    for (tag, root) in [
        ("crate-roots-not-rust", "README.md"),
        ("crate-roots-missing", "it/gone.rs"),
    ] {
        let dir = common::fixtures::doc_tree(
            tag,
            &format!(
                "{FILES}--- README.md\n# it\n--- ce.toml\n[graph]\ncrate_roots = [\"{root}\"]\n"
            ),
        );
        let err = deadcode::run(&dir, None, &common::core_bin())
            .err()
            .map(|e| format!("{e:#}"))
            .unwrap_or_else(|| panic!("{tag}: judged on a root it cannot honour"));
        assert!(
            err.contains("crate_roots declares") && err.contains(root),
            "{tag}: refused by name: {err}"
        );
    }
}
