//! The replay (BARS-GEN2 section 2): one corpus's first-parent history
//! read into the model — every tree from the window's first parent to
//! the tip judged by this build's `ce deadcode`, every step read off
//! `git diff -M --name-status` — on a CLONE of the registered clone
//! (the registered clone is read-only), checked out commit by commit.
//! Nothing here counts; model.rs reads the events and fates, readings.rs
//! the shares.

use super::model::{Dead, FUTURE, History, Step, Tree, WINDOW};
use crate::common::gitio::git_ok;
use crate::common::{first_parent, git_lines};
use codeeraser::scan::lang::Lang;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The product's word for a documentation file: a file of a document
/// language (Markdown, HTML) or plain prose.
pub fn document(path: &str) -> bool {
    matches!(
        Lang::from_path(Path::new(path)),
        Some(Lang::Markdown | Lang::Html | Lang::Text)
    )
}

/// One parsed `git diff -M --name-status` line into a step.
pub fn read_status(step: &mut Step, line: &str) {
    let cols: Vec<&str> = line.split('\t').collect();
    let status = cols.first().copied().unwrap_or("");
    let letter = status.chars().next().unwrap_or(' ');
    step.touched += 1;
    match (letter, cols.as_slice()) {
        ('R', [_, from, to]) => {
            step.renamed.insert(to.to_string(), from.to_string());
            if status != "R100" {
                step.changed.insert(to.to_string());
            }
        }
        ('C', [_, _, to]) => drop(step.changed.insert(to.to_string())),
        ('D', [_, p]) => drop(step.deleted.insert(p.to_string())),
        (_, [_, p]) => drop(step.changed.insert(p.to_string())),
        _ => panic!("unread name-status line {line:?}"),
    }
}

/// The step from `p` to `c`.
pub fn step(repo: &Path, p: &str, c: &str) -> Step {
    let mut s = Step::default();
    for line in git_lines(repo, &["diff", "-M", "--name-status", p, c]) {
        read_status(&mut s, &line);
    }
    s
}

/// The dead map of one `ce deadcode --format json` document: a root is
/// an `unref_*` verdict (no reference from any file of the tree).
pub fn dead_of(doc: &Value) -> BTreeMap<String, Dead> {
    doc["dead"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|r| {
            let path = r["name"].as_str().expect("name").to_string();
            let root = r["verdict"]
                .as_str()
                .is_some_and(|v| v.starts_with("unref"));
            let d = Dead {
                root,
                document: document(&path),
            };
            (path, d)
        })
        .collect()
}

/// One commit's tree: checked out in the copy, judged by this build.
fn tree(copy: &Path, sha: &str) -> Tree {
    git_ok(copy, &["checkout", "-q", "-f", "--detach", sha]);
    git_ok(copy, &["clean", "-q", "-ffdx", "-e", ".ce"]);
    let files: BTreeSet<String> = git_lines(copy, &["ls-tree", "-r", "--name-only", sha])
        .into_iter()
        .collect();
    let out = crate::common::ce_command(copy, &[])
        .args(["deadcode", "--format", "json", "."])
        .output()
        .expect("run ce deadcode");
    let doc: Option<Value> = serde_json::from_slice(&out.stdout).ok();
    let judged =
        out.status.code() != Some(2) && doc.as_ref().is_some_and(|d| d["degraded"].is_null());
    let dead = doc
        .filter(|_| judged)
        .map(|d| dead_of(&d))
        .unwrap_or_default();
    Tree {
        files,
        dead,
        judged,
    }
}

/// The first-parent chain the replay reads, oldest first: from the
/// window's first parent to the tip.
pub fn chain(repo: &Path, tip: &str) -> Vec<String> {
    let mut all = first_parent(repo, tip);
    all.reverse();
    let take = (WINDOW + FUTURE + 1).min(all.len());
    all.split_off(all.len() - take)
}

/// The corpus's history: a fresh copy of the registered clone, every
/// tree of the chain and every step between.
pub fn history(clone: &Path, copy: &Path, tip: &str) -> History {
    if copy.exists() {
        std::fs::remove_dir_all(copy).expect("remove the earlier copy");
    }
    let (from, to) = (clone.to_string_lossy(), copy.to_string_lossy());
    git_ok(
        Path::new("."),
        &["clone", "-q", "--no-checkout", &from, &to],
    );
    let shas = chain(copy, tip);
    let mut h = History {
        shas: shas.clone(),
        ..History::default()
    };
    for (i, sha) in shas.iter().enumerate() {
        h.steps.push(if i == 0 {
            Step::default()
        } else {
            step(copy, &shas[i - 1], sha)
        });
        h.trees.push(tree(copy, sha));
    }
    h
}
