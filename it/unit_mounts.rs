//! Every `#[cfg(test)]` module of cli/src rides in the suite as
//! cli/tests/unit/<mirrored path> behind a `#[path]` mount (plan v2.18
//! step #13). Three sets must coincide, or "the score prices product
//! code alone and the crate still tests itself" breaks silently: the
//! mounts src declares, the files unit/ holds, and the files `cargo
//! package` ships. A unit file no source mounts is never compiled —
//! the suite's own graph lists unit/ as entries, so its deadcode gate
//! cannot see the orphan; a mount without its file fails only under
//! cfg(test), which `cargo publish`'s verify build never enables
//! (both raised by the step's adversarial review).

use crate::common;
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

const UNIT: &str = "cli/tests/unit/";

/// Repo-relative, `/`-joined on every platform.
fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .expect("under the repo root")
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// `#[path = "<rel>"]` resolves from the declaring file's directory
/// (mod-rs and non-mod-rs files alike); `..` folds.
fn resolve(from: &Path, rel: &str) -> PathBuf {
    let mut out = from.parent().expect("a file has a directory").to_path_buf();
    for c in Path::new(rel).components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(n) => out.push(n),
            _ => {}
        }
    }
    out
}

/// The mounts one source file declares. Any `#[cfg(test)]` that is
/// not the three-line shape `#[cfg(test)] / #[path = "…"] / mod x;`
/// is a test body that crept back into src — reported, never mounted.
fn mounts_in(file: &Path, text: &str, stray: &mut Vec<String>) -> Vec<PathBuf> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if *l != "#[cfg(test)]" {
            continue;
        }
        let path = lines
            .get(i + 1)
            .and_then(|p| p.strip_prefix("#[path = \""))
            .and_then(|p| p.strip_suffix("\"]"));
        let is_mod = lines.get(i + 2).is_some_and(|m| {
            m.trim_start_matches("pub(crate) ").starts_with("mod ") && m.ends_with(';')
        });
        match path {
            Some(p) if is_mod => out.push(resolve(file, p)),
            _ => stray.push(format!("{}:{}", file.display(), i + 1)),
        }
    }
    out
}

fn declared(root: &Path) -> BTreeSet<String> {
    let mut files = Vec::new();
    common::files_with_ext(&root.join("cli/src"), "rs", &mut files);
    let (mut out, mut stray) = (BTreeSet::new(), Vec::new());
    for f in &files {
        let text = std::fs::read_to_string(f).expect("source file");
        for m in mounts_in(f, &text, &mut stray) {
            out.insert(rel(root, &m));
        }
    }
    assert!(
        stray.is_empty(),
        "a #[cfg(test)] in cli/src that is not a unit/ mount:\n{}",
        stray.join("\n")
    );
    out
}

fn on_disk(root: &Path) -> BTreeSet<String> {
    let mut files = Vec::new();
    common::files_with_ext(&root.join(UNIT), "rs", &mut files);
    files.iter().map(|f| rel(root, f)).collect()
}

fn diff(a: &BTreeSet<String>, b: &BTreeSet<String>) -> String {
    let only = |x: &BTreeSet<String>, y: &BTreeSet<String>| {
        x.difference(y).cloned().collect::<Vec<_>>().join("\n  ")
    };
    format!(
        "only left:\n  {}\nonly right:\n  {}",
        only(a, b),
        only(b, a)
    )
}

#[test]
fn every_mount_has_its_file_and_every_file_its_mount() {
    let root = common::repo_root();
    let (mounts, files) = (declared(&root), on_disk(&root));
    let outside: Vec<&String> = mounts.iter().filter(|m| !m.starts_with(UNIT)).collect();
    assert!(
        outside.is_empty(),
        "a cfg(test) mount outside {UNIT}: {outside:?}"
    );
    assert!(!mounts.is_empty(), "no mount found: the parser went quiet");
    assert!(
        mounts == files,
        "src mounts != unit/ files\n{}",
        diff(&mounts, &files)
    );
}

/// `cargo package --list` is the tarball's inventory without building
/// it: the suite ships exactly the mounted files (a consumer's
/// `cargo test` must compile) and nothing else of it (fixtures,
/// integration tests, the product's own .ce/ state never ride along).
#[test]
fn the_crate_packages_exactly_the_mounted_files() {
    let root = common::repo_root();
    let out = std::process::Command::new(env!("CARGO"))
        .args([
            "package",
            "--list",
            "--locked",
            "--allow-dirty",
            "--manifest-path",
        ])
        .arg(root.join("cli/Cargo.toml"))
        .output()
        .expect("cargo package --list");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let shipped: BTreeSet<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().replace('\\', "/"))
        .filter(|l| l.starts_with("tests/"))
        .map(|l| format!("cli/{l}"))
        .collect();
    let stray: Vec<&String> = shipped.iter().filter(|s| !s.starts_with(UNIT)).collect();
    assert!(
        stray.is_empty(),
        "the tarball ships suite files outside {UNIT}: {stray:?}"
    );
    let files = on_disk(&root);
    assert!(
        !files.is_empty(),
        "no unit file on disk: the walk went quiet"
    );
    assert!(
        shipped == files,
        "packaged unit files != unit/ files\n{}",
        diff(&shipped, &files)
    );
}
