//! `ce update` end to end over a hermetic release index (L round,
//! user directive 2026-08-29 "更新功能"). `CE_UPDATE_BASE` points the
//! check at a directory served over `file://` (ce.sh's bootstrap
//! seam) and `CE_UPDATE_TARGET_DIR` keeps the apply leg away from the
//! binary cargo is running. Each leg pins one link of the trust
//! chain: the pins are the tag's committed manifest's, a mismatch
//! places nothing, a copy another package manager owns is never
//! overwritten, the verdict is the exit code, and the SessionStart
//! notice rides only when a newer release exists.

use crate::common::{self, run_ce_env, tmp};
use codeeraser::update::apply::sha256_hex;
use codeeraser::update::version::{Platform, of_tag};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn plat() -> Platform {
    Platform::of(std::env::consts::OS, std::env::consts::ARCH)
}

struct Release {
    base: String,
    dir: PathBuf,
    ce_pin: String,
    ce_bytes: String,
    core_bytes: String,
}

/// A release index at a scratch dir: `latest.json` naming `tag`,
/// `<tag>/manifest.env` with pins measured over the fake assets
/// beside it (or, tampered, a pin no asset matches).
fn release(name: &str, tag: &str, tamper: bool) -> Release {
    let dir = tmp(name);
    let ver = of_tag(tag);
    let assets = dir.join("assets");
    std::fs::create_dir_all(&assets).expect("assets dir");
    std::fs::create_dir_all(dir.join(tag)).expect("tag dir");
    let p = plat();
    let ce_bytes = format!("fake ce {ver}\n");
    let core_bytes = format!("fake ce-core {ver}\n");
    let ce_path = assets.join(format!("ce-{ver}-{}{}", p.key, p.ext));
    let core_path = assets.join(format!("ce-core-{ver}-{}{}", p.key, p.ext));
    std::fs::write(&ce_path, &ce_bytes).expect("ce asset");
    std::fs::write(&core_path, &core_bytes).expect("core asset");
    let mut ce_pin = sha256_hex(&ce_path).expect("sha");
    if tamper {
        ce_pin = "0".repeat(64);
    }
    let mut manifest = format!(
        "# fixture\nCE_MANIFEST_VERSION=\"{ver}\"\nCE_BASE_URL=\"file://{}\"\n\
         CE_SHA256_{k}_CE=\"{ce_pin}\"\nCE_SHA256_{k}_CECORE=\"{}\"\n",
        slashes(&assets),
        sha256_hex(&core_path).expect("sha"),
        k = p.manifest_key(),
    );
    if let Some(asset) = p.installer_asset(&ver) {
        let path = assets.join(asset);
        std::fs::write(&path, "fake installer\n").expect("installer asset");
        // every bundle key, so the leg is platform-blind
        for key in ["SETUP", "APPIMAGE", "DMG"] {
            manifest += &format!(
                "CE_SHA256_{}_{key}=\"{}\"\n",
                p.manifest_key(),
                sha256_hex(&path).expect("sha")
            );
        }
    }
    std::fs::write(dir.join(tag).join("manifest.env"), manifest).expect("manifest");
    std::fs::write(
        dir.join("latest.json"),
        serde_json::json!({"tag_name": tag}).to_string(),
    )
    .expect("latest.json");
    Release {
        base: format!("file://{}", slashes(&dir)),
        dir,
        ce_pin,
        ce_bytes,
        core_bytes,
    }
}

fn slashes(p: &Path) -> String {
    p.display().to_string().replace('\\', "/")
}

/// `ce update --yes <extra>` against `r`, placing into `target`.
fn apply(r: &Release, target: &Path, extra: &[&str]) -> std::process::Output {
    let mut args = vec!["update", "--yes"];
    args.extend_from_slice(extra);
    run_ce_env(
        &r.dir,
        &args,
        &[
            ("CE_UPDATE_BASE", &r.base),
            ("CE_UPDATE_TARGET_DIR", &slashes(target)),
        ],
    )
}

