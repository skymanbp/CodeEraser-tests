//! The two runs on the real corpora, both `#[ignore]` and both refused
//! until the registry names the product commit and that commit's product
//! is the one under test (G6): the tips (section 1's second replacement
//! reason, read with the product build) and the replay itself. Neither
//! writes the registry; the replay writes one doc per corpus to
//! CE_DEADFILE_GEN2_OUT (G14), and CE_DEADFILE_GEN2_ONLY=<dir name>
//! restricts a run to one corpus.
//!   CE_DEADFILE_GEN2_ROOT=<clones> CE_DEADFILE_GEN2_COPIES=<scratch dir> cargo test --test it -- --ignored eval_deadfile_gen2_parts::run::deadfile_gen2_tips --nocapture
//!   CE_DEADFILE_GEN2_ROOT=<clones> CE_DEADFILE_GEN2_COPIES=<scratch dir> CE_DEADFILE_GEN2_OUT=<dir> cargo test --test it -- --ignored eval_deadfile_gen2_parts::run::deadfile_gen2_replay --nocapture

use super::doc::{corpus_doc, done, write};
use super::gitz::{checkout, clone_copy};
use super::judge::{deadcode, product_record};
use super::model::replayed;
use super::replay::history;
use super::{dir_name, product_commit, registry};
use crate::common::repo_root;
use crate::eval_lang_parts::generate::env;
use serde_json::Value;
use std::path::Path;

/// The registry and the product record, refused while the product commit
/// is unset or names another product than HEAD's.
pub fn armed() -> (Value, Value) {
    let reg = registry();
    let commit = product_commit(&reg).unwrap_or_else(|| {
        panic!("the registry names no product commit: generation 2 runs only on the build of the landed part 4A")
    });
    let record = product_record(&repo_root(), commit)
        .unwrap_or_else(|e| panic!("not the registered product: {e}"));
    (reg, record)
}

/// The chosen rows, CE_DEADFILE_GEN2_ONLY's alone when it is set.
fn rows(reg: &Value) -> Vec<Value> {
    let only = std::env::var("CE_DEADFILE_GEN2_ONLY").ok();
    let all = reg["corpora"].as_array().cloned().unwrap_or_default();
    let rows: Vec<Value> = all
        .into_iter()
        .filter(|r| only.as_deref().is_none_or(|o| dir_name(r) == o))
        .collect();
    assert!(!rows.is_empty(), "no registry row is {only:?}");
    rows
}

fn paths(row: &Value, suffix: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let name = dir_name(row);
    (
        Path::new(&env("CE_DEADFILE_GEN2_ROOT")).join(name),
        Path::new(&env("CE_DEADFILE_GEN2_COPIES")).join(format!("{name}{suffix}")),
    )
}

/// Each chosen corpus's tip judged by this build, by G5's predicate:
/// `judged` or the reason.
#[test]
#[ignore = "runs ce deadcode on the registered clones; armed by the registry's product commit"]
fn deadfile_gen2_tips() {
    let (reg, _) = armed();
    for row in rows(&reg) {
        let (clone, copy) = paths(&row, "-tip");
        let tip = row["tip"].as_str().expect("tip");
        clone_copy(&clone, &copy);
        let read = match checkout(&copy, tip) {
            Err(why) => why,
            Ok(()) => match deadcode(&copy) {
                Err(run) => panic!(
                    "{} {tip}: not a statement about the tree: {run}",
                    dir_name(&row)
                ),
                Ok(Ok(_)) => "judged".into(),
                Ok(Err(why)) => why,
            },
        };
        println!("{} {tip} {read}", row["language"]);
    }
}

/// The replay: per chosen corpus every event of the window with its fate,
/// the unjudged trees and the side numbers, into that corpus's doc; a
/// corpus whose doc holds this tip and product is skipped. The tables and
/// the bars are read off the docs (report.rs), never here.
#[test]
#[ignore = "replays the registered clones' histories; armed by the registry's product commit"]
fn deadfile_gen2_replay() {
    let (reg, record) = armed();
    let out = env("CE_DEADFILE_GEN2_OUT");
    let out = Path::new(&out);
    for row in rows(&reg) {
        let (name, tip) = (dir_name(&row), row["tip"].as_str().expect("tip"));
        if done(out, name, tip, &record) {
            println!("{name}: its doc is written for this tip and product, skipped");
            continue;
        }
        let (clone, copy) = paths(&row, "");
        let h = history(&clone, &copy, tip);
        let (events, sides) = replayed(&h);
        println!(
            "{name}: {} commits, {} events, {} unjudged window pairs",
            h.trees.len(),
            events.len(),
            sides.unjudged_pairs
        );
        write(out, &corpus_doc((name, tip, &record), &h, &events, &sides));
    }
}
