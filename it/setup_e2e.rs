//! `ce setup` end to end against a FAKE `claude` on PATH (plan v2.29
//! step 10: O72 / O73 / O82) — the installer's Claude Code wiring, now
//! one Rust body every installer and every hand-run share. The fake
//! records each call and answers by its row's script: which
//! marketplaces the listing holds, whether `add` and `install` succeed.
//! One table drives the exit-code legend the NSIS hook prints (0 / 5 /
//! 10 / 11 / 12 / 13), so every row is asserted the same way and the
//! claude command sequence is read back exactly — the marketplace
//! source is the `release` ref, the install target the
//! plugin@marketplace pair, and a kept registration is never re-added.

use crate::common::{run_ce_env, tmp};
use codeeraser::setup::{Exit, NAMES};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Output;

const LIST: &str = "plugin marketplace list --json";
const ADD: &str = "plugin marketplace add skymanbp/CodeEraser@release";
const INSTALL: &str = "plugin install codeeraser@codeeraser";
const UPDATE: &str = "plugin update codeeraser@codeeraser";

struct Fake {
    bin: PathBuf,
    calls: PathBuf,
}

/// A `claude` on its own PATH entry, answering by script: the
/// marketplaces `listed` (as `plugin marketplace list --json` prints
/// them) for the listing, `add_rc` for `marketplace add`, `install_rc`
/// for `plugin install`, 0 for everything else; every call's arguments
/// are appended to `calls`.
fn fake(dir: &Path, listed: &[&str], add_rc: i32, install_rc: i32) -> Fake {
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).expect("bin");
    let calls = dir.join("calls.log");
    let listing = dir.join("list.json");
    let rows: Vec<Value> = listed
        .iter()
        .map(|n| json!({"name": n, "source": "github"}))
        .collect();
    std::fs::write(&listing, Value::Array(rows).to_string()).expect("listing");
    let (log, list) = (calls.display(), listing.display());
    if cfg!(windows) {
        let body = format!(
            "@echo off\r\necho %* >>\"{log}\"\r\nif \"%1 %2 %3\"==\"plugin marketplace list\" (type \"{list}\" & exit /b 0)\r\nif \"%1 %2 %3\"==\"plugin marketplace add\" exit /b {add_rc}\r\nif \"%1 %2\"==\"plugin install\" exit /b {install_rc}\r\nexit /b 0\r\n"
        );
        std::fs::write(bin.join("claude.cmd"), body).expect("shim");
    } else {
        let body = format!(
            "#!/bin/sh\necho \"$*\" >> \"{log}\"\ncase \"$1 $2 $3\" in\n  'plugin marketplace list') cat \"{list}\"; exit 0 ;;\n  'plugin marketplace add') exit {add_rc} ;;\nesac\ncase \"$1 $2\" in\n  'plugin install') exit {install_rc} ;;\nesac\nexit 0\n"
        );
        let script = bin.join("claude");
        std::fs::write(&script, body).expect("script");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }
    }
    Fake { bin, calls }
}

/// The calls since the log was last cleared.
fn calls(f: &Fake) -> Vec<String> {
    std::fs::read_to_string(&f.calls)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// `ce setup --marker-dir <dir> <args>` with PATH = the fake's
/// directory alone, HOME pointed away from any real `~/.local/bin`,
/// and the logon seam pinned to this process's user unless `extra`
/// says otherwise (a later entry overrides).
fn run_setup(fake_bin: &Path, dir: &Path, extra: &[(&str, &str)], args: &[&str]) -> Output {
    let home = dir.join("home");
    std::fs::create_dir_all(&home).expect("home");
    let (path, home, marker_dir) = (
        fake_bin.display().to_string(),
        home.display().to_string(),
        dir.display().to_string(),
    );
    let me = ["USERNAME", "USER", "LOGNAME"]
        .iter()
        .find_map(|k| std::env::var(k).ok())
        .unwrap_or_default();
    let mut env = vec![
        ("PATH", path.as_str()),
        ("HOME", home.as_str()),
        ("USERPROFILE", home.as_str()),
        ("CE_SETUP_LOGON_USER", me.as_str()),
    ];
    env.extend_from_slice(extra);
    let mut argv = vec!["setup", "--marker-dir", marker_dir.as_str()];
    argv.extend_from_slice(args);
    run_ce_env(dir, &argv, &env)
}

/// The JSON face and its exit code.
fn setup(
    fake_bin: &Path,
    dir: &Path,
    extra: &[(&str, &str)],
    args: &[&str],
) -> (Value, Option<i32>) {
    let mut argv = vec!["--format", "json"];
    argv.extend_from_slice(args);
    let out = run_setup(fake_bin, dir, extra, &argv);
    let doc = serde_json::from_slice(&out.stdout).expect("setup document");
    (doc, out.status.code())
}

/// The step states one document carries, one word each in a fixed
/// order: marketplace state, plugin state, marker before, marker
/// after — the tables below spell the expectation the same way.
fn states(d: &Value) -> String {
    let s = |k: &str| d[k]["state"].as_str().unwrap_or("?").to_string();
    let b = |k: &str| {
        d["marker"][k]
            .as_bool()
            .map_or("?".into(), |v| v.to_string())
    };
    format!(
        "{} {} {} {}",
        s("marketplace"),
        s("plugin"),
        b("before"),
        b("after")
    )
}

/// (name, marketplaces listed, add rc, install rc, logon override,
/// exit, calls, states)
type Row = (
    &'static str,
    &'static [&'static str],
    i32,
    i32,
    Option<&'static str>,
    Exit,
    &'static [&'static str],
    &'static str,
);

