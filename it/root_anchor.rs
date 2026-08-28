//! A run pointed at a SUBDIRECTORY answers to the project, not to the
//! directory it was handed. The CLI, MCP and GUI all take a root from
//! a human, and none of them ascended: `ce check cli` judged with no
//! ce.toml and no ce-baseline.json — green by having no rules — and
//! planted a second `.ce/` wherever it was pointed (five such strays
//! existed in this repository, hidden behind an unanchored ignore
//! rule). The hooks fixed this class in 2026-08-19; these are the
//! same properties for every other surface.

use crate::common;
use codeeraser::{config::Config, dedup, score};
use std::path::{Path, PathBuf};

/// An anchored project plus one subdirectory inside it — the shape
/// every case here needs, written once.
fn project_with_subdir(tag: &str, rel: &str) -> (PathBuf, PathBuf) {
    let dir = common::tmp(tag);
    let sub = dir.join(rel);
    std::fs::create_dir_all(&sub).expect("mkdir");
    (dir, sub)
}

/// The three state throats — config, index, baseline — all resolve to
/// the project anchor, while the walk still scopes to what was named.
/// What "answering to the project" means, for a plain subdirectory
/// and for a declared submodule alike: the project's declaration is
/// the one loaded, and an analyze rooted below mints the project's
/// index, never a `.ce` beside the subdirectory.
fn answers_to_the_project(dir: &Path, sub: &Path) {
    let cfg = Config::load(sub).expect("load from the project");
    assert_eq!(cfg.dedup.budget, Some(41), "the project's declaration wins");
    dedup::analyze(sub, None, None, None).expect("analyze");
    assert!(dir.join(".ce/index.db").is_file(), "the project's index");
    assert!(
        !sub.join(".ce").exists(),
        "no stray beside the subdirectory"
    );
}

#[test]
fn state_follows_the_project_not_the_named_subdirectory() {
    let (dir, sub) = project_with_subdir("anchor-state", "pkg/deep");
    std::fs::write(dir.join("ce.toml"), "[dedup]\nbudget = 41\n").expect("ce.toml");
    answers_to_the_project(&dir, &sub);

    // and the baseline is the project's one file, wherever it is named
    let want = dir.join("ce-baseline.json");
    assert_eq!(score::baseline::path_for(&sub), want);
    assert_eq!(score::baseline::path_for(&dir), want);
    std::fs::remove_dir_all(&dir).ok();
}

/// An explicit `--db` still wins verbatim: a deliberately separate
/// index is the documented escape hatch, and the ascent must not
/// swallow it.
#[test]
fn an_explicit_db_path_is_never_re_rooted() {
    let (dir, sub) = project_with_subdir("anchor-db", "pkg");
    let chosen = dir.join("elsewhere/custom.db");
    std::fs::create_dir_all(chosen.parent().expect("parent")).expect("mkdir");

    dedup::analyze(&sub, Some(chosen.clone()), None, None).expect("analyze");
    assert!(chosen.is_file(), "the named db is the one that was built");
    assert!(!dir.join(".ce").exists(), "no project index was minted");
    std::fs::remove_dir_all(&dir).ok();
}

/// `ce eject` names the strays an older binary planted. It is the
/// migration path for the ones that already exist, so the dry run
/// must LIST them — a silent uninstall that leaves indexes of file
/// paths and symbol names behind is the gap, not the sweep.
#[test]
fn eject_lists_nested_stray_caches() {
    let dir = common::tmp("anchor-eject");
    std::fs::write(dir.join("ce.toml"), "\n").expect("ce.toml");
    // (stray, the files that anchor it): a nested ce.toml or an
    // UNDECLARED real .git is a project of its own — not this eject's
    // business, said so by name; a DECLARED submodule is ours, its
    // stray too (the gitfile's target must resolve: root.rs)
    let table: [(&str, &[(&str, &str)]); 6] = [
        (".ce", &[]),
        ("pkg/.ce", &[]),
        ("pkg/deep/.ce", &[]),
        ("vendored/.ce", &[("vendored/ce.toml", "\n")]),
        (
            "vendored_git/.ce",
            &[("vendored_git/.git/HEAD", "ref: refs/heads/main\n")],
        ),
        (
            "submod/.ce",
            &[
                ("submod/.git", "gitdir: ../realgit\n"),
                ("realgit/HEAD", "ref: refs/heads/main\n"),
                (".gitmodules", "[submodule \"s\"]\n\tpath = submod\n"),
            ],
        ),
    ];
    for (stray, anchors) in table {
        std::fs::create_dir_all(dir.join(stray)).expect("mkdir");
        std::fs::write(dir.join(stray).join("index.db"), b"x").expect("seed");
        common::ladder::materialize(&dir, anchors);
    }

    let out = common::run_ce(&dir, &["eject"]);
    assert!(out.status.success(), "eject dry run");
    let text = String::from_utf8_lossy(&out.stdout).replace('\\', "/");
    let says = |head: &str, rel: &str| {
        text.lines()
            .any(|l| l.starts_with(head) && l.ends_with(&format!("/{rel}")))
    };
    for rel in ["pkg/.ce", "pkg/deep/.ce", "submod/.ce"] {
        assert!(
            says("would remove:", rel),
            "stray {rel} unnamed in:\n{text}"
        );
    }
    for rel in ["vendored", "vendored_git"] {
        assert!(
            !says("would remove:", &format!("{rel}/.ce")),
            "{rel}: not ours:\n{text}"
        );
        assert!(
            says("left to its own eject", rel),
            "{rel}: named as kept:\n{text}"
        );
    }
    assert!(
        Path::new(&dir).join("vendored/.ce").exists(),
        "dry run removes nothing"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// A declared submodule answers to the superproject — config, index
/// and baseline — seated or not (plan v2.18 follow-up: a guard whose
/// envelope cwd sat in `cli/tests` once minted a tests-only index
/// there under default knobs).
#[test]
fn a_declared_submodule_answers_to_the_superproject() {
    let (dir, sub) = project_with_subdir("anchor-submodule", "sub");
    let a = common::rust_fn(1);
    common::ladder::materialize(
        &dir,
        &[
            ("ce.toml", "[dedup]\nbudget = 41\n"),
            (".gitmodules", "[submodule \"s\"]\n\tpath = sub\n"),
            ("realgit/HEAD", "ref: refs/heads/main\n"),
            ("sub/.git", "gitdir: ../realgit\n"),
            ("sub/a.rs", &a),
        ],
    );
    assert_eq!(
        score::baseline::path_for(&sub),
        dir.join("ce-baseline.json")
    );
    answers_to_the_project(&dir, &sub);
    std::fs::remove_dir_all(&dir).ok();
}
