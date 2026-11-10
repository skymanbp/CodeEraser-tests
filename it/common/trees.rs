//! Program trees as one literal (lanes v235_pyimport and
//! v235_graphtrust): trees split by `====` lines, each a header
//! `name|dead,…` — the files `ce deadcode` must report dead, comma-
//! separated, none for a program whose every file is live — then
//! `@path` lines, each followed by the file's text. One literal, not a
//! tuple table (rows of one shape rhyme under the clone gate), and one
//! reader for the legs that read it (the third copy was the gate's) —
//! with one judged reader (`judged`, `ce deadcode`'s dead rows) and one
//! plan reader (`reasons`) for the same reason.

use super::fixtures::{init_and_commit, tmp};
use codeeraser::erase;
use codeeraser::graph::deadcode;
use std::path::{Path, PathBuf};

/// A tree's files, `(path, text)`.
pub type Files = Vec<(String, String)>;

/// The trees of a literal: (name, dead files expected, files). A CRLF
/// checkout (Windows) reads the same trees.
pub fn trees(doc: &str) -> Vec<(String, Vec<String>, Files)> {
    doc.replace("\r\n", "\n")
        .split("====\n")
        .map(|block| {
            let (head, body) = block.split_once('\n').expect("header line");
            let (name, dead) = head.split_once('|').expect("name|dead");
            let files = body
                .split('@')
                .filter(|f| !f.is_empty())
                .map(|f| {
                    let (path, text) = f.split_once('\n').expect("path line");
                    (path.to_string(), text.to_string())
                })
                .collect();
            let dead = dead
                .split(',')
                .filter(|d| !d.is_empty())
                .map(str::to_string)
                .collect();
            (name.to_string(), dead, files)
        })
        .collect()
}

/// A tree written under a fresh tmp dir and committed as one repo.
pub fn seeded(name: &str, files: &[(String, String)]) -> PathBuf {
    let dir = tmp(name);
    for (path, text) in files {
        let at = dir.join(path);
        std::fs::create_dir_all(at.parent().expect("parent")).expect("dirs");
        std::fs::write(at, text).expect("write");
    }
    init_and_commit(&dir, "seed");
    dir
}

/// One tree of a literal after `ce deadcode`: its name, the dead set the
/// literal expects, its seeded directory, and the report's dead rows as
/// (path, confidence).
pub struct Judged {
    pub name: String,
    pub want: Vec<String>,
    pub dir: PathBuf,
    pub dead: Vec<(String, Option<i64>)>,
}

impl Judged {
    /// The dead rows' paths, in report order.
    pub fn paths(&self) -> Vec<&str> {
        self.dead.iter().map(|(p, _)| p.as_str()).collect()
    }
}

/// Every tree of `doc`, seeded under `<prefix>-<name>` and judged by
/// `ce deadcode` with the core at `core`.
pub fn judged(prefix: &str, doc: &str, core: &str) -> Vec<Judged> {
    trees(doc)
        .into_iter()
        .map(|(name, want, files)| {
            let dir = seeded(&format!("{prefix}-{name}"), &files);
            let report = deadcode::run(&dir, None, core).expect("deadcode");
            let dead = report
                .dead
                .iter()
                .map(|d| (d.path.clone(), d.conf))
                .collect();
            Judged {
                name,
                want,
                dir,
                dead,
            }
        })
        .collect()
}

/// Every tree of `doc` judged (`judged`), each one's dead set the one its
/// header names; the judged trees come back for the leg's own checks.
pub fn dead_sets_hold(prefix: &str, doc: &str, core: &str) -> Vec<Judged> {
    let all = judged(prefix, doc, core);
    for tree in &all {
        assert_eq!(tree.paths(), tree.want, "{}: dead set", tree.name);
    }
    all
}

/// The reason of every row of `ce erase`'s plan for the tree at `dir`.
pub fn reasons(dir: &Path, core: &str) -> Vec<&'static str> {
    let plan = erase::plan(dir, None, core).expect("plan");
    plan.rows.iter().map(|r| r.reason).collect()
}
