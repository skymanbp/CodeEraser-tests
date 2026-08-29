//! Fixture seeding — the T2 seed source, the tmp dir anchor, and
//! the git-repo shapes the gate batteries share. Split from mod.rs
//! in the headroom sprint: audit.rs importing these THROUGH the hub
//! made this support tree a module cycle the graph axis itself
//! billed on the self-scan.

use super::gitio::git;
use std::path::{Path, PathBuf};

/// Fresh per-test dir under the cargo target tmpdir (wiped if
/// present). Carries an empty `.git` anchor: hookio::project_root
/// ascends to the nearest ce.toml/.git, and an anchorless fixture
/// under target/tmp would ascend into the REAL repo (three guard
/// batteries did exactly that when the anchoring landed). Real hook
/// cwds are never anchorless voids; the walker skips hidden dirs,
/// so scans see nothing.
pub fn tmp(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join(".git")).expect("mkdir");
    dir
}

/// Write a fixture DOCUMENT out as its tree: a line `--- <path>`
/// opens one file, and everything until the next such line is its
/// body. One document per fixture rather than a table of (path,
/// source) pairs: the pair table is this repo's most-rhyming token
/// shape — its own clone gate matched a six-entry one against two
/// unrelated fixture tables — and a document also reads as the tree
/// it makes. Shared by the symbol, export-surface and mounts legs
/// (the third copy of the splitter was the gate's own verdict).
pub fn write_doc(dir: &Path, doc: &str) {
    let mut files: Vec<(&str, String)> = Vec::new();
    for line in doc.lines() {
        match line.strip_prefix("--- ") {
            Some(path) => files.push((path.trim(), String::new())),
            None => {
                if let Some((_, body)) = files.last_mut() {
                    body.push_str(line);
                    body.push('\n');
                }
            }
        }
    }
    for (rel, body) in files {
        if let Some(parent) = Path::new(rel)
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
        {
            std::fs::create_dir_all(dir.join(parent)).expect("mkdir fixture subdir");
        }
        std::fs::write(dir.join(rel), body).expect("write fixture file");
    }
}

/// A fixture document written out under a fresh tmp dir — the two
/// lines every document-shaped leg opened with (the deadcode legs let
/// their own run build the index; `indexed_doc` adds the walk).
pub fn doc_tree(name: &str, doc: &str) -> PathBuf {
    let dir = tmp(name);
    write_doc(&dir, doc);
    dir
}

/// A fixture document written out under a fresh tmp dir and indexed
/// by the real walk — the three lines every wire-reading leg opened
/// with.
pub fn indexed_doc(name: &str, doc: &str) -> PathBuf {
    let dir = doc_tree(name, doc);
    super::build_index(&dir);
    dir
}

/// ~60 normalized tokens; `seed` renames identifiers and changes
/// literals so pairs of outputs are T2 (not T1) clones clearing t=50.
pub fn rust_fn(seed: u32) -> String {
    format!(
        "fn work_{seed}(input_{seed}: &[i64], limit_{seed}: i64) -> i64 {{
    let mut total_{seed} = {seed};
    for value_{seed} in input_{seed} {{
        if *value_{seed} > limit_{seed} {{
            total_{seed} += value_{seed} * {seed} + 7;
        }} else {{
            total_{seed} -= value_{seed} / 3;
        }}
    }}
    total_{seed}
}}
"
    )
}

/// Write `a.rs` (the T2 seed) plus a ce.toml pinning the guard mode.
pub fn seed_sources(dir: &Path, mode: &str) {
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    std::fs::write(dir.join("ce.toml"), format!("[guard]\nmode = \"{mode}\"\n")).expect("ce.toml");
}

/// a.rs + b.rs forming a T2 clone pair that clears t=50.
/// The one-line ce.toml that declares a dedup budget — the knob every
/// gate fixture turns, and the cheapest non-default declaration (a
/// Some digest) the baseline fixtures need.
pub fn seed_budget(dir: &Path, budget: u32) {
    std::fs::write(dir.join("ce.toml"), format!("[dedup]\nbudget = {budget}\n")).expect("ce.toml");
}

pub fn seed_clone_pair(dir: &Path) {
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs (T2 clone)");
}

