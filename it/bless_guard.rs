//! Gate 1 of the derived-fact registry (plan v2.21): the bless switch
//! has ONE reader, CI never spells it, and a bless under CI refuses.
//! The census reads the suite's own source so a seventh `env::var`
//! site cannot appear unnoticed; the refusal is proved in a child
//! process, because an in-process env mutation would race every
//! other test on the thread pool (the daemon_conn_deadline lesson).

use crate::common::{files_with_ext, repo_root};
use std::path::Path;
use std::process::Command;

/// The two switches, spelled in pieces so this file is not itself a
/// reader by the census's own pattern.
fn switch(name: &str) -> String {
    format!("CE_{name}")
}

fn read_sites(needle: &str) -> Vec<String> {
    let mut files = Vec::new();
    files_with_ext(&repo_root().join("cli/tests"), "rs", &mut files);
    let pattern = format!("env::var(\"{needle}\")");
    let pattern_os = format!("env::var_os(\"{needle}\")");
    // One entry per read SITE (a file reading twice lists twice), keyed
    // by file: a line number would make every doc edit above the reader
    // a census failure.
    let mut sites = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).expect("read suite source");
        for line in text.lines() {
            if line.contains(&pattern) || line.contains(&pattern_os) {
                let rel = path.strip_prefix(repo_root()).expect("under root");
                sites.push(rel.display().to_string().replace('\\', "/"));
            }
        }
    }
    sites.sort();
    sites
}

#[test]
fn the_bless_switch_has_one_reader() {
    assert_eq!(
        read_sites(&switch("BLESS")),
        ["cli/tests/it/facts/mod.rs"],
        "CE_BLESS is read through facts::blessing() only"
    );
    assert_eq!(
        read_sites(&switch("REFREEZE")),
        ["cli/tests/it/eval_support/universe.rs"],
        "CE_REFREEZE (the named re-sign modifier) has one reader"
    );
}

#[test]
fn the_workflows_never_spell_a_bless() {
    let mut files = Vec::new();
    files_with_ext(&repo_root().join(".github/workflows"), "yml", &mut files);
    assert!(!files.is_empty(), "workflow files found");
    for path in files {
        let text = std::fs::read_to_string(&path).expect("read workflow");
        for name in ["BLESS", "REFREEZE"] {
            assert!(
                !text.contains(&switch(name)),
                "{}: a workflow must never hand a run CE_{name}",
                path.display()
            );
        }
    }
}

/// The probe the refusal leg runs in a child: prints the switch's
/// reading, or dies on the refusal. `#[ignore]` keeps it out of the
/// plain run — it is a subject, not a test.
#[test]
#[ignore]
fn probe() {
    println!("blessing={}", crate::facts::blessing());
}

fn run_probe(env: &[(&str, &str)], drop_ci: bool) -> (bool, String, String) {
    let mut cmd = Command::new(std::env::current_exe().expect("this test binary"));
    cmd.args(["--exact", "bless_guard::probe", "--ignored", "--nocapture"])
        .env_remove(switch("BLESS"))
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")));
    if drop_ci {
        cmd.env_remove("CI");
    }
    let out = cmd.envs(env.iter().copied()).output().expect("spawn probe");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn blessing_reads_exactly_one_and_refuses_under_ci() {
    let bless = switch("BLESS");
    let (ok, out, _) = run_probe(&[], true);
    assert!(ok && out.contains("blessing=false"), "unset: {out}");
    let (ok, out, _) = run_probe(&[(&bless, "0")], true);
    assert!(ok && out.contains("blessing=false"), "CE_BLESS=0: {out}");
    let (ok, out, _) = run_probe(&[(&bless, "1")], true);
    assert!(
        ok && out.contains("blessing=true"),
        "CE_BLESS=1 locally: {out}"
    );
    let (ok, _, err) = run_probe(&[(&bless, "1"), ("CI", "true")], false);
    assert!(
        !ok && err.contains("CE_BLESS=1 under CI"),
        "CE_BLESS=1 on CI must refuse by name: {err}"
    );
}
