//! Relocation credit for the duplicate-write class (v2.35 guard lane,
//! GD4). The write-time probe refused a MOVE whose source was removed
//! first whenever the moved function had other surviving twins (P2 E7:
//! 116 of 360), though the wall it mirrors — `ce dedup --check`, which
//! counts blocks — saw no new block. The stage counts the matched run
//! in HEAD and in the tree the write leaves (relocate/1 judges
//! `occNow <= occHead`); a write all of whose matches are relocations
//! passes. Every case here is one of the brief's legs: the safe order
//! passes, the wrong order, a second copy and a staged extra copy are
//! refused, restoring a file to its HEAD content passes, and without
//! git there is no credit — the refusal as it stood, the reason named.

use crate::common::{self, init_and_commit, last_observe, run_hook, tmp, write_all};
use std::path::Path;

/// A function well past the class floors (113 tokens, as P2 measured
/// the probe's own count on this shape), and an unrelated one.
const F: &str = "def relocate_me(rows, key, limit):\n    seen = {}\n    for row in rows:\n        k = row.get(key)\n        if k is None:\n            continue\n        bucket = seen.setdefault(k, [])\n        if len(bucket) >= limit:\n            raise ValueError(\"too many rows for %s\" % k)\n        bucket.append(row)\n    total = sum(len(v) for v in seen.values())\n    return sorted(seen.items(), key=lambda kv: (len(kv[1]), kv[0])), total\n";
const G: &str = "def unrelated(x):\n    return x + 1\n";

/// A committed, indexed tree where `f` lives in a.py and its twin in
/// b.py.
fn twins(tag: &str) -> std::path::PathBuf {
    let dir = tmp(tag);
    let a = format!("{G}\n\n{F}");
    write_all(&dir, &[("a.py", a.as_str()), ("b.py", F)]);
    init_and_commit(&dir, "seed");
    common::build_index(&dir);
    dir
}

/// The decision on writing `text` at `rel`, and the probe line.
fn write(dir: &Path, rel: &str, text: &str) -> (String, serde_json::Value) {
    let out = run_hook(
        dir,
        &["probe", "--hook"],
        &common::pretooluse_envelope_at(dir, rel, "Write", text),
    );
    let probe = common::observe_lines(dir)
        .into_iter()
        .rev()
        .find(|l| l["event"] == "probe")
        .expect("a probe line");
    (out, probe)
}

#[test]
fn a_move_with_its_source_removed_first_passes() {
    let dir = twins("guard-relocate-e7");
    std::fs::write(dir.join("a.py"), G).expect("source removed first");
    let (out, probe) = write(&dir, "c.py", F);
    assert!(out.trim().is_empty(), "a relocation is not refused: {out}");
    assert_eq!(
        (&probe["matches"], &probe["relocated"]),
        (&1.into(), &1.into()),
        "{probe}"
    );
    assert!(probe.get("relocation").is_none(), "{probe}");
}

#[test]
fn a_copy_before_the_source_goes_is_refused() {
    let dir = twins("guard-relocate-order");
    let reason = common::expect_write_denied(&dir, "c.py", F, "duplicates 2 indexed region(s)");
    assert!(reason.contains("b.py"), "{reason}");
    let probe = last_observe(&dir);
    assert_eq!(probe["relocated"], 0, "{probe}");
}

#[test]
fn two_copies_for_one_removed_source_are_refused() {
    let dir = twins("guard-relocate-two");
    std::fs::write(dir.join("a.py"), G).expect("one source removed");
    let both = format!("{F}\n\n{}", F.replace("relocate_me", "relocate_me_too"));
    common::expect_write_denied(&dir, "c.py", &both, "indexed region(s)");
}

#[test]
fn a_staged_copy_counts_on_the_tree_side() {
    let dir = twins("guard-relocate-staged");
    std::fs::write(dir.join("a.py"), G).expect("source removed first");
    write_all(&dir, &[("c.py", F)]);
    common::git_ok(&dir, &["add", "c.py"]);
    common::expect_write_denied(&dir, "d.py", F, "indexed region(s)");
}

#[test]
fn restoring_a_file_to_its_head_content_passes() {
    let dir = twins("guard-relocate-restore");
    std::fs::remove_file(dir.join("b.py")).expect("b.py deleted");
    let (out, probe) = write(&dir, "b.py", F);
    assert!(out.trim().is_empty(), "the file back as HEAD has it: {out}");
    assert_eq!(probe["relocated"], probe["matches"], "{probe}");
}

#[test]
fn without_git_there_is_no_credit() {
    let dir = tmp("guard-relocate-nogit");
    write_all(&dir, &[("a.py", G), ("b.py", F)]);
    common::build_index(&dir);
    // the scratch dir sits inside this repository's own checkout: stop
    // git's discovery at the dir, so it is no repository at all
    let ceiling = dir.parent().expect("a parent").display().to_string();
    let out = common::run_hook_env(
        &dir,
        &["probe", "--hook"],
        &common::pretooluse_envelope_at(&dir, "c.py", "Write", F),
        &[("GIT_CEILING_DIRECTORIES", ceiling.as_str())],
    );
    assert!(out.contains("\"deny\"") && out.contains("b.py"), "{out}");
    let probe = last_observe(&dir);
    assert!(
        probe["relocation"]
            .as_str()
            .is_some_and(|w| w.starts_with("git")),
        "{probe}"
    );
}