/// `git init` + the first commit of everything present — the one
/// repo-birth stanza (trend_rebuild and trend_submodule each grew it
/// beside their seeds until the ratchet paired them).
pub fn init_and_commit(dir: &Path, msg: &str) {
    git(dir, &["init", "-q"]);
    commit_all(dir, msg);
}

/// Stage everything present and commit it — the one commit stanza
/// (the P4 ratchet caught trend_rebuild's seed re-growing this trio
/// of git calls token for token).
pub fn commit_all(dir: &Path, msg: &str) {
    git(dir, &["add", "."]);
    git(dir, &["commit", "-qm", msg]);
}

/// A superproject with one commit of its own, then one that mounts a
/// submodule holding `seed_clone_pair`'s T2 pair at `mount` — `suite`
/// unless a leg needs the judgment to EXCLUDE it (`vendor/` hid the
/// pair from the live score, measured). The local-path transport git
/// refuses by default since 2.38.1 is allowed for this fixture alone.
pub fn seed_superproject(name: &str, mount: &str) -> PathBuf {
    let sub = tmp(&format!("{name}-sub"));
    seed_clone_pair(&sub);
    init_and_commit(&sub, "pair");
    let sup = tmp(name);
    std::fs::write(sup.join("root.rs"), rust_fn(3)).expect("root.rs");
    init_and_commit(&sup, "root");
    let url = sub.to_str().expect("utf8").replace('\\', "/");
    git(
        &sup,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "-q",
            &url,
            mount,
        ],
    );
    commit_all(&sup, "mount");
    sup
}

/// The measurement walk as `(rel, foreign)` rows — what every
/// owner-rule leg (guard_hook, foreign_readers) asserts on, so the
/// collect-map-collect stanza has one home (the clone gate paired the
/// first two copies).
pub fn walked(dir: &Path) -> Vec<(String, bool)> {
    codeeraser::scan::walk::collect(dir, &[])
        .expect("collect")
        .iter()
        .map(|w| (codeeraser::scan::walk::rel_str(dir, &w.path), w.foreign))
        .collect()
}

/// Append to a tracked file: the smallest edit a git leg can see.
pub fn append(path: &Path, tail: &str) {
    let text = std::fs::read_to_string(path).expect("read") + tail;
    std::fs::write(path, text).expect("append");
}

/// `git submodule deinit`: the checkout leaves, the gitlink stays — the
/// unseated shape every refusal leg asks about.
pub fn unseat(sup: &Path, mount: &str) {
    git(sup, &["submodule", "deinit", "-f", "-q", mount]);
}

/// The three-commit history the churn batteries measure, and the one
/// every long-measurement face reuses: a root commit adding a.rs; an
/// edit INSIDE work_1 plus a new b.rs (one rewrite, one append, one
/// co-change); a third touching both — a.rs gaining a top-level tail
/// and b.rs losing its function body, so some window additions stop
/// surviving. Shared because the progress face needs the same three
/// phases to have work to do, and its own copy of this was a
/// 121-token twin of churn.rs's that the dedup ratchet refused.
pub fn seed_churn_history(dir: &Path) {
    git(dir, &["init", "-q"]);
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("a.rs");
    commit_all(dir, "seed");
    let edited = rust_fn(1).replace("+ 7;", "+ 8;\n            total_1 += 1;");
    std::fs::write(dir.join("a.rs"), &edited).expect("a.rs edit");
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs");
    commit_all(dir, "edit + new");
    std::fs::write(dir.join("a.rs"), format!("{edited}\n// tail\n")).expect("a.rs tail");
    std::fs::write(dir.join("b.rs"), "// emptied\n").expect("b.rs emptied");
    commit_all(dir, "entangle + churn");
}

/// Git repo with the T2 seed committed and b.rs (the clone) staged
/// but uncommitted — the audit/precommit fixture shape.
pub fn seed_git_clone_repo(dir: &Path, mode: &str) {
    seed_sources(dir, mode);
    git(dir, &["init", "-q"]);
    commit_all(dir, "seed");
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("b.rs (uncommitted clone)");
    git(dir, &["add", "b.rs"]); // numstat vs HEAD sees staged new files
}
