//! The differential legs of plan v2.33 W1 item 3 for the two optional
//! structure tables: the S5 staleness rows (`--days`) over one committed
//! repository whose history straddles the window, and the S6 redundancy
//! rows (`--deep`) over clone blocks drawn on the case and the dead files
//! of the liveness judgment — the core's path form (CE.Structure.Raw) fed
//! by the live `rows::stale_docs` / `rows::redundancy` against the frozen
//! `stale_doc_rows` / `redundancy_rows` (oracle/rows.rs, c2abca5d), each
//! with the tables every request carries, in the frozen measuring side's
//! order. Both frozen readers ask git or the core per case, so these
//! legs cost minutes: `CE_CORE_BIN=… cargo test --lib -- --ignored
//! structure::oracle::diff_stale`.

use crate::corelink::Link;
use crate::structure::oracle::diff_gen::{self, Case, Draw, SEEDS, plain};
use crate::structure::oracle::diff_rows::{answered, request, with};
use crate::structure::oracle::{diff_hold, rows, tree};
use crate::structure::rows as live;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The window both legs ask for, in days.
const DAYS: u32 = 30;

/// The tables every request carries, then `keys`.
fn keys(extra: [&'static str; 2]) -> Vec<&'static str> {
    let always = [
        "patternShapes",
        "conventions",
        "fileRefs",
        "dirEdges",
        "declared",
    ];
    always
        .into_iter()
        .chain(extra)
        .filter(|k| !k.is_empty())
        .collect()
}

fn git(root: &Path, args: &[&str], when: i64) {
    let out = Command::new("git")
        .current_dir(root)
        .args(args)
        .env("GIT_AUTHOR_DATE", format!("@{when} +0000"))
        .env("GIT_COMMITTER_DATE", format!("@{when} +0000"))
        .output()
        .expect("git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A repository of sixty files, a third of them Markdown, committed over
/// forty days in twenty commits — none within a day of the window's edge,
/// so the two readers' two clocks agree on every commit.
fn universe(seed: u64) -> (PathBuf, Vec<String>) {
    let root = crate::testutil::scratch(&format!("i3-stale-{seed:x}"));
    let mut d = Draw::case(seed, usize::MAX);
    let mut paths: Vec<String> = (0..60).map(|_| diff_gen::path(&mut d)).collect();
    for p in paths.iter_mut().step_by(3) {
        *p = format!("{}.md", p.trim_end_matches(".md"));
    }
    paths.sort();
    paths.dedup();
    // a name that is also a directory of another path cannot be a file
    let all = paths.clone();
    paths.retain(|p| !all.iter().any(|q| q.starts_with(&format!("{p}/"))));
    git(&root, &["init", "-q"], 0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("a clock");
    for k in 0..20i64 {
        let age = if k < 8 { 40 - k } else { 28 - (k - 8) * 2 };
        for p in paths.iter().filter(|_| d.chance(30)) {
            let f = root.join(p);
            std::fs::create_dir_all(f.parent().expect("a parent")).expect("a dir");
            std::fs::write(&f, format!("{p} {k}\n")).expect("a file");
        }
        git(&root, &["add", "-A"], 0);
        let when = now.as_secs() as i64 - age * 86_400;
        git(
            &root,
            &[
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "c",
            ],
            when,
        );
    }
    (root, paths)
}

#[test]
#[ignore = "instrument: the item-3 differential, run with a core"]
fn the_cores_staleness_rows_answer_what_the_frozen_rust_built() {
    for seed in SEEDS {
        let (root, paths) = universe(seed);
        let one = |link: &mut Link, d: &mut Draw| {
            let c = diff_gen::case(d, Some(&paths));
            let frozen = rows::stale_doc_rows(&root, &c.w, &tree::build(&c.walked), DAYS);
            let want = with(link, &c, frozen.map(|(docs, edges)| json!({"staleDocRows": plain(docs), "staleEdgeRows": plain(edges)})));
            let docs = live::stale_docs(&root, &c.w, DAYS).expect("the live staleness facts");
            let got = answered(
                link,
                &request(&c, Some(docs), None),
                &keys(["staleDocRows", "staleEdgeRows"]),
            );
            (want, got)
        };
        let held = diff_hold::hold(seed, one);
        assert!(held.bad.is_empty(), "{}", held.bad.join("\n"));
        let _ = std::fs::remove_dir_all(&root);
    }
}

/// Clone blocks between drawn files: walked ones mostly, a stranger now
/// and then (the frozen reader's fault road).
fn blocks(d: &mut Draw, c: &Case) -> crate::dedup::pairs::Blocks {
    let file = |d: &mut Draw| {
        if d.chance(4) {
            diff_gen::path(d)
        } else {
            c.walked[d.under(c.walked.len())].clone()
        }
    };
    let blocks: Vec<Value> = (0..d.under(5))
        .map(|_| {
            let (a, b) = (file(d), file(d));
            json!({"a_file": a, "a_start": 1, "a_end": 9, "b_file": b, "b_start": 1, "b_end": 9, "tokens": 60, "distinct": 9})
        })
        .collect();
    let all = json!({"blocks": blocks, "groups": [], "hot_chained": 0, "stale_skipped": 0,
        "low_diversity_suppressed": 0, "distincts": []});
    serde_json::from_value(all).expect("blocks")
}

#[test]
#[ignore = "instrument: the item-3 differential, run with a core"]
fn the_cores_redundancy_rows_answer_what_the_frozen_rust_built() {
    let core = crate::daemon::judge::core_bin().expect("a core");
    let root = std::env::temp_dir().join(format!("ce-i3-dead-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("a temp root");
    for seed in SEEDS {
        let one = |link: &mut Link, d: &mut Draw| {
            let c = diff_gen::case(d, None);
            let found = blocks(d, &c);
            let frozen = rows::redundancy_rows(&root, &core, &c.w, &found, &tree::build(&c.walked));
            let want = with(link, &c, frozen.map(|r| json!({"redundancy": plain(r)})));
            let got = match live::redundancy(&root, &core, &c.w, &found) {
                Err(e) => json!({"fault": e.to_string()}),
                Ok(r) => answered(link, &request(&c, None, Some(r)), &keys(["redundancy", ""])),
            };
            (want, got)
        };
        let held = diff_hold::hold(seed, one);
        assert!(held.bad.is_empty(), "{}", held.bad.join("\n"));
        assert!(held.faults > 0, "seed {seed:#x}: no fault case");
    }
    let _ = std::fs::remove_dir_all(&root);
}
