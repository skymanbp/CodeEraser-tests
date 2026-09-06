//! packaging/** — the Homebrew formula and the winget manifests — are
//! projections of plugin/bin/manifest.env (plan v2.29 step 10, O71):
//! scripts/packaging.js writes them and `--check` says whether the
//! tree's copies are current. A generator that lied would agree with
//! itself, so this file also reads the pins BACK out of the committed
//! files with its own line readers and holds them to the manifest —
//! the bytes answer to the pins, not to the generator. `CE_BLESS=1`
//! regenerates first.

use crate::common::{expect_ok, node, repo_root};
use crate::facts::{blessing, read};
use codeeraser::update::manifest;
use codeeraser::update::version::{Platform, TARGETS};
use std::collections::BTreeMap;
use std::process::Output;

const FORMULA: &str = "packaging/homebrew/Formula/codeeraser.rb";
const WINGET: &str = "packaging/winget/manifests/s/skymanbp/CodeEraser";
const ID: &str = "skymanbp.CodeEraser";

fn shipped() -> BTreeMap<String, String> {
    manifest::parse(&read(&repo_root(), "plugin/bin/manifest.env"))
}

fn generator(args: &[&str]) -> Output {
    node(&[&["scripts/packaging.js"], args].concat(), &[])
}

/// Every `<key> "<value>"` line's value (the Ruby DSL).
fn dsl(text: &str, key: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| l.trim().strip_prefix(key))
        .filter_map(|r| r.trim().strip_prefix('"'))
        .map(|r| r.trim_end_matches('"').to_string())
        .collect()
}

/// The one `Key: value` line's value (a YAML mapping, spelled once).
fn yaml(text: &str, key: &str) -> String {
    let want = format!("{key}: ");
    let mut hits = text.lines().filter_map(|l| l.trim().strip_prefix(&want));
    let value = hits
        .next()
        .unwrap_or_else(|| panic!("no {key} line"))
        .trim()
        .to_string();
    assert!(hits.next().is_none(), "{key} spelled twice");
    value
}

#[test]
fn the_committed_files_are_the_generators_current_output() {
    if blessing() {
        assert!(generator(&[]).status.success(), "regenerate");
    }
    expect_ok(
        &generator(&["--check"]),
        "packaging/** drifted from the manifest",
    );
}

#[test]
fn the_formula_spells_only_pinned_bytes_for_the_built_targets() {
    let m = shipped();
    let (ver, base) = (&m["CE_MANIFEST_VERSION"], &m["CE_BASE_URL"]);
    let rb = read(&repo_root(), FORMULA);
    let (mut want_urls, mut want_shas) = (Vec::new(), Vec::new());
    for key in TARGETS.iter().filter(|k| !k.ends_with("-windows")) {
        let mk = Platform::of_key(key).manifest_key();
        for (what, tail) in [("ce", "CE"), ("ce-core", "CECORE")] {
            let pin = &m[&format!("CE_SHA256_{mk}_{tail}")];
            if pin.is_empty() {
                continue;
            }
            want_urls.push(format!("{base}/{what}-{ver}-{key}"));
            want_shas.push(pin.clone());
        }
    }
    let (mut urls, mut shas) = (dsl(&rb, "url"), dsl(&rb, "sha256"));
    for v in [&mut urls, &mut shas, &mut want_urls, &mut want_shas] {
        v.sort();
    }
    assert_eq!(urls, want_urls, "formula urls");
    assert_eq!(shas, want_shas, "formula pins");
    assert!(rb.contains("class Codeeraser < Formula"), "{rb}");
    assert_eq!(dsl(&rb, "license"), ["Apache-2.0"]);
}

#[test]
fn the_winget_manifests_pin_the_windows_installer_at_the_manifest_version() {
    let m = shipped();
    let ver = &m["CE_MANIFEST_VERSION"];
    let mut versions: Vec<String> = std::fs::read_dir(repo_root().join(WINGET))
        .expect("winget manifests directory")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    versions.sort();
    assert_eq!(
        versions,
        std::slice::from_ref(ver),
        "one version directory, the manifest's"
    );
    let p = Platform::of_key("x86_64-windows");
    let pin = &m[&format!("CE_SHA256_{}_SETUP", p.manifest_key())];
    let url = format!(
        "{}/{}",
        m["CE_BASE_URL"],
        p.installer_asset(ver).expect("setup")
    );
    let files = [
        (format!("{ID}.yaml"), "version"),
        (format!("{ID}.installer.yaml"), "installer"),
        (format!("{ID}.locale.en-US.yaml"), "defaultLocale"),
    ];
    let texts: Vec<String> = files
        .iter()
        .map(|(f, _)| read(&repo_root(), &format!("{WINGET}/{ver}/{f}")))
        .collect();
    let schema = yaml(&texts[0], "ManifestVersion");
    for (text, (_, kind)) in texts.iter().zip(&files) {
        assert_eq!(yaml(text, "PackageIdentifier"), ID);
        assert_eq!(yaml(text, "PackageVersion"), *ver);
        assert_eq!(yaml(text, "ManifestVersion"), schema);
        assert_eq!(yaml(text, "ManifestType"), *kind);
    }
    let installer = &texts[1];
    assert!(yaml(installer, "InstallerSha256").eq_ignore_ascii_case(pin));
    assert_eq!(yaml(installer, "InstallerUrl"), url);
    assert_eq!(yaml(installer, "InstallerType"), "nullsoft");
    // the ARP key the NSIS installer writes, measured on the desktop
    // install (HKLM ...CurrentVersion/Uninstall/CodeEraser, 1.5.1, 2026-09-06)
    assert_eq!(yaml(installer, "ProductCode"), "CodeEraser");
    // winget-pkgs wants a BOM for anything beyond ASCII; the files stay ASCII
    assert!(
        texts.iter().all(|t| t.is_ascii()),
        "a winget manifest carries non-ASCII bytes"
    );
    assert_eq!(yaml(&texts[2], "License"), "Apache-2.0");
}
