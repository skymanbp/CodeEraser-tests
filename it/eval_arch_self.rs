//! The architecture family's reading of this repository, frozen (plan
//! v2.31 step 9, design booklet §7.4): not the working tree — a tree
//! that moves with every commit would make every drift a false alarm —
//! but one pinned commit, exported with `git archive` and its
//! `.gitmodules` dropped (a declared submodule the export cannot seat
//! would make the walk refuse by name; the tests are another universe,
//! and the main root's measured set is the one a seated checkout
//! measures). The reading moves only when the MEASUREMENT or the CORE
//! does, so a drift here is a real one: the counts, every directory's
//! level and metrics, every cut with its weight and exactness, every
//! misplaced file, and the clusters as their count and sizes (their
//! members are long, and the misplaced rows already say where they
//! disagree with the tree).
//!
//! Two roads: the CI leg re-measures and compares field by field; the
//! ignored leg rewrites the doc under CE_BLESS=1 — into the directory
//! CE_ARCH_OUT names when it is set (common::ledger::out_file), so the
//! run leaves the tree clean and generated_from reads dirty = false:
//!   CE_BLESS=1 cargo test --test it -- --ignored eval_arch_self::regenerate --nocapture

use crate::common;
use serde_json::{Map, Value, json};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const DOC: &str = "contracts/eval/arch-self-v1.json";
const SCHEMA: &str = "ce.eval-arch-self/1.0.0";
const REPO: &str = "skymanbp/CodeEraser";
/// v2.31 step 3's follow-up: the tree the analysis track's first four
/// families were measured on.
const COMMIT: &str = "f2a7a2b4c2f52b8b919cec12d1bacd036b2d2058";
const FORM: &str = "git archive, .gitmodules removed";

/// The pinned commit exported under a fresh tmp dir, LF line endings
/// on every platform (the archive otherwise follows the checkout's
/// autocrlf), the submodule declaration removed.
fn exported() -> PathBuf {
    let dir = common::tmp("eval-arch-self");
    let mut git = Command::new("git")
        .arg("-C")
        .arg(common::repo_root())
        .args([
            "-c",
            "core.autocrlf=false",
            "archive",
            "--format=tar",
            COMMIT,
        ])
        .stdout(Stdio::piped())
        .spawn()
        .expect("git archive");
    let tar = Command::new("tar")
        .args(["-x", "-f", "-", "-C"])
        .arg(&dir)
        .stdin(Stdio::from(git.stdout.take().expect("the archive stream")))
        .status()
        .expect("tar");
    let archived = git.wait().expect("git archive");
    assert!(
        archived.success() && tar.success(),
        "export {COMMIT}: git {archived}, tar {tar} (is the commit fetched?)"
    );
    std::fs::remove_file(dir.join(".gitmodules")).expect("the export's .gitmodules");
    dir
}

/// The fields the doc freezes, off one run of the family's face.
fn reading(root: &Path) -> Value {
    let doc = codeeraser::arch::face::run(root, None, &common::core_bin(), &[]).expect("arch");
    let r = <codeeraser::arch::report::Report as serde::Deserialize>::deserialize(&doc)
        .expect("an arch document");
    assert_eq!(r.degraded, None, "the self reading needs a judging core");
    let layers: Map<String, Value> = r
        .layers
        .iter()
        .map(|l| (l.dir.clone(), json!(l.level)))
        .collect();
    let metrics: Map<String, Value> = r
        .metrics
        .iter()
        .map(|m| (m.dir.clone(), json!([m.fan_in, m.fan_out, m.instability])))
        .collect();
    let mut sizes: Vec<usize> = r.clusters.iter().map(|c| c.files.len()).collect();
    sizes.sort_unstable_by(|a, b| b.cmp(a));
    json!({
        "counts": r.counts,
        "layers": layers,
        "cuts": r.cuts.iter().map(|c| json!([c.from, c.to, c.refs, c.exact])).collect::<Vec<_>>(),
        "metrics": metrics,
        "misplaced": r.misplaced.iter().map(|m| json!([m.path, m.dir, m.majority])).collect::<Vec<_>>(),
        "clusters": {"count": r.clusters.len(), "sizes": sizes},
    })
}

/// What one field's two readings disagree on: the keys (an object) or
/// the rows (an array) on one side only or valued differently, at most
/// a screenful.
fn drift(field: &str, frozen: &Value, now: &Value) -> Vec<String> {
    let mut out = Vec::new();
    match (frozen, now) {
        (Value::Object(a), Value::Object(b)) => {
            for k in a.keys().chain(b.keys().filter(|k| !a.contains_key(*k))) {
                if a.get(k) != b.get(k) {
                    out.push(format!(
                        "{field}.{k:?}: frozen {:?} now {:?}",
                        a.get(k),
                        b.get(k)
                    ));
                }
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            out.extend(
                a.iter()
                    .filter(|x| !b.contains(x))
                    .map(|x| format!("{field}: frozen only {x}")),
            );
            out.extend(
                b.iter()
                    .filter(|x| !a.contains(x))
                    .map(|x| format!("{field}: now only {x}")),
            );
        }
        _ => out.push(format!("{field}: frozen {frozen} now {now}")),
    }
    out.truncate(24);
    out
}

/// CI: the pinned commit re-measures to the frozen reading.
#[test]
fn the_pinned_commit_reads_as_frozen() {
    let doc: Value = std::fs::read_to_string(common::repo_root().join(DOC))
        .map_err(|e| e.to_string())
        .and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string()))
        .unwrap_or_else(|e| panic!("the frozen arch reading {DOC}: {e}"));
    assert_eq!(
        (&doc["schema"], &doc["corpus"]),
        (
            &json!(SCHEMA),
            &json!({"repo": REPO, "commit": COMMIT, "form": FORM})
        )
    );
    let now = reading(&exported());
    let frozen = &doc["reading"];
    let diffs: Vec<String> = [
        "counts",
        "layers",
        "cuts",
        "metrics",
        "misplaced",
        "clusters",
    ]
    .iter()
    .flat_map(|f| drift(f, &frozen[*f], &now[*f]))
    .collect();
    assert!(
        diffs.is_empty(),
        "the arch reading of {COMMIT} drifted — the measurement or the core moved:\n{}\n\
         re-freeze after reading the diff:\n  CE_BLESS=1 cargo test --test it -- --ignored \
         eval_arch_self::regenerate --nocapture",
        diffs.join("\n")
    );
}

/// By hand: the doc rewritten from a fresh export.
#[test]
#[ignore = "writes the frozen doc; needs CE_BLESS=1"]
fn regenerate() {
    assert!(
        crate::facts::blessing(),
        "regenerate writes {DOC}: set CE_BLESS=1"
    );
    let root = common::repo_root();
    let doc = json!({
        "schema": SCHEMA,
        "corpus": {"repo": REPO, "commit": COMMIT, "form": FORM},
        "generated_from": crate::eval_support::provenance::generated_from(),
        "reading": reading(&exported()),
    });
    let text = format!("{}\n", serde_json::to_string_pretty(&doc).expect("doc"));
    let path = root.join(common::ledger::out_file("CE_ARCH_OUT", DOC));
    std::fs::write(&path, &text).expect("write the frozen arch reading");
    println!(
        "{}",
        serde_json::to_string_pretty(&doc["reading"]["counts"]).expect("counts")
    );
}
