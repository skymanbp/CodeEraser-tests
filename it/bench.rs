//! The bench harness (M9 batch 5): every dashboard number is
//! produced by REPLAY — this file — and lands in
//! contracts/bench/bench.json, the single source the site, README
//! and GUI render from (hand-filling a surface re-opens the D7
//! defect class). One ignored driver and one live gate here; the
//! per-tag backfill driver is bench_backfill.rs, which borrows this
//! module's `measure_tree` and `git_out`:
//!   bench_append   — measure THIS checkout's built binaries
//!   bench_doc_is_wellformed — CI: the committed doc stays sound
//! Run in RELEASE; a debug measurement is refused, not annotated
//! (PERF-BUDGET.md:60-62):
//!   cargo test --release --test bench -- --ignored --nocapture

use crate::bench_support as bs;
use crate::common;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A fresh db path per cold run — SCHEMA/GRAPH rev wipes across tags
/// make any cache reuse silently wrong (constraint 7 of the map).
fn fresh_db(tag: &str, i: usize) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("bench-{tag}-{i}/index.db"));
    let _ = std::fs::remove_dir_all(p.parent().expect("parent"));
    p
}

/// Point a command at the tag's own core TWO ways: the env var modern
/// versions read, and the PATH early ones resolve the bare name
/// `ce-core` through.
///
/// The second is not belt-and-braces. v0.1.0's `ce` takes the core as a
/// `--core` flag defaulting to the bare name and reads CE_CORE_BIN only
/// on its daemon path, so with PATH untouched it reaches whatever
/// ce-core is INSTALLED on the machine — which on this machine answers
/// a protocol six majors newer and kills the run by name. A tag whose
/// installed neighbour happened to be compatible would be worse: the
/// row would be measured against a foreign core and say nothing about
/// the tag. BENCH.md promises the tag's own binaries; this is what
/// makes that true for every tag rather than the recent ones.
fn with_core(c: &mut Command, core: &Path) {
    c.env("CE_CORE_BIN", core);
    if let Some(dir) = core.parent() {
        let sep = if cfg!(windows) { ";" } else { ":" };
        let rest = std::env::var("PATH").unwrap_or_default();
        c.env("PATH", format!("{}{sep}{rest}", dir.display()));
    }
}

fn ce_cmd(ce: &Path, core: &Path, root: &Path, args: &[&str]) -> Command {
    let mut c = Command::new(ce);
    c.args(args).arg(root);
    with_core(&mut c, core);
    c
}

/// One tree's latency series: cold rows re-index from nothing, warm
/// rows re-run over a primed cache. Counts are deliberately absent
/// here — console wording drifts across tags, and a parsed count
/// that silently mis-greps would be a fabricated number.
pub fn measure_tree(
    root: &Path,
    ce: &Path,
    core: &Path,
    commit: &str,
    dirty: bool,
    tag: Option<&str>,
) -> anyhow::Result<Vec<Value>> {
    // The series key is the RELEASE, not the binary's self-report:
    // the v0.1.0 tag's crate still said 0.0.1 (version discipline
    // started at v0.2.0), and keying by self-report filed that tag's
    // measurement under a version that was never released. The commit
    // field stays the provenance either way.
    let version = match tag {
        Some(t) => t.trim_start_matches('v').to_string(),
        None => bs::version_of(ce)?,
    };
    let date = bs::today();
    let mut out = Vec::new();
    let mut push = |metric: &str, sorted: &[u128]| {
        out.push(bs::row(
            &version, commit, dirty, metric, "ms", sorted, None, &date,
        ));
    };
    let (scan, _) = bs::samples(5, || ce_cmd(ce, core, root, &["scan"]), "scan")?;
    push("scan", &scan);
    let mut cold = Vec::new();
    for i in 0..3 {
        let db = fresh_db("dedup", i);
        let mut c = ce_cmd(ce, core, root, &["dedup"]);
        c.arg("--db").arg(&db);
        cold.push(bs::timed(c, "dedup cold")?.0);
    }
    cold.sort_unstable();
    push("dedup_cold", &cold);
    let db = fresh_db("warm", 0);
    bs::timed(warm_cmd(ce, core, root, "dedup", &db), "dedup prime")?;
    let warm = [
        ("dedup", 5, "dedup_warm"),
        ("deadcode", 3, "deadcode_warm"),
        ("docdup", 3, "docdup_warm"),
        ("check", 3, "check_warm"),
    ];
    for (sub, n, metric) in warm {
        let (ms, _) = bs::samples(n, || warm_cmd(ce, core, root, sub, &db), sub)?;
        push(metric, &ms);
    }
    out.extend(hook_rows(root, ce, core, &version, commit, dirty, &date)?);
    Ok(out)
}

fn warm_cmd(ce: &Path, core: &Path, root: &Path, sub: &str, db: &Path) -> Command {
    let mut c = ce_cmd(ce, core, root, &[sub]);
    c.arg("--db").arg(db);
    c
}

