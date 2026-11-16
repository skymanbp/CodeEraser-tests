//! The installer bundle as `ce update` recognises it: by the GUI app's
//! program beside ce, whose name Tauri takes from the GUI crate's
//! package (1.10.0 looked for `CodeEraser.exe`, a name no shipped
//! bundle carries, and called every desktop install "placed by hand").
//! The constant is held to the file that decides the name, read at run
//! time; the end-to-end row runs ce beside a file of that name. The
//! installer's retired twins are held to the updater's sweep the same
//! way, from the hook's own text.

use crate::common::repo_root;
use crate::update_e2e::from_folder;
use codeeraser::update::apply::retired_twin;
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

/// The installer's twin and the updater's sweep are one spelling: the
/// rename target hooks.nsh writes (`<name>.<register>.old`), its register
/// replaced by a tick, is a name `retired_twin` accepts for every
/// external binary of the bundle (build-target.yml's externalBin), and
/// the hook's sweep deletes exactly those binaries' twins.
#[test]
fn the_installers_twins_are_what_the_updater_sweeps() {
    let root = repo_root();
    let read = |p: &str| std::fs::read_to_string(root.join(p)).expect(p);
    let nsh = read("gui/src-tauri/windows/hooks.nsh");
    let wf = read(".github/workflows/build-target.yml");
    let bins: Vec<&str> = between(&wf, r#""externalBin":["#, "]")
        .split(',')
        .map(|b| b.trim_matches('"').rsplit('/').next().expect("a name"))
        .collect();
    assert_eq!(bins, ["ce", "ce-core"], "externalBin");
    let rename = r#"Rename "$INSTDIR\${NAME}.exe" "$INSTDIR\"#;
    let target = nsh
        .lines()
        .find_map(|l| l.trim().strip_prefix(rename)?.strip_suffix('"'))
        .expect("the hook renames ${NAME}.exe aside");
    let (register, _) = target
        .strip_prefix("${NAME}.")
        .and_then(|t| t.split_once('.'))
        .expect("<name>.<register>.<suffix>");
    let deletes: Vec<&str> = between(&nsh, "!macro CE_SWEEP_RETIRED", "!macroend")
        .lines()
        .filter_map(|l| {
            l.trim()
                .strip_prefix(r#"Delete "$INSTDIR\"#)?
                .strip_suffix('"')
        })
        .collect();
    for bin in &bins {
        let twin = target
            .replace("${NAME}", bin)
            .replace(register, "271348953");
        assert!(retired_twin(&twin), "{twin}");
        for pattern in [format!("{bin}.*.old"), format!("{bin}.old")] {
            assert!(deletes.contains(&pattern.as_str()), "sweep lacks {pattern}");
            assert!(retired_twin(&pattern.replace('*', "7")), "{pattern}");
        }
    }
    assert_eq!(
        deletes.len(),
        2 * bins.len(),
        "the sweep deletes {deletes:?}"
    );
}

/// The text after the first `open` up to the next `close`.
fn between<'a>(text: &'a str, open: &str, close: &str) -> &'a str {
    let rest = text
        .split_once(open)
        .unwrap_or_else(|| panic!("no {open}"))
        .1;
    &rest[..rest
        .find(close)
        .unwrap_or_else(|| panic!("no {close} after {open}"))]
}
