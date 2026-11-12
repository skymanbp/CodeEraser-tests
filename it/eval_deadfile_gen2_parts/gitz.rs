//! The replay's git reads, NUL-separated (G8): without `-z` git quotes a
//! non-ASCII path and the product's JSON does not, so a dead root with
//! such a name read as born at every commit. Modes come from the full
//! `ls-tree` format (G16): a gitlink (160000) is no file of the tree, a
//! symbolic link (120000) is listed apart. Steps read `git diff -M -l0
//! --name-status -z` (ruling: `-l0` is "git's rename detection at its
//! default similarity" — the similarity is git's default, the search
//! never gives up on a large commit, and the result does not depend on
//! this machine's configuration).

use super::model::Step;
use crate::common::gitio::git_ok;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

/// A tree's paths: its files (every blob, symbolic links included) and,
/// apart, the symbolic links.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Listing {
    pub files: BTreeSet<String>,
    pub symlinks: BTreeSet<String>,
}

/// One `ls-tree -r -z` output read into a listing:
/// `<mode> SP <type> SP <object> TAB <path>` records, NUL-ended.
pub fn read_listing(raw: &str) -> Listing {
    let mut l = Listing::default();
    for rec in raw.split('\0').filter(|r| !r.is_empty()) {
        let (meta, path) = rec
            .split_once('\t')
            .unwrap_or_else(|| panic!("unread ls-tree record {rec:?}"));
        match meta.split(' ').next() {
            Some("160000") => {}
            Some("120000") => {
                l.symlinks.insert(path.to_string());
                l.files.insert(path.to_string());
            }
            _ => drop(l.files.insert(path.to_string())),
        }
    }
    l
}

pub fn ls_tree(repo: &Path, sha: &str) -> Listing {
    read_listing(&git_ok(repo, &["ls-tree", "-r", "-z", sha]))
}

/// One `git diff -M -l0 --name-status -z` output into a step: with `-z`
/// a rename or copy is `R<score>\0from\0to\0` (`C` likewise), any other
/// status `X\0path\0`.
pub fn read_statuses(step: &mut Step, raw: &str) {
    let mut tok = raw.split('\0').filter(|t| !t.is_empty());
    while let Some(status) = tok.next() {
        let mut path = || {
            tok.next()
                .unwrap_or_else(|| panic!("status {status:?} without its path"))
                .to_string()
        };
        let first = path();
        step.touched += 1;
        match status.chars().next() {
            Some('R') => {
                let to = path();
                if status != "R100" {
                    step.changed.insert(to.clone());
                }
                step.renamed.insert(to, first);
            }
            Some('C') => drop(step.changed.insert(path())),
            Some('D') => drop(step.deleted.insert(first)),
            Some(_) => drop(step.changed.insert(first)),
            None => panic!("empty status"),
        }
    }
}

/// The step from `p` to `c`.
pub fn step(repo: &Path, p: &str, c: &str) -> Step {
    let mut s = Step::default();
    let raw = git_ok(repo, &["diff", "-M", "-l0", "--name-status", "-z", p, c]);
    read_statuses(&mut s, &raw);
    s
}

/// `git checkout -f --detach sha` in the copy; a tree git cannot check out
/// here (G17: a name this file system refuses) is `checkout: <first
/// stderr line>`, not the end of the run.
pub fn checkout(copy: &Path, sha: &str) -> Result<(), String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(copy)
        .args(["checkout", "-q", "-f", "--detach", sha])
        .output()
        .expect("git");
    if out.status.success() {
        return Ok(());
    }
    let err = String::from_utf8_lossy(&out.stderr);
    let first = err
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    Err(format!("checkout: {first}"))
}

/// R1c, F1: the first pair in path order of two spellings that fold to
/// one (the key is the whole spelling's `to_lowercase()`) among the
/// tree's paths and every ancestor directory of each, as the reason that
/// leaves the tree unjudged: a case-insensitive file system holds one of
/// the two, and the product would read a tree that is not the
/// repository's.
pub fn case_clash(l: &Listing) -> Option<String> {
    let spellings: BTreeSet<&str> = l
        .files
        .iter()
        .flat_map(|p| {
            p.match_indices('/')
                .map(|(i, _)| &p[..i])
                .chain([p.as_str()])
        })
        .collect();
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for s in spellings {
        if let Some(a) = seen.insert(s.to_lowercase(), s) {
            return Some(format!("checkout: paths differ only by case: {a} | {s}"));
        }
    }
    None
}

/// The attributes written into every copy: `info/attributes` comes first,
/// so no `.gitattributes` of the corpus converts, smudges or expands a
/// byte at checkout (R1a).
pub const NO_CONVERSION: &str = "* -text -eol -filter -ident -working-tree-encoding\n";

/// A copy of a clone that checks out the bytes the repository holds:
/// no line-ending conversion (the machine's or the corpus's own), long
/// paths allowed (G10), and a symbolic link a small text file on every
/// platform (R1b).
pub fn clone_copy(clone: &Path, copy: &Path) {
    if copy.exists() {
        std::fs::remove_dir_all(copy).expect("remove the earlier copy");
    }
    let (from, to) = (clone.to_string_lossy(), copy.to_string_lossy());
    let mut args = vec!["clone", "-q", "--no-checkout"];
    for c in [
        "core.autocrlf=false",
        "core.longpaths=true",
        "core.symlinks=false",
    ] {
        args.extend(["-c", c]);
    }
    args.extend([&*from, &*to]);
    git_ok(Path::new("."), &args);
    let info = copy.join(".git").join("info");
    std::fs::create_dir_all(&info).expect("mkdir .git/info");
    std::fs::write(info.join("attributes"), NO_CONVERSION).expect("write info/attributes");
}