/// The interactive-path metric: `ce probe --hook` end to end on a
/// seeded twin-pair fixture (the retired hook_e2e_p95_under_1s
/// shape, driven through the TAG'S OWN binary). Latency only — the
/// decision itself is version policy and asserting it per tag would
/// pin old tags to today's tiers.
fn hook_rows(
    root: &Path,
    ce: &Path,
    core: &Path,
    version: &str,
    commit: &str,
    dirty: bool,
    date: &str,
) -> anyhow::Result<Vec<Value>> {
    let fix = root.join("target/bench-hook-fixture");
    let _ = std::fs::remove_dir_all(&fix);
    std::fs::create_dir_all(fix.join(".git"))?;
    std::fs::write(fix.join("a.rs"), common::rust_fn(1))?;
    std::fs::write(fix.join("c.rs"), common::rust_fn(7))?;
    bs::timed(ce_cmd(ce, core, &fix, &["dedup"]), "fixture index")?;
    let env = hook_envelope(&fix, &common::rust_fn(1));
    let mut ms: Vec<u128> = Vec::with_capacity(30);
    for _ in 0..30 {
        let t0 = std::time::Instant::now();
        let out = hook_once(ce, core, &fix, &env)?;
        ms.push(t0.elapsed().as_millis());
        drop(out);
    }
    // shut the fixture daemon down so the worktree can be removed
    let _ = Command::new(ce)
        .arg("eject")
        .arg(&fix)
        .arg("--yes")
        .output();
    ms.sort_unstable();
    Ok(vec![bs::row(
        version,
        commit,
        dirty,
        "hook_probe",
        "ms",
        &ms,
        None,
        date,
    )])
}

fn hook_once(ce: &Path, core: &Path, root: &Path, envelope: &str) -> anyhow::Result<String> {
    use std::io::Write;
    // stdout goes to a FILE, never a pipe: the probe lazily spawns a
    // daemon that INHERITS the handle and outlives the probe, so a
    // piped wait_with_output blocks on an EOF that never comes
    let out_path = root.join("bench-hook.out");
    let mut cmd = Command::new(ce);
    cmd.args(["probe", "--hook"]).current_dir(root);
    with_core(&mut cmd, core);
    let mut child = cmd
        .stdin(std::process::Stdio::piped())
        .stdout(std::fs::File::create(&out_path)?)
        .stderr(std::process::Stdio::null())
        .spawn()?;
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(envelope.as_bytes())?;
    child.wait()?;
    Ok(std::fs::read_to_string(&out_path).unwrap_or_default())
}

fn hook_envelope(dir: &Path, content: &str) -> String {
    let cwd = dir.display().to_string().replace('\\', "/");
    serde_json::json!({
        "session_id": "bench", "transcript_path": "t", "cwd": cwd,
        "hook_event_name": "PreToolUse", "tool_name": "Write",
        "tool_input": {"file_path": format!("{cwd}/b.rs"), "content": content},
        "tool_use_id": "t"
    })
    .to_string()
}

pub fn git_out(args: &[&str]) -> String {
    let out = Command::new("git").args(args).output().expect("git");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Measure THIS checkout with its release binaries. Debug builds are
/// refused outright — a debug number in the series would poison
/// every comparison it touches.
#[test]
#[ignore = "bench: run in release; writes contracts/bench/bench.json"]
fn bench_append() {
    if cfg!(debug_assertions) {
        panic!("bench numbers are release-only (PERF-BUDGET.md:60-62)");
    }
    let ce = PathBuf::from(env!("CARGO_BIN_EXE_ce"));
    let core = PathBuf::from(std::env::var("CE_CORE_BIN").expect("CE_CORE_BIN"));
    let commit = git_out(&["rev-parse", "HEAD"]);
    let dirty = !git_out(&["status", "--porcelain"]).is_empty();
    let rows = measure_tree(Path::new(".."), &ce, &core, &commit, dirty, None).expect("measure");
    let n = rows.len();
    bs::merge_rows(rows).expect("merge");
    println!("bench_append: {n} rows merged into contracts/bench/bench.json");
}

/// CI gate: the committed doc stays sound — schema pinned, every
/// series row release-provenanced and statistically non-vacuous,
/// every frozen point carrying its source (a number without a source
/// is a hand-filled number, the exact thing this batch forbids).
#[test]
fn bench_doc_is_wellformed() {
    let Ok(text) = std::fs::read_to_string(bs::bench_path()) else {
        // the doc lands with the harness's first append; absent is
        // legal only until then and the render gate will demand it
        return;
    };
    let doc: Value = serde_json::from_str(&text).expect("bench.json parses");
    assert_eq!(doc["schema"], bs::BENCH_SCHEMA, "schema pinned");
    for r in doc["rows"].as_array().expect("rows") {
        for key in [
            "version", "commit", "metric", "p50", "p95", "host", "harness",
        ] {
            assert!(!r[key].is_null(), "row missing {key}: {r}");
        }
        assert!(r["n"].as_u64().unwrap_or(0) >= 3, "n < 3 is noise: {r}");
    }
    for f in doc["frozen"].as_array().expect("frozen") {
        for key in ["metric", "value", "source"] {
            assert!(!f[key].is_null(), "frozen point missing {key}: {f}");
        }
    }
}
