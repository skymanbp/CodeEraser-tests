//! `--core` names the core for the whole `ce` process (plan v2.32 step
//! 2 follow-up): since every command reads the core's definition
//! package, which core is a process question, so the flag is global
//! (like `--lang`) and the package loader reads it. CI 36882292566 was
//! red on all three platforms because the Dogfood step named its core
//! with `--core` and exported no CE_CORE_BIN: the loader resolved
//! through the default chain alone and refused the first command.
//!
//! Every leg runs `ce` bare: no CE_CORE_BIN, PATH an empty directory
//! (so no installed ce-core answers), the debug `ce` with no sibling
//! core. The core the positive legs name is the test process's own
//! CE_CORE_BIN, which every it leg already requires.

// the package battery's one-file tree: `ce scan` and `ce graph
// --sites` both read it
use crate::tables_package::seeded;
use std::path::Path;
use std::process::{Command, Output};

/// The core the positive legs name, as an absolute path.
fn named_core() -> String {
    let core = std::env::var("CE_CORE_BIN").expect("it legs run with CE_CORE_BIN");
    std::path::absolute(&core)
        .expect("absolute core path")
        .display()
        .to_string()
}

/// `ce <args>` in `dir` with no core anywhere but what `env` adds.
fn bare(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
    let empty = dir.join("empty-path");
    std::fs::create_dir_all(&empty).expect("empty PATH dir");
    Command::new(env!("CARGO_BIN_EXE_ce"))
        .args(args)
        .env_remove("CE_CORE_BIN")
        .env_remove("CE_LANG")
        .env("PATH", &empty)
        .env("CE_UPDATE_CHECK", "0")
        .envs(env.iter().copied())
        .current_dir(dir)
        .output()
        .expect("run ce")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Exit 0, or the stderr that says why not.
fn passed(out: &Output) {
    assert_eq!(out.status.code(), Some(0), "{}", stderr(out));
}

#[test]
fn scan_named_by_flag_alone() {
    let core = named_core();
    let flagged = seeded("core-flag-scan");
    let by_flag = bare(
        &flagged,
        &["scan", ".", "--format", "json", "--core", &core],
        &[],
    );
    passed(&by_flag);
    let exported = seeded("core-flag-scan-env");
    let by_env = bare(
        &exported,
        &["scan", ".", "--format", "json"],
        &[("CE_CORE_BIN", &core)],
    );
    passed(&by_env);
    assert_eq!(
        String::from_utf8_lossy(&by_flag.stdout),
        String::from_utf8_lossy(&by_env.stdout),
        "the flag and CE_CORE_BIN name the same core"
    );
}

#[test]
fn global_form_before_the_subcommand() {
    let core = named_core();
    let before = seeded("core-flag-before");
    let after = seeded("core-flag-after");
    let early = bare(
        &before,
        &["--core", &core, "scan", ".", "--format", "json"],
        &[],
    );
    passed(&early);
    let late = bare(
        &after,
        &["scan", ".", "--format", "json", "--core", &core],
        &[],
    );
    passed(&late);
    assert_eq!(early.stdout, late.stdout);
}

#[test]
fn a_command_that_never_had_the_flag() {
    let dir = seeded("core-flag-graph");
    let out = bare(
        &dir,
        &["graph", ".", "--sites", "--core", &named_core()],
        &[],
    );
    passed(&out);
}

/// Exit 2, and stderr names every one of `words`.
fn refused(out: &Output, words: &[&str]) {
    let why = stderr(out);
    assert_eq!(out.status.code(), Some(2), "{why}");
    assert!(words.iter().all(|w| why.contains(w)), "{words:?}: {why}");
}

#[test]
fn no_core_anywhere_refuses_by_name() {
    let dir = seeded("core-flag-none");
    refused(
        &bare(&dir, &["scan", "."], &[]),
        &["core unavailable", "tables/1"],
    );
}

#[test]
fn a_named_core_that_does_not_exist_is_named_back() {
    let dir = seeded("core-flag-nope");
    let nope = dir.join("nope").display().to_string();
    refused(&bare(&dir, &["scan", ".", "--core", &nope], &[]), &["nope"]);
}
