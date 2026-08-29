use super::*;

// the crate's ONE tmp-dir scaffold — writing a second copy here
// is exactly what testutil exists to prevent, and the ratchet
// caught it before this comment did
use crate::testutil::scratch;

/// A scratch tree whose `repo/` carries the anchor, returning
/// (scratch, anchored repo, a deep descendant). Both ascent tests
/// need exactly this and the dedup gate refused the second copy.
fn anchored(tag: &str, depth: &str) -> (PathBuf, PathBuf, PathBuf) {
    let dir = scratch(tag);
    let repo = dir.join("repo");
    let deep = repo.join(depth);
    std::fs::create_dir_all(&deep).expect("mkdir");
    std::fs::write(repo.join("ce.toml"), "\n").expect("anchor");
    (dir, repo, deep)
}

/// The root ascends to the nearest anchor (ce.toml or .git), the
/// path itself first; an anchorless tree keeps what it was given —
/// the field-report counterexample was `cd background/` flipping
/// the same write's verdict.
#[test]
fn project_root_ascends_to_the_nearest_anchor() {
    let (dir, repo, deep) = anchored("root", "sub/deep");
    assert_eq!(project_root(&deep), repo, "ascends to ce.toml");
    assert_eq!(project_root(&repo), repo, "the path itself first");
    let (root, ascended) = resolve(&deep);
    assert_eq!(root, repo, "resolve agrees with project_root");
    assert!(ascended, "an ascent reports itself");
    assert!(!resolve(&repo).1, "no ascent, nothing to say");

    let loose = dir.join("loose");
    std::fs::create_dir_all(&loose).expect("mkdir");
    // the walk above `loose` may cross REAL anchors on the host
    // (temp dirs live under a user profile) — assert the honest
    // property instead: the answer is `loose` itself or one of
    // its ancestors carrying a real anchor, never a sibling
    let got = project_root(&loose);
    assert!(loose.starts_with(&got), "never leaves the ancestry line");
    // and from INSIDE it, a relative `.` resolves absolute and says
    // moved exactly when the root differs: the fallback used to
    // return the typed path itself, so `resolve(".")` in an
    // anchorless tree compared `/abs/here` against `.` and reported
    // an ascent that never happened — `ce baseline .` there was
    // refused as "inside a project" it was the root of (O30)
    let (root, moved) = resolved_from(&loose);
    let here = std::fs::canonicalize(&loose).expect("canon");
    assert_eq!(
        moved,
        root != here,
        "moved says exactly whether the root differs: {root:?}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// `resolve(".")` with the process cwd at `dir`, restored after —
/// the one stanza both relative-path tests need (the dedup gate
/// paired them), answered canonical: the temp dir may itself be
/// reached through a symlink (macOS /var -> /private/var).
fn resolved_from(dir: &Path) -> (PathBuf, bool) {
    let keep = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(dir).expect("cd");
    let (root, moved) = resolve(Path::new("."));
    std::env::set_current_dir(keep).expect("cd back");
    assert!(root.is_absolute(), "never a relative root: {root:?}");
    (std::fs::canonicalize(&root).expect("canon"), moved)
}

/// A relative path walks its ancestry too. `Path::new("cli")
/// .parent()` is `Some("")`, which used to end the walk after one
/// level — latent while only absolute hook cwds arrived here, live
/// the moment the CLI's `.`-shaped positionals came through.
#[test]
fn a_relative_path_still_ascends() {
    let (dir, repo, deep) = anchored("relroot", "sub");
    let (got, moved) = resolved_from(&deep);
    assert_eq!(got, std::fs::canonicalize(&repo).expect("canon"));
    assert!(moved, "an ascent reports itself");
    std::fs::remove_dir_all(&dir).ok();
}

/// A `.git` FILE anchors only when its pointer resolves — git's
/// own bar. A plain file and a dangling pointer are both writable
/// in one Write, and neither may re-root a hook.
#[test]
fn a_gitfile_anchors_only_when_its_target_resolves() {
    let dir = scratch("anchor");
    let sub = dir.join("sub");
    std::fs::create_dir_all(&sub).expect("mkdir");
    let gitfile = sub.join(".git");

    std::fs::write(&gitfile, "not a gitdir pointer").expect("plain");
    assert!(!is_git_anchor(&gitfile), "a plain file is not an anchor");

    std::fs::write(&gitfile, "gitdir: ../nowhere\n").expect("dangling");
    assert!(!is_git_anchor(&gitfile), "a pointer to nothing is not one");

    let real = dir.join("realgit");
    std::fs::create_dir_all(&real).expect("mkdir");
    std::fs::write(&gitfile, "gitdir: ../realgit\n").expect("relative");
    assert!(is_git_anchor(&gitfile), "a resolving pointer IS an anchor");

    std::fs::write(&gitfile, format!("gitdir: {}\n", real.display())).expect("absolute");
    assert!(is_git_anchor(&gitfile), "absolute targets resolve too");

    std::fs::remove_dir_all(&dir).ok();
}

/// A declared submodule roots at its superproject, seated or not
/// (the same answer before and after `git submodule update
/// --init`); an undeclared nested repository and a submodule
/// carrying its own ce.toml still root at themselves.
#[test]
fn a_declared_submodule_roots_at_its_superproject() {
    let (dir, repo, deep) = anchored("submod", "sub/deep");
    std::fs::create_dir_all(repo.join("realgit")).expect("mkdir");
    let declare = "[submodule \"s\"]\n\tpath = sub\n[submodule \"t\"]\n\tpath = sub2\n";
    std::fs::write(repo.join(".gitmodules"), declare).expect(".gitmodules");
    for sub in ["sub", "foreign", "sub2"] {
        std::fs::create_dir_all(repo.join(sub)).expect("mkdir");
        std::fs::write(repo.join(sub).join(".git"), "gitdir: ../realgit\n").expect("gitfile");
    }
    std::fs::write(repo.join("sub2/ce.toml"), "\n").expect("opt-out");
    assert_eq!(
        project_root(&repo.join("sub")),
        repo,
        "declared: the parent's"
    );
    assert_eq!(project_root(&deep), repo, "…all the way down");
    assert_eq!(
        project_root(&repo.join("foreign")),
        repo.join("foreign"),
        "undeclared escapes"
    );
    assert_eq!(
        project_root(&repo.join("sub2")),
        repo.join("sub2"),
        "own ce.toml opts out"
    );
    std::fs::remove_file(repo.join("sub/.git")).expect("deinit");
    assert_eq!(project_root(&deep), repo, "checkout-invariant");
    std::fs::remove_dir_all(&dir).ok();
}
