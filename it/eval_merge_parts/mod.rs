//! The clone-merge family's frozen instruments (plan v2.31 step 7,
//! design booklet §6.5): the suggestion set `ce merge` answers over
//! this repository at a pinned commit and over the four pinned external
//! corpora, frozen row by row; the hash-ranked sample of it a blind
//! auditor judges; and the precision the audit reads. One binding for
//! the gates (eval_merge_suggestions.rs, eval_merge_review.rs) and the
//! `#[ignore]` generators. No RNG, no clock: ranks are the shared
//! domain-separated identity hash, seats the shared largest remainder.

pub mod member;
pub mod sample;

use member::members_of;

use crate::common;
use crate::eval_support::corpus::{PINNED_CORPORA, pinned_root};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const SCHEMA: &str = "ce.eval-merge-suggestions/1.0.0";
pub const DOC: &str = "contracts/eval/merge-suggestions-v1.json";

/// This repository as the self corpus: its name, the commit whose tree
/// is measured, and the form the tree takes (A9's `arch-self` reading).
pub const SELF: (&str, &str) = ("codeeraser", "f2a7a2b4c2f52b8b919cec12d1bacd036b2d2058");
pub const SELF_FORM: &str = "git archive, .gitmodules removed";

pub fn repo_file(rel: &str) -> PathBuf {
    common::repo_root().join(rel)
}

pub fn load(rel: &str) -> Option<Value> {
    let text = std::fs::read_to_string(repo_file(rel)).ok()?;
    Some(serde_json::from_str(&text).unwrap_or_else(|e| panic!("{rel}: {e}")))
}

pub fn write(rel: &str, doc: &Value) {
    let text = serde_json::to_string_pretty(doc).expect("json") + "\n";
    std::fs::write(repo_file(rel), text).unwrap_or_else(|e| panic!("{rel}: {e}"));
}

/// The frozen set as text: every field of a corpus on one line, its
/// rows one per line — a drift reads as the rows it moved, and the set
/// stays a fraction of its pretty size.
pub fn write_lined(rel: &str, doc: &Value) {
    let corpora: Vec<String> = doc["corpora"]
        .as_array()
        .expect("corpora")
        .iter()
        .map(corpus_lined)
        .collect();
    let schema = line(&doc["schema"]);
    let text = format!(
        "{{\"schema\": {schema},\n\"corpora\": [\n{}\n]}}\n",
        corpora.join(",\n")
    );
    std::fs::write(repo_file(rel), text).unwrap_or_else(|e| panic!("{rel}: {e}"));
}

fn line(v: &Value) -> String {
    serde_json::to_string(v).expect("json")
}

/// One corpus: each field but the rows on its own line, then the rows.
fn corpus_lined(c: &Value) -> String {
    let mut fields: Vec<String> = c
        .as_object()
        .expect("corpus")
        .iter()
        .filter(|(k, _)| *k != "rows")
        .map(|(k, v)| format!("  {}: {}", line(&json!(k)), line(v)))
        .collect();
    let rows: Vec<String> = c["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(line)
        .collect();
    fields.push(format!("  \"rows\": [\n{}\n  ]", rows.join(",\n")));
    format!("{{\n{}\n}}", fields.join(",\n"))
}

/// The pinned commit's tree, unpacked from `git archive` into a fresh
/// fixture dir with `.gitmodules` removed: no submodule is seated in an
/// archive, and a declared one unseated is a named refusal.
pub fn self_tree(name: &str) -> PathBuf {
    let dir = common::tmp(name);
    let mut archive = Command::new("git")
        .arg("-C")
        .arg(common::repo_root())
        .args(["archive", "--format=tar", SELF.1])
        .stdout(Stdio::piped())
        .spawn()
        .expect("git archive");
    let unpacked = Command::new("tar")
        .args(["-x", "-f", "-"])
        .current_dir(&dir)
        .stdin(archive.stdout.take().expect("piped"))
        .status()
        .expect("tar");
    assert!(
        archive.wait().expect("git").success(),
        "git archive {}",
        SELF.1
    );
    assert!(unpacked.success(), "tar -x");
    let _ = std::fs::remove_file(dir.join(".gitmodules"));
    dir
}

/// Every corpus the set freezes: the self tree, then the four pinned
/// external checkouts (tips verified).
pub fn corpora() -> Vec<(String, Value, PathBuf)> {
    let mut out = vec![(
        SELF.0.to_string(),
        json!({"repo": "skymanbp/CodeEraser", "commit": SELF.1, "form": SELF_FORM}),
        self_tree("merge-eval-self"),
    )];
    for (name, tip) in PINNED_CORPORA {
        let form = json!({"repo": name, "commit": tip, "form": "pinned checkout"});
        out.push((name.to_string(), form, pinned_root(name, tip)));
    }
    out
}

/// The merge document over a tree.
pub fn measure(root: &Path) -> Value {
    codeeraser::faces::merge(root, &common::core_bin()).expect("merge face")
}

/// One corpus's frozen section: the counts, the groups not sent, and a
/// row per suggestion ordered by (family, members).
pub fn section(doc: &Value) -> Value {
    assert!(
        doc["degraded"].is_null(),
        "a degraded document freezes nothing: {}",
        doc["degraded"]
    );
    let mut rows: Vec<Value> = doc["groups"]
        .as_array()
        .expect("groups")
        .iter()
        .map(|g| {
            json!({
                "members": members_of(g), "family": g["family"], "fragment": g["fragment"],
                "params": g["params"], "kept": g["kept"], "savings": g["savings"],
                "feasible": g["feasible"], "reason": g["reason"],
                "holes": g["holes"].as_array().expect("holes").len(),
            })
        })
        .collect();
    rows.sort_by_key(|r| (r["family"].to_string(), r["members"].to_string()));
    json!({"counts": doc["counts"], "unsendable": doc["unsendable"], "rows": rows})
}

/// A frozen corpus entry by name.
pub fn frozen(doc: &Value, name: &str) -> Value {
    let corpora = doc["corpora"].as_array().expect("corpora");
    let at = corpora.iter().position(|c| c["name"] == name);
    corpora[at.unwrap_or_else(|| panic!("{name}: not frozen"))].clone()
}