const ROWS: &[Row] = &[
    (
        "setup-fresh",
        &[],
        0,
        0,
        None,
        Exit::Wired,
        &[LIST, ADD, INSTALL, UPDATE],
        "added installed false true",
    ),
    (
        "setup-kept",
        &["codeeraser"],
        0,
        0,
        None,
        Exit::Kept,
        &[LIST, INSTALL, UPDATE],
        "present installed false false",
    ),
    (
        "setup-addfail",
        &[],
        1,
        0,
        None,
        Exit::AddFailed,
        &[LIST, ADD],
        "failed skipped false false",
    ),
    (
        "setup-installfail",
        &[],
        0,
        1,
        None,
        Exit::InstallFailed,
        &[LIST, ADD, INSTALL],
        "added failed false false",
    ),
    (
        "setup-otheruser",
        &[],
        0,
        0,
        Some("someone-else"),
        Exit::ElevatedUserDiffers,
        &[],
        "skipped skipped false false",
    ),
];

#[test]
fn the_exit_code_legend_is_read_back_row_by_row() {
    for &(name, listed, add_rc, install_rc, logon, exit, want_calls, want_states) in ROWS {
        let dir = tmp(name);
        let f = fake(&dir, listed, add_rc, install_rc);
        let extra: Vec<(&str, &str)> = logon
            .map(|l| ("CE_SETUP_LOGON_USER", l))
            .into_iter()
            .collect();
        let (d, code) = setup(&f.bin, &dir, &extra, &[]);
        assert_eq!(
            (
                code,
                d["exit"].as_u64(),
                states(&d).as_str(),
                d["user"]["differs"].as_bool(),
                d["error"].is_null(),
                dir.join(NAMES.marker).is_file(),
            ),
            (
                Some(i32::from(exit.code())),
                Some(u64::from(exit.code())),
                want_states,
                Some(logon.is_some()),
                !matches!(exit, Exit::AddFailed | Exit::InstallFailed),
                want_states.ends_with(" true"),
            ),
            "{name}: {d}"
        );
        assert_eq!(calls(&f), want_calls, "{name}");
    }
}

#[test]
fn no_claude_anywhere_is_exit_10_and_touches_nothing() {
    let dir = tmp("setup-noclaude");
    let empty = dir.join("empty");
    std::fs::create_dir_all(&empty).expect("empty");
    let (d, code) = setup(&empty, &dir, &[], &[]);
    assert_eq!(
        (
            code,
            d["claude"].is_null(),
            d["exit"].as_u64(),
            d["path"]["dir"].as_str(),
            dir.join(NAMES.marker).exists()
        ),
        (
            Some(i32::from(Exit::NoClaude.code())),
            true,
            Some(10),
            Some(dir.display().to_string().as_str()),
            false
        ),
        "{d}"
    );
}

/// One act on the shared directory: the arguments, the exit code, the
/// states (marker after = on disk) and the claude calls it made.
type Act = (
    &'static [&'static str],
    Option<i32>,
    &'static str,
    &'static [&'static str],
);

/// Wire, unwire, unwire again — one directory, three acts in order.
const ACTS: &[Act] = &[
    (
        &[],
        Some(0),
        "added installed false true",
        &[LIST, ADD, INSTALL, UPDATE],
    ),
    (
        &["--unwire"],
        Some(0),
        "removed uninstalled true false",
        &[
            "plugin uninstall codeeraser@codeeraser",
            "plugin marketplace remove codeeraser",
        ],
    ),
    // no marker left: nothing to remove, and claude is not even asked
    (&["--unwire"], Some(0), "skipped skipped false false", &[]),
];

#[test]
fn unwire_removes_exactly_what_setup_added() {
    let dir = tmp("setup-unwire");
    let f = fake(&dir, &[], 0, 0);
    for &(args, want_code, want_states, want_calls) in ACTS {
        let _ = std::fs::remove_file(&f.calls);
        let (d, code) = setup(&f.bin, &dir, &[], args);
        assert_eq!(
            (code, states(&d).as_str(), dir.join(NAMES.marker).is_file()),
            (want_code, want_states, want_states.ends_with(" true")),
            "{args:?}: {d}"
        );
        assert_eq!(calls(&f), want_calls, "{args:?}");
    }
    // the console face says the same in words with the same exit code,
    // and an unknown logon (the empty seam) never refuses
    let out = run_setup(&f.bin, &dir, &[("CE_SETUP_LOGON_USER", "")], &["--unwire"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains("not wired by `ce setup`"));
}
