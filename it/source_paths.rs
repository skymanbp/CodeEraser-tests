//! A comment that names a path under `cli/src/` or `core/app/` names a
//! file or directory that exists, or says when it existed.
//!
//! `source_citations` reads the citation shape only (a file, a line
//! span, a quoted anchor), so a comment pointing at a file by bare path
//! was outside every gate: eleven core modules went on describing a
//! deleted Rust file in the present tense, and a GUI comment named two
//! console files that had moved into the core (release 1.10.0, items
//! H10 and H17).
//!
//! The rule, exact: on a comment line (Rust and JavaScript `//`, `/*`,
//! `*`; Haskell `--`, `{-`) of a tracked file under `cli/src`,
//! `core/app` or `gui`, every path-shaped token under one of the two
//! roots is a tracked file or a tracked directory, or the comment block
//! holding it (the run of comment lines around it) names a commit at
//! which the path existed or the commit that removed it (its parent
//! had it). A trailing comment after code on the same line is not
//! read: the token there may sit in a string.

use crate::common;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::process::Command;

/// The trees walked, by tracked path prefix.
const WALKED: [&str; 3] = ["cli/src/", "core/app/", "gui/"];

/// Whether `line` is a comment line of a file with extension `ext`.
fn is_comment(ext: &str, line: &str) -> bool {
    let t = line.trim_start();
    match ext {
        "hs" => t.starts_with("--") || t.starts_with("{-"),
        "rs" | "js" => t.starts_with("//") || t.starts_with("/*") || t.starts_with('*'),
        _ => false,
    }
}

/// Every path-shaped token under the two roots on one line: the root,
/// then path characters, ending on a letter, digit or `_` (a sentence's
/// full stop is not part of a path), and not glued to a path on its left.
fn paths_on(line: &str) -> Vec<String> {
    let path_char = |c: char| c.is_ascii_alphanumeric() || "_./-".contains(c);
    let mut out = Vec::new();
    for root in ["cli/src/", "core/app/"] {
        for (i, _) in line.match_indices(root) {
            if line[..i].chars().next_back().is_some_and(path_char) {
                continue;
            }
            let rest = &line[i..];
            let end = rest.find(|c: char| !path_char(c)).unwrap_or(rest.len());
            let token =
                rest[..end].trim_end_matches(|c: char| !(c.is_ascii_alphanumeric() || c == '_'));
            out.push(token.to_string());
        }
    }
    out
}

/// The commit-shaped words (7 to 40 lowercase hex digits) of a block.
fn commits_in(block: &str) -> Vec<&str> {
    block
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| {
            (7..=40).contains(&w.len())
                && w.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
        .collect()
}

fn git(root: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("git");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// The tracked files and every directory above one.
fn tracked(root: &Path) -> (Vec<String>, BTreeSet<String>) {
    let (ok, text) = git(root, &["ls-files"]);
    assert!(ok, "git ls-files");
    let files: Vec<String> = text.lines().map(String::from).collect();
    let mut known: BTreeSet<String> = files.iter().cloned().collect();
    for f in &files {
        let mut at = f.as_str();
        while let Some((dir, _)) = at.rsplit_once('/') {
            known.insert(dir.to_string());
            at = dir;
        }
    }
    (files, known)
}

/// Every stale path mention of one file's text: `site: path`.
fn stale_in(
    rel: &str,
    text: &str,
    known: &BTreeSet<String>,
    dated: &mut impl FnMut(&str, &str) -> bool,
) -> Vec<String> {
    let ext = rel.rsplit('.').next().unwrap_or("");
    let lines: Vec<&str> = text.lines().collect();
    let comment = |i: usize| is_comment(ext, lines[i]);
    let mut bad = Vec::new();
    for i in (0..lines.len()).filter(|&i| comment(i)) {
        for p in paths_on(lines[i]) {
            if known.contains(&p) {
                continue;
            }
            let lo = (0..i).rev().take_while(|&j| comment(j)).last().unwrap_or(i);
            let hi = (i + 1..lines.len())
                .take_while(|&j| comment(j))
                .last()
                .unwrap_or(i);
            let block = lines[lo..=hi].join("\n");
            if !commits_in(&block).iter().any(|c| dated(c, &p)) {
                bad.push(format!("{rel}:{}: {p}", i + 1));
            }
        }
    }
    bad
}

#[test]
fn every_path_a_comment_names_exists_or_is_dated() {
    let root = common::repo_root();
    let (files, known) = tracked(&root);
    let mut seen: HashMap<(String, String), bool> = HashMap::new();
    let mut dated = |c: &str, p: &str| {
        *seen
            .entry((c.to_string(), p.to_string()))
            .or_insert_with(|| {
                [format!("{c}:{p}"), format!("{c}^:{p}")]
                    .iter()
                    .any(|spec| git(&root, &["cat-file", "-e", spec]).0)
            })
    };
    let (mut walked, mut bad) = (0, Vec::new());
    for rel in files
        .iter()
        .filter(|f| WALKED.iter().any(|w| f.starts_with(w)))
    {
        let Ok(text) = std::fs::read_to_string(root.join(rel)) else {
            continue;
        };
        walked += 1;
        bad.extend(stale_in(rel, &text, &known, &mut dated));
    }
    assert!(walked > 300, "walked {walked} files");
    assert!(
        bad.is_empty(),
        "comments naming paths that do not exist:\n{}",
        bad.join("\n")
    );
}

/// The shapes the rule separates, on a synthetic file: a tracked path,
/// a tracked directory, a sentence's full stop, a gone path with no
/// commit, the same path dated in its block, and a trailing comment.
#[test]
fn the_rule_separates_its_shapes() {
    let known: BTreeSet<String> = ["cli/src/a.rs", "cli/src", "cli"].map(String::from).into();
    let mut dated = |c: &str, p: &str| c == "abc1234" && p == "core/app/Gone.hs";
    let text = "\
// see cli/src/a.rs and the tree cli/src.
fn f() {} // cli/src/trailing.rs
// core/app/Gone.hs is gone

// moved from core/app/Gone.hs
// at abc1234
-- core/app/Other.hs
";
    let got = stale_in("x.rs", text, &known, &mut dated);
    assert_eq!(got, ["x.rs:3: core/app/Gone.hs"]);
    let hs = stale_in(
        "x.hs",
        "-- core/app/Other.hs\n{- cli/src/a.rs -}\n",
        &known,
        &mut dated,
    );
    assert_eq!(hs, ["x.hs:1: core/app/Other.hs"]);
}
