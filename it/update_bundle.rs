//! The installer bundle as `ce update` recognises it: by the GUI app's
//! program beside ce, whose name Tauri takes from the GUI crate's
//! package (1.10.0 looked for `CodeEraser.exe`, a name no shipped
//! bundle carries, and called every desktop install "placed by hand").
//! The constant is held to the file that decides the name, read at run
//! time; the end-to-end row runs ce beside a file of that name.

use crate::common::repo_root;
use crate::update_e2e::from_folder;
use codeeraser::update::install::BUNDLE_APP;
use serde_json::Value;

/// `[package] name` of the GUI crate, and no `mainBinaryName` in its
/// Tauri config that would rename the program (or one equal to it).
#[test]
fn the_bundle_app_is_the_gui_crates_program() {
    let gui = repo_root().join("gui/src-tauri");
    let cargo = std::fs::read_to_string(gui.join("Cargo.toml")).expect("gui Cargo.toml");
    let cargo: toml::Table = toml::from_str(&cargo).expect("gui Cargo.toml parses");
    assert_eq!(cargo["package"]["name"].as_str(), Some(BUNDLE_APP));
    let conf = std::fs::read_to_string(gui.join("tauri.conf.json")).expect("tauri.conf.json");
    let conf: Value = serde_json::from_str(&conf).expect("tauri.conf.json parses");
    let renamed = conf.get("mainBinaryName").and_then(Value::as_str);
    assert!(
        renamed.is_none_or(|n| n == BUNDLE_APP),
        "mainBinaryName {renamed:?}"
    );
}

/// ce beside the bundle's app is the bundle's sidecar (install 1), and
/// a newer release asks for the installer (action 4), never `--yes`
/// alone, which would leave the app at its version.
#[test]
fn ce_beside_the_bundle_app_asks_for_the_installer() {
    let app = format!("{BUNDLE_APP}{}", std::env::consts::EXE_SUFFIX);
    let (_, d, code, _) = from_folder("update-bundle", "CodeEraser", &[&app], &["--version"]);
    assert_eq!(
        (code, &d["current"]["install"], &d["action"]),
        (Some(1), &Value::from(1), &Value::from(4)),
        "{d}"
    );
}
