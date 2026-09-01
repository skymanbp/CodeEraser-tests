//! Which releases the series holds, and how each one's row is made:
//! a detached worktree per tag, the tag's OWN `ce` + `ce-core` built
//! there, and the measurement taken with THOSE. Split from bench.rs
//! when that file passed the 300-line edict — measuring a tree and
//! deciding which trees the series holds are two jobs.
//!
//!   cargo test --release --test it -- --ignored bench_backfill

use crate::bench::measure_tree;
use crate::bench_support as bs;
use std::path::PathBuf;
use std::process::Command;

/// Every tag that earns a row, oldest first, plus the ones the rule
/// turns away. v0.0.1-m0 is structurally absent (no product exists
/// there). The rule is applied BEFORE CE_BENCH_TAGS narrows the list,
/// so asking for a subset can never smuggle in a tag the whole-series
/// run would have skipped.
fn series_tags(only: Option<&str>) -> (Vec<String>, Vec<String>) {
    let all: Vec<String> = bs::git_out(&["tag", "--sort=creatordate"])
        .lines()
        .filter(|t| *t != "v0.0.1-m0")
        .map(str::to_string)
        .collect();
    let (mut joins, mut turned_away) = (Vec::new(), Vec::new());
    for (i, tag) in all.iter().enumerate() {
        // the oldest tag has no predecessor: all of it is new
        match i.checked_sub(1) {
            Some(p) if !bs::brings_something_new(&all[p], tag) => turned_away.push(tag.clone()),
            _ => joins.push(tag.clone()),
        }
    }
    let asked = |t: &String| only.is_none_or(|f| f.split(',').any(|x| x == t.as_str()));
    (joins.into_iter().filter(asked).collect(), turned_away)
}

/// Per-tag backfill (user ruling ④): detached worktree, the tag's
/// submodules seated (bench_support::seat_submodules — from v1.3.0 the
/// tests are one, and an unseated one is refused by name), the tag's
/// OWN ce + ce-core built and measured.
#[test]
#[ignore = "bench backfill: builds every release tag; slow (an hour-class run)"]
fn bench_backfill() {
    bs::release_only();
    // CE_BENCH_TAGS (comma list) re-runs a subset — an hour-class run
    // must not be the only way to replace one contaminated tag.
    let only = std::env::var("CE_BENCH_TAGS").ok();
    let (tags, turned_away) = series_tags(only.as_deref());
    if !turned_away.is_empty() {
        // named, never silent: a reader of this log must be able to see
        // which releases the series deliberately has no row for
        println!(
            "no row (same cli/src + core/app as predecessor): {}",
            turned_away.join(", ")
        );
    }
    for tag in &tags {
        println!("== {tag}");
        backfill_one(tag).expect(tag);
    }
    println!("bench_backfill: {} tags done", tags.len());
}

fn backfill_one(tag: &str) -> anyhow::Result<()> {
    let wt = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("bench-wt-{tag}"));
    let _ = Command::new("git")
        .args(["worktree", "remove", "--force"])
        .arg(&wt)
        .output();
    let ok = Command::new("git")
        .args(["worktree", "add", "--detach"])
        .arg(&wt)
        .arg(tag)
        .status()?;
    anyhow::ensure!(ok.success(), "worktree add {tag}");
    let seated = bs::seat_submodules(&wt)?;
    println!("   seated {} submodule(s)", seated.len());
    let build = |dir: &str, prog: &str, args: &[&str]| -> anyhow::Result<()> {
        let st = Command::new(prog)
            .args(args)
            .current_dir(wt.join(dir))
            .status()?;
        anyhow::ensure!(st.success(), "{prog} {args:?} at {tag}");
        Ok(())
    };
    build("cli", "cargo", &["build", "--release", "--locked"])?;
    build("core", "cabal", &["build", "all"])?;
    let core_out = Command::new("cabal")
        .args(["list-bin", "ce-core"])
        .current_dir(wt.join("core"))
        .output()?;
    let core = PathBuf::from(String::from_utf8_lossy(&core_out.stdout).trim());
    let ce = wt.join(if cfg!(windows) {
        "cli/target/release/ce.exe"
    } else {
        "cli/target/release/ce"
    });
    let commit = bs::git_out(&["rev-parse", &format!("{tag}^{{commit}}")]);
    let rows = measure_tree(&wt, &ce, &core, &commit, false, Some(tag))?;
    bs::merge_rows(rows)?;
    let _ = Command::new("git")
        .args(["worktree", "remove", "--force"])
        .arg(&wt)
        .output();
    Ok(())
}