/// A refused `--yes`: exit 2, and stderr names the reason.
fn refused(r: &Release, target: &Path) -> String {
    let out = apply(r, target, &[]);
    assert_eq!(out.status.code(), Some(2));
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// The JSON face and its exit code under `env`.
fn check(env: &[(&str, &str)]) -> (Value, Option<i32>) {
    let out = run_ce_env(&tmp("update-cwd"), &["update", "--format", "json"], env);
    let doc = serde_json::from_slice(&out.stdout).expect("update document");
    (doc, out.status.code())
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_default()
}

/// Files in a scratch dir (`tmp` seeds a `.git` anchor, not residue).
fn files_in(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .expect("dir")
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .count()
}

#[test]
fn the_check_reads_the_tags_pins_and_exits_with_the_verdict() {
    let r = release("update-newer", "v9.9.9", false);
    let env = [("CE_UPDATE_BASE", r.base.as_str())];
    let (d, code) = check(&env);
    assert_eq!(code, Some(1), "{d}");
    assert_eq!(d["schema"], codeeraser::update::SCHEMA_ID);
    assert_eq!(d["verdict"], 1);
    assert_eq!(d["latest"]["tag"], "v9.9.9");
    assert_eq!(d["latest"]["version"], "9.9.9");
    assert_eq!(d["pins"]["ce"], r.ce_pin);
    assert_eq!(d["current"]["version"], env!("CARGO_PKG_VERSION"));
    // cargo's target dir is nobody's ledger: the apply leg is ours
    assert_eq!(d["current"]["install"], 0);
    assert_eq!(d["action"], 1);
    // the console face renders the same document in both languages
    // and carries the same exit code
    let en = run_ce_env(&r.dir, &["update"], &env);
    let text = String::from_utf8_lossy(&en.stdout);
    assert_eq!(en.status.code(), Some(1));
    assert!(text.contains("latest: 9.9.9 — update available"), "{text}");
    assert!(text.contains("ce update --yes"), "{text}");
    let zh = run_ce_env(&r.dir, &["--lang", "zh", "update"], &env);
    assert!(String::from_utf8_lossy(&zh.stdout).contains("有更新"));
}

#[test]
fn the_same_version_is_up_to_date_and_yes_is_a_named_refusal() {
    let tag = format!("v{}", env!("CARGO_PKG_VERSION"));
    let r = release("update-same", &tag, false);
    let (d, code) = check(&[("CE_UPDATE_BASE", &r.base)]);
    assert_eq!(
        (code, &d["verdict"], &d["action"]),
        (Some(0), &Value::from(0), &Value::from(0))
    );
    let target = tmp("update-same-target");
    let err = refused(&r, &target);
    assert!(err.contains("already up to date"), "{err}");
    assert!(!target.join(format!("ce{}", plat().ext)).exists());
}

#[test]
fn yes_places_both_binaries_only_after_both_pins_verify() {
    let r = release("update-apply", "v9.9.9", false);
    let target = tmp("update-apply-target");
    let ce = target.join(format!("ce{}", plat().ext));
    let core = target.join(format!("ce-core{}", plat().ext));
    // a previous copy sits there: retired, not lost mid-write
    std::fs::write(&ce, "old ce\n").expect("old ce");
    let out = apply(&r, &target, &["--installer", "--format", "json"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let done: Value = serde_json::from_slice(&out.stdout).expect("apply document");
    assert_eq!(done["version"], "9.9.9");
    assert_eq!(read(&ce), r.ce_bytes);
    assert_eq!(read(&core), r.core_bytes);
    // nothing ran the retired copy, so the best-effort sweep took it
    assert!(!target.join("ce.old").exists());
    assert_eq!(files_in(&target), 2, "no download residue");
    if plat().installer_asset("9.9.9").is_some() {
        let saved = done["installer"].as_str().expect("installer path");
        assert_eq!(read(Path::new(saved)), "fake installer\n");
    } else {
        assert_eq!(done["installer"], Value::Null);
    }
}

#[test]
fn a_pin_mismatch_places_nothing() {
    let r = release("update-tamper", "v9.9.9", true);
    // the check still reports the release: the pin is the manifest's
    // word, and the refusal belongs to the apply leg
    let (d, _) = check(&[("CE_UPDATE_BASE", &r.base)]);
    assert_eq!(d["verdict"], 1);
    let target = tmp("update-tamper-target");
    let err = refused(&r, &target);
    assert!(err.contains("SHA256 mismatch"), "{err}");
    assert_eq!(files_in(&target), 0, "nothing placed, no residue");
}

#[test]
fn a_copy_another_package_manager_owns_is_named_never_overwritten() {
    let r = release("update-plugin", "v9.9.9", false);
    let exe_dir = Path::new(env!("CARGO_BIN_EXE_ce"))
        .parent()
        .expect("exe dir")
        .display()
        .to_string();
    let env = [
        ("CE_UPDATE_BASE", r.base.as_str()),
        ("CLAUDE_PLUGIN_DATA", exe_dir.as_str()),
    ];
    let (d, code) = check(&env);
    assert_eq!(
        (code, &d["current"]["install"], &d["action"]),
        (Some(1), &Value::from(3), &Value::from(2))
    );
    let out = run_ce_env(&r.dir, &["update", "--yes"], &env);
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("/plugin update codeeraser"), "{err}");
}

#[test]
fn no_release_index_is_unknown_never_current() {
    let missing = tmp("update-missing");
    let base = format!("file://{}", slashes(&missing));
    let (d, code) = check(&[("CE_UPDATE_BASE", &base)]);
    assert_eq!(code, Some(2), "{d}");
    assert_eq!(d["verdict"], 2);
    assert_eq!(d["action"], 0);
    assert!(d["latest"]["error"].as_str().is_some_and(|e| !e.is_empty()));
    let out = run_ce_env(&missing, &["update"], &[("CE_UPDATE_BASE", &base)]);
    assert!(String::from_utf8_lossy(&out.stdout).contains("latest: unknown — "));
}

fn health(dir: &Path, env: &[(&str, &str)]) -> String {
    let envelope = serde_json::json!({
        "session_id": "t", "transcript_path": "t",
        "cwd": slashes(dir), "hook_event_name": "SessionStart"
    })
    .to_string();
    let out = common::run_hook_env(dir, &["health", "--hook"], &envelope, env);
    let v: Value = serde_json::from_str(out.trim()).expect("hook json");
    v["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("additionalContext")
        .to_string()
}

#[test]
fn the_session_notice_rides_only_when_newer_and_is_cached_a_day() {
    let r = release("update-notice", "v9.9.9", false);
    let project = tmp("update-notice-project");
    let cache = tmp("update-notice-cache");
    let cache_s = slashes(&cache);
    let armed = [
        ("CE_UPDATE_BASE", r.base.as_str()),
        ("CLAUDE_PLUGIN_DATA", cache_s.as_str()),
        ("CE_UPDATE_CHECK", "1"),
    ];
    let ctx = health(&project, &armed);
    assert!(
        ctx.starts_with("[ce "),
        "the status line is untouched: {ctx}"
    );
    assert!(ctx.contains("\n[ce update: 9.9.9 available — "), "{ctx}");
    assert!(cache.join("codeeraser-update-check.json").is_file());
    // cached: the index gone, the notice still stands for a day
    std::fs::remove_file(r.dir.join("latest.json")).expect("rm index");
    assert!(health(&project, &armed).contains("9.9.9 available"));
    // the harness default (off) and an explicit opt-out say nothing
    assert!(!health(&project, &armed[..2]).contains("ce update:"));
    // a current binary says nothing either — and its own cache
    let same = release(
        "update-notice-same",
        &format!("v{}", env!("CARGO_PKG_VERSION")),
        false,
    );
    let cache2 = slashes(&tmp("update-notice-cache2"));
    let quiet = health(
        &project,
        &[
            ("CE_UPDATE_BASE", &same.base),
            ("CLAUDE_PLUGIN_DATA", &cache2),
            ("CE_UPDATE_CHECK", "1"),
        ],
    );
    assert!(!quiet.contains("ce update:"), "{quiet}");
}
