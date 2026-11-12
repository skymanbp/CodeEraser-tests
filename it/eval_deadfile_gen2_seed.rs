//! The history G9's end-to-end leg replays (lane v235_licence, part 4B),
//! seeded commit by commit in a temp directory: what each commit holds,
//! written out as a git repository. The leg itself is
//! eval_deadfile_gen2_e2e.rs.

use crate::common::gitio::{git, git_ok};
use crate::common::write_doc;
use std::path::{Path, PathBuf};

/// The seeded history: `--- path` opens a file body (written at the next
/// `=== commit`), `=== rm p`, `=== mv a b`, `=== commit` and
/// `=== link name target` (a symbolic link entry, mode 120000, written
/// into the index and committed at once: `add -A` would stage its
/// removal, the link being no file of the work tree). Commit k is the
/// k-th commit made.
const SEED: &str = "\
--- README.md
# Readme

See [the reference](ref.md).
--- ref.md
# Reference

one
two
three
four
five
six
seven
eight
=== commit
--- \u{fc}ber.md
# Uber

nobody links this page
--- gone.md
# Gone

nobody links this page either
--- zap.md
# Zap

deleted at the refused commit
--- old.md
# Old

alpha
beta
gamma
delta
epsilon
zeta
eta
theta
--- e1.md
# Edited

alpha
beta
gamma
delta
epsilon
zeta
eta
theta
--- p1.md
# Page one

See [the section](p2.md#sec).
--- p2.md
# Page two

## sec

reached only through a section link from a dead page
=== commit
=== mv old.md new.md
=== commit
=== mv e1.md e2.md
--- e2.md
# Edited

alpha
beta
gamma
delta
epsilon
zeta
eta
theta
iota
=== commit
--- README.md
# Readme

No links any more.
=== commit
=== rm gone.md
=== commit
--- ce.toml
[thresholds
=== rm zap.md
=== commit
=== rm ce.toml
=== commit
--- README.md
# Readme

See [the reference](ref.md) again.
=== commit
=== link link.md ref.md
";

/// One `=== ` command of the seed.
fn command(dir: &Path, cmd: &str, commits: &mut usize) {
    let w: Vec<&str> = cmd.split_whitespace().collect();
    match w[..] {
        ["commit"] => {
            git(dir, &["add", "-A"]);
            git(
                dir,
                &[
                    "commit",
                    "-q",
                    "--allow-empty",
                    "-m",
                    &format!("c{commits}"),
                ],
            );
            *commits += 1;
        }
        ["rm", p] => git(dir, &["rm", "-q", p]),
        ["mv", a, b] => git(dir, &["mv", a, b]),
        ["link", name, target] => {
            let oid = hash_blob(dir, target);
            let info = format!("120000,{oid},{name}");
            git(dir, &["update-index", "--add", "--cacheinfo", &info]);
            git(dir, &["commit", "-q", "-m", &format!("c{commits}")]);
            *commits += 1;
        }
        _ => panic!("seed command {cmd:?}"),
    }
}

fn hash_blob(dir: &Path, text: &str) -> String {
    let tmp = dir.join(".git").join("link-target");
    std::fs::write(&tmp, text).expect("write");
    let oid = git_ok(dir, &["hash-object", "-w", &tmp.to_string_lossy()]);
    oid.trim().to_string()
}

/// The seed written out commit by commit; the tip's sha. The file
/// sections between two commands are one fixture document.
pub fn seed(dir: &Path) -> String {
    git(dir, &["init", "-q"]);
    let (mut commits, mut files) = (0usize, String::new());
    for line in SEED.lines() {
        match line.strip_prefix("=== ") {
            Some(cmd) => {
                write_doc(dir, &files);
                files.clear();
                command(dir, cmd, &mut commits);
            }
            None => {
                files.push_str(line);
                files.push('\n');
            }
        }
    }
    git_ok(dir, &["rev-parse", "HEAD"]).trim().to_string()
}

/// The seed in `<tmp>/<name>/seed`, the copy's path beside it, the tip.
pub fn seeded(name: &str) -> (PathBuf, PathBuf, String) {
    let root = crate::common::tmp(name);
    let (seeded, copy) = (root.join("seed"), root.join("copy"));
    std::fs::create_dir_all(&seeded).expect("mkdir");
    let tip = seed(&seeded);
    (seeded, copy, tip)
}
