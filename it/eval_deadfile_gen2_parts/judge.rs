//! Two predicates the tips leg and the replay share. G5: "the product
//! judged this tree" — the exit code is 0 or 1, stdout parses and
//! `degraded` is null; anything else is unjudged with its reason (the exit
//! code, the `degraded` word, the first line of stderr), and a reason that
//! is no statement about the TREE (the core missing, a version skew, a
//! deadline, an IO error) is refused by name instead of counted; the run's
//! `ce` waits `CE_CORE_DEADLINE_SECS=3600`, so a starved core is never read
//! as a refusal; a refusal counts only when a second run on a cold index
//! repeats it (R2). G6: "armed" — the product under test is the
//! registered one, path by path, with a record of what ran; an armed
//! run joins the instrument's commit to it (R3, F2).

use crate::common::gitio::{git_ok, git_out};
use serde_json::{Value, json};
use std::path::Path;

/// Words that make a reason a statement about the run, not the tree.
pub const NOT_THE_TREE: &str = "core unavailable|proto mismatch|out of step|core killed|\
cannot start|unexpected reply type|bad hello reply|desync|bad reply|os error";

/// The core's reply wait for the replay's runs, in seconds.
pub const DEADLINE_SECS: &str = "3600";

/// The product paths G6 holds equal to the registered commit's (the two
/// cabal project files choose the core's dependencies and flags, R3a).
pub const PRODUCT_PATHS: [&str; 7] = [
    "cli/src",
    "core/app",
    "cli/Cargo.toml",
    "cli/Cargo.lock",
    "core/ce-core.cabal",
    "core/cabal.project",
    "core/cabal.project.freeze",
];

/// `Ok(Ok(doc))` judged, `Ok(Err(reason))` unjudged, `Err(reason)` a
/// reason about the run.
pub type Judgment = Result<Result<Value, String>, String>;

/// G5 on one run's exit code and output.
pub fn judgment(code: Option<i32>, stdout: &[u8], stderr: &[u8]) -> Judgment {
    let err = String::from_utf8_lossy(stderr);
    let first = err.lines().map(str::trim).find(|l| !l.is_empty());
    let said = first.map(|l| format!(": {l}")).unwrap_or_default();
    let exit = code.map_or_else(|| "no exit code".to_string(), |c| format!("exit {c}"));
    let reason = match (code, serde_json::from_slice::<Value>(stdout)) {
        (Some(0 | 1), Ok(doc)) if doc["degraded"].is_null() => return Ok(Ok(doc)),
        (Some(0 | 1), Ok(doc)) => format!("{exit}: degraded: {}", doc["degraded"]),
        (Some(0 | 1), Err(_)) => format!("{exit}: stdout does not parse{said}"),
        _ => format!("{exit}{said}"),
    };
    match NOT_THE_TREE.split('|').find(|w| reason.contains(w)) {
        Some(w) => Err(format!("{reason} (\"{w}\")")),
        None => Ok(Err(reason)),
    }
}

/// `ce deadcode --format json .` in `dir`, read by G5.
pub fn deadcode(dir: &Path) -> Judgment {
    let out = crate::common::ce_command(dir, &[("CE_CORE_DEADLINE_SECS", DEADLINE_SECS)])
        .args(["deadcode", "--format", "json", "."])
        .output()
        .expect("run ce deadcode");
    judgment(out.status.code(), &out.stdout, &out.stderr)
}

/// A tree's settled reading (R2): the document or the reason it stays
/// unjudged, and the first run's reason when only the second judged it.
#[derive(Debug)]
pub struct Settled {
    pub read: Result<Value, String>,
    pub retried: Option<String>,
}

/// R2: a refusal counts only when the product repeats it. The first run
/// judged: judged, `again` is not called. A reason about the run, either
/// time: `Err` (the run stops by name). Unjudged, then judged: judged by
/// the second, the first reason kept. Unjudged twice for the same reason:
/// unjudged; for two reasons: `Err` with both.
pub fn settled(first: Judgment, again: impl FnOnce() -> Judgment) -> Result<Settled, String> {
    let first = match first? {
        Ok(doc) => {
            return Ok(Settled {
                read: Ok(doc),
                retried: None,
            });
        }
        Err(why) => why,
    };
    match again()? {
        Ok(doc) => Ok(Settled {
            read: Ok(doc),
            retried: Some(first),
        }),
        Err(why) if why == first => Ok(Settled {
            read: Err(why),
            retried: None,
        }),
        Err(why) => Err(format!(
            "two runs refuse one tree for two reasons: {first} | {why}"
        )),
    }
}

/// `ce deadcode` in `dir`, settled by R2: a refusal is asked again on a
/// cold index (`.ce` removed).
pub fn settle(dir: &Path) -> Result<Settled, String> {
    settled(deadcode(dir), || {
        let index = dir.join(".ce");
        if index.exists() {
            std::fs::remove_dir_all(&index).expect("remove the index");
        }
        deadcode(dir)
    })
}

/// R3b, F2: an armed run's record joined with the instrument's commit,
/// the HEAD of the tests subrepo at `tests` (the instrument and the
/// helpers it calls), refused by name while it holds an uncommitted
/// change.
pub fn with_instrument(mut record: Value, tests: &Path) -> Result<Value, String> {
    if let Some(line) = git_ok(tests, &["status", "--porcelain"]).lines().next() {
        return Err(format!("the instrument has an uncommitted change: {line}"));
    }
    record["instrument"] = json!(oid(tests, "HEAD"));
    Ok(record)
}

/// sha256 hex of a file's bytes.
pub fn file_sha(path: &str) -> String {
    use sha2::Digest;
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let h = sha2::Sha256::digest(&bytes);
    h.iter().map(|b| format!("{b:02x}")).collect()
}

fn oid(root: &Path, rev: &str) -> String {
    git_ok(root, &["rev-parse", rev]).trim().to_string()
}

/// G6: every product path's object id at the registered commit equals
/// HEAD's, and none has an uncommitted change; the record then names the
/// commit, the two trees, every path's object, the build profile and the
/// sha256 of the `ce` and `ce-core` that run. It says nothing about the
/// instrument: the product's identity does not depend on the tests tree,
/// the armed run's does (F2, `with_instrument`).
pub fn product_record(root: &Path, commit: &str) -> Result<Value, String> {
    if !git_out(root, &["cat-file", "-e", &format!("{commit}^{{commit}}")]).0 {
        return Err(format!("{commit}: no such commit here"));
    }
    let mut objects = serde_json::Map::new();
    for p in PRODUCT_PATHS {
        let (at, head) = (
            oid(root, &format!("{commit}:{p}")),
            oid(root, &format!("HEAD:{p}")),
        );
        if at != head {
            return Err(format!(
                "{p}: {at} at the registered commit, {head} at HEAD"
            ));
        }
        objects.insert(p.to_string(), json!(at));
    }
    let mut status = vec!["status", "--porcelain", "--"];
    status.extend(PRODUCT_PATHS);
    if let Some(line) = git_ok(root, &status).lines().next() {
        return Err(format!("uncommitted change under the product: {line}"));
    }
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    Ok(json!({
        "commit": commit,
        "commit_tree": oid(root, &format!("{commit}^{{tree}}")),
        "head_tree": oid(root, "HEAD^{tree}"),
        "objects": objects,
        "profile": profile,
        "ce_sha256": file_sha(env!("CARGO_BIN_EXE_ce")),
        "core_sha256": file_sha(&crate::common::core_bin()),
    }))
}
