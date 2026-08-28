//! Resurrecting a retired instrument from history after the suite
//! moved into the cli/tests submodule (plan v2.18). The parent's own
//! history still holds every retired file as a blob, so the sanctioned
//! recipe extracts from it WITHOUT touching the index — `git show
//! <sha>:<path> > <path>` / `git archive <sha> <dir> | tar -x` — and
//! retires with a plain `rm`: the revived files are tracked by NEITHER
//! repository. An index-writing checkout aimed below the gitlink exits
//! 0 and silently REPLACES the gitlink with historical blobs in the
//! superproject index (measured 2026-08-28), after which the old `git
//! rm` retirement cannot put it back. Three legs: no documented command
//! writes the superproject index below a declared path (read off
//! `.gitmodules`, so a second submodule extends the gate by itself),
//! every extraction recipe resolves in the parent's history, and git
//! still behaves the way the wording relies on.

use crate::common;
use codeeraser::gitmodules;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Every markdown under docs/ and at the root, plus the suite's own
/// instrument headers — the corpus that carries recipes.
fn corpus(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.join("docs")];
    while let Some(dir) = stack.pop() {
        for p in std::fs::read_dir(&dir).expect("docs dir").flatten().map(|e| e.path()) {
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "md") {
                out.push(p);
            }
        }
    }
    for (dir, ext) in [(root.to_path_buf(), "md"), (root.join("cli/tests/it"), "rs")] {
        for p in std::fs::read_dir(&dir).expect("dir").flatten().map(|e| e.path()) {
            if p.is_file() && p.extension().is_some_and(|x| x == ext) {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// (display path, 1-based line, text) over the corpus.
fn lines_of(root: &Path) -> Vec<(String, usize, String)> {
    let mut out = Vec::new();
    for p in corpus(root) {
        let shown = p.strip_prefix(root).expect("under root").to_string_lossy().replace('\\', "/");
        let text = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{shown}: {e}"));
        out.extend(text.lines().enumerate().map(|(i, l)| (shown.clone(), i + 1, l.to_string())));
    }
    out
}

fn git_out(dir: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new("git").arg("-C").arg(dir).args(args).output().expect("git");
    (out.status.success(), String::from_utf8_lossy(&out.stdout).into_owned())
}

/// `git show <rev>:<path>` and `git archive <rev> <path>` on one line.
fn recipes(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (cmd, colon) in [("git show ", true), ("git archive ", false)] {
        let mut rest = text;
        while let Some(i) = rest.find(cmd) {
            rest = &rest[i + cmd.len()..];
            let mut toks = rest.split_whitespace().map(|t| t.trim_matches('`'));
            let (rev, path) = match (colon, toks.next(), toks.next()) {
                (true, Some(spec), _) => match spec.split_once(':') {
                    Some((r, p)) => (r, p),
                    None => continue,
                },
                (false, Some(r), Some(p)) => (r, p),
                _ => continue,
            };
            if rev.contains('<') || is_rev(rev) {
                out.push((rev.to_string(), path.to_string()));
            }
        }
    }
    out
}

/// A revision a recipe spells: hex with an optional `^`/`~N` suffix.
/// Prose and source that happen to say `git show` (this file's own
/// parser literal) are not recipes.
fn is_rev(rev: &str) -> bool {
    let bare = rev.split(['^', '~']).next().unwrap_or("");
    bare.len() >= 4 && bare.chars().all(|c| c.is_ascii_hexdigit())
}

/// Leg 1: no `git checkout`/`git rm` naming a declared submodule path
/// anywhere in the corpus — the two commands that write the parent's
/// index, and the two the old recipes used.
#[test]
fn no_documented_command_writes_the_superproject_index_below_a_gitlink() {
    let root = common::repo_root();
    let declared = gitmodules::declared(&root);
    assert!(!declared.is_empty(), "a gate with nothing to guard is vacuous");
    let below = |tok: &str| declared.iter().any(|s| tok == s || tok.starts_with(&format!("{s}/")));
    let mut offenders = Vec::new();
    for (file, n, text) in lines_of(&root) {
        for cmd in ["git checkout", "git rm"] {
            let Some(i) = text.find(cmd) else { continue };
            let tail = &text[i + cmd.len()..];
            if tail.split(|c: char| c.is_whitespace() || c == '`' || c == '"').any(below) {
                offenders.push(format!("{file}:{n}: {}", text.trim()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "index-writing commands aimed below a gitlink:\n{}",
        offenders.join("\n")
    );
}

/// Leg 2: every extraction recipe resolves in the parent's history —
/// the commit exists and the path is in its tree. Anti-vacuity: the
/// corpus carries at least the nine recipes it had when this landed.
#[test]
fn every_extraction_recipe_resolves_in_the_parents_history() {
    let root = common::repo_root();
    let mut seen = 0usize;
    for (file, n, text) in lines_of(&root) {
        for (rev, path) in recipes(&text) {
            if rev.contains('<') {
                continue; // a template placeholder, not a commit
            }
            seen += 1;
            let commit = format!("{rev}^{{commit}}");
            assert!(
                git_out(&root, &["rev-parse", "--verify", "-q", &commit]).0,
                "{file}:{n}: {rev} is not a commit of this repository"
            );
            let (ok, tree) = git_out(&root, &["ls-tree", "--name-only", &rev, "--", &path]);
            assert!(ok && !tree.trim().is_empty(), "{file}:{n}: {path} is not in {rev}'s tree");
        }
    }
    assert!(seen >= 9, "the recipe corpus went quiet ({seen}): the parser stopped matching");
}

/// Leg 3: the git semantics the wording relies on, on a superproject
/// whose first commit tracked `suite/f.rs` as a BLOB and whose second
/// re-homed it into a submodule — this repository's own shape at the
/// switch. Extraction leaves the gitlink; the index-writing checkout
/// replaces it, exit 0.
#[test]
fn extraction_leaves_the_gitlink_and_checkout_replaces_it() {
    let (sup, child) = (common::tmp("recipes-super"), common::tmp("recipes-child"));
    for (dir, rel) in [(&child, "f.rs"), (&sup, "suite/f.rs")] {
        let file = dir.join(rel);
        std::fs::create_dir_all(file.parent().expect("parent")).expect("mkdir");
        std::fs::write(file, common::rust_fn(1)).expect(rel);
        common::init_and_commit(dir, "seed");
    }
    let blob_era = git_out(&sup, &["rev-parse", "HEAD"]).1.trim().to_string();
    common::git(&sup, &["rm", "-rq", "suite"]);
    let url = child.to_str().expect("utf8").replace('\\', "/");
    common::git(&sup, &["-c", "protocol.file.allow=always", "submodule", "add", "-q", &url, "suite"]);
    common::commit_all(&sup, "submodule era");
    let gitlink = || git_out(&sup, &["ls-files", "-s", "suite"]).1;
    assert!(gitlink().starts_with("160000"), "{}", gitlink());

    let blob = git_out(&sup, &["show", &format!("{blob_era}:suite/f.rs")]).1;
    std::fs::write(sup.join("suite/g.rs"), blob).expect("extracted");
    assert!(gitlink().starts_with("160000"), "extraction left the gitlink: {}", gitlink());
    let status = git_out(&sup, &["status", "--porcelain"]).1;
    assert!(
        status.lines().all(|l| l.trim_start().starts_with("M suite")),
        "only the child's untracked content shows, as ` M suite`: {status:?}"
    );

    common::git(&sup, &["checkout", &blob_era, "--", "suite/f.rs"]);
    assert!(!gitlink().contains("160000"), "the checkout replaced the gitlink: {}", gitlink());
}
