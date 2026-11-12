//! The two runs on the real corpora, both `#[ignore]` and both refused
//! until the registry names the product commit (part 4A landed, the
//! main session's word): the tips (section 1's second replacement
//! reason, read with the product build) and the replay itself. Neither
//! writes the registry; the replay writes its doc to CE_DEADFILE_GEN2_OUT.
//!   CE_DEADFILE_GEN2_ROOT=<clones> CE_DEADFILE_GEN2_COPIES=<scratch dir> cargo test --test it -- --ignored eval_deadfile_gen2_parts::run::deadfile_gen2_tips --nocapture
//!   CE_DEADFILE_GEN2_ROOT=<clones> CE_DEADFILE_GEN2_COPIES=<scratch dir> CE_DEADFILE_GEN2_OUT=<dir> cargo test --test it -- --ignored eval_deadfile_gen2_parts::run::deadfile_gen2_replay --nocapture

use super::model::{State, replayed};
use super::replay::history;
use super::report::fate_name;
use super::{dir_name, product_commit, registry};
use crate::eval_lang_parts::generate::env;
use serde_json::{Value, json};
use std::path::Path;

/// The registry, refused while its product commit is unset.
fn armed() -> Value {
    let reg = registry();
    assert!(
        product_commit(&reg).is_some(),
        "the registry names no product commit: generation 2 runs only on the build of the landed part 4A"
    );
    reg
}

fn rows(reg: &Value) -> Vec<Value> {
    reg["corpora"].as_array().cloned().unwrap_or_default()
}

/// Each chosen corpus's tip judged by this build: the exit code of
/// `ce deadcode` (2 = refused, section 1's replacement reason).
#[test]
#[ignore = "runs ce deadcode on the registered clones; armed by the registry's product commit"]
fn deadfile_gen2_tips() {
    let reg = armed();
    for row in rows(&reg) {
        let (clone, copy) = (
            Path::new(&env("CE_DEADFILE_GEN2_ROOT")).join(dir_name(&row)),
            Path::new(&env("CE_DEADFILE_GEN2_COPIES")).join(format!("{}-tip", dir_name(&row))),
        );
        let tip = row["tip"].as_str().expect("tip");
        let judged = tip_judged(&clone, &copy, tip);
        println!("{} {} judged {}", row["language"], tip, judged);
    }
}

fn tip_judged(clone: &Path, copy: &Path, tip: &str) -> bool {
    if copy.exists() {
        std::fs::remove_dir_all(copy).expect("remove the earlier copy");
    }
    let (from, to) = (clone.to_string_lossy(), copy.to_string_lossy());
    crate::common::gitio::git_ok(
        Path::new("."),
        &["clone", "-q", "--no-checkout", &from, &to],
    );
    crate::common::gitio::git_ok(copy, &["checkout", "-q", "-f", "--detach", tip]);
    let out = crate::common::ce_command(copy, &[])
        .args(["deadcode", "--format", "json", "."])
        .output()
        .expect("run ce deadcode");
    out.status.code() != Some(2)
}

/// The replay: per chosen corpus every event of the window with its
/// fate, the unjudged trees and the dependents, into one doc. The
/// tables and the bars are read off that doc (readings.rs), never here.
#[test]
#[ignore = "replays the registered clones' histories; armed by the registry's product commit"]
fn deadfile_gen2_replay() {
    let reg = armed();
    let mut corpora = Vec::new();
    for row in rows(&reg) {
        let name = dir_name(&row).to_string();
        let (clone, copy) = (
            Path::new(&env("CE_DEADFILE_GEN2_ROOT")).join(&name),
            Path::new(&env("CE_DEADFILE_GEN2_COPIES")).join(&name),
        );
        let h = history(&clone, &copy, row["tip"].as_str().expect("tip"));
        let (events, dependents) = replayed(&h);
        let unjudged = h.trees.iter().filter(|t| !t.judged).count();
        let list: Vec<Value> = events
            .iter()
            .map(|(e, f)| {
                let state = if e.state == State::Orphaned {
                    "orphaned"
                } else {
                    "born"
                };
                json!({"commit": h.shas[e.at], "path": e.path, "state": state,
                       "kind": format!("{:?}", e.kind).to_lowercase(), "fate": fate_name(*f)})
            })
            .collect();
        println!(
            "{name}: {} commits, {} events, {unjudged} unjudged trees",
            h.trees.len(),
            list.len()
        );
        corpora.push(
            json!({"corpus": name, "commits": h.shas, "unjudged": unjudged,
                            "dependents": dependents, "events": list}),
        );
    }
    let doc = json!({"schema": "ce.eval-deadfile-gen2-replay/1.0.0",
                     "product": reg["product"], "corpora": corpora});
    let out = Path::new(&env("CE_DEADFILE_GEN2_OUT")).join("deadfile-gen2-replay-v1.json");
    std::fs::write(
        &out,
        serde_json::to_string_pretty(&doc).expect("doc") + "\n",
    )
    .expect("write");
}
