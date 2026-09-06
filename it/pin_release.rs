//! `scripts/pin_release.js` decides whether a release may be pinned
//! (docs/RELEASE.md §2.1), and its verdict used to depend on WHEN it
//! ran rather than on what the manifest said: it read "does the version
//! move?" from the working file and the moved-line count from `git
//! diff` against HEAD, so a second run over its own uncommitted output
//! expected fifteen lines, was shown seventeen, and refused a correct
//! tree — the retry and the `--bless` re-run every release needs. The
//! judgment now reads the FINAL state, so these legs run it twice.
//!
//! Offline by construction: `CE_PIN_SANDBOX` names a stand-in for the
//! draft (`draft/` holds the asset names and the SHA256SUMS text) plus
//! a disposable copy of `plugin/bin/manifest.env`, and the sibling
//! generators are named instead of run. No network, no `gh`, and the
//! repository's own manifest is never touched.

use crate::common::{node, repo_root, tmp};
use codeeraser::update::manifest;
use codeeraser::update::version::{Platform, TARGETS};
use std::path::{Path, PathBuf};
use std::process::Output;

/// A version no release carries, so a stray sandbox can never be read
/// as a real pin.
const VER: &str = "9.9.9";

/// (asset name, manifest key) per artifact, in SHA256SUMS order —
/// derived from the Rust roster the way scripts/roster.js derives
/// them, and release_roster.rs holds those two spellings together, so
/// this fixture cannot drift on its own.
fn roster() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for key in TARGETS {
        let p = Platform::of_key(key);
        let mk = p.manifest_key();
        let tail = p
            .bundle()
            .expect("every target ships a bundle")
            .manifest_tail();
        out.push((
            format!("ce-{VER}-{key}{}", p.ext),
            format!("CE_SHA256_{mk}_CE"),
        ));
        out.push((
            format!("ce-core-{VER}-{key}{}", p.ext),
            format!("CE_SHA256_{mk}_CECORE"),
        ));
        out.push((
            p.installer_asset(VER).expect("a bundle"),
            format!("CE_SHA256_{mk}_{tail}"),
        ));
    }
    out
}

/// A distinct, well-formed 64-hex pin per artifact.
fn pin(nth: usize) -> String {
    format!("{nth:02x}").repeat(32)
}

/// A stand-in draft beside a disposable copy of the shipped manifest.
fn sandbox(case: &str) -> PathBuf {
    let dir = tmp(case);
    let draft = dir.join("draft");
    std::fs::create_dir_all(&draft).expect("mkdir draft");
    let mut sums = String::new();
    for (nth, (name, _)) in roster().iter().enumerate() {
        std::fs::write(draft.join(name), b"").expect("asset");
        sums.push_str(&format!("{}  {name}\n", pin(nth)));
    }
    std::fs::write(draft.join("SHA256SUMS"), sums).expect("SHA256SUMS");
    let shipped = repo_root().join("plugin/bin/manifest.env");
    std::fs::copy(shipped, dir.join("manifest.env")).expect("manifest copy");
    dir
}

fn run(dir: &Path, args: &[&str]) -> Output {
    let sandbox = dir.to_string_lossy().to_string();
    let argv = [&["scripts/pin_release.js", VER][..], args].concat();
    node(&argv, &[("CE_PIN_SANDBOX", sandbox.as_str())])
}

fn written(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("manifest.env")).expect("manifest")
}

/// stdout when the run succeeded, both streams in the panic when not.
fn said(out: &Output, what: &str) -> String {
    assert!(
        out.status.success(),
        "{what}:\n{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// stderr of a run that had to refuse.
fn refused(out: &Output, what: &str) -> String {
    let err = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(!out.status.success(), "{what} was accepted:\n{err}");
    err
}

#[test]
fn a_first_pin_writes_the_whole_roster_and_an_identical_retry_agrees() {
    let dir = sandbox("pin-release-retry");
    let first = said(&run(&dir, &[]), "the first pin");
    assert!(
        first.contains("line(s) moved"),
        "the first pin said: {first}"
    );
    let after = written(&dir);
    let m = manifest::parse(&after);
    let url = format!("https://github.com/skymanbp/CodeEraser/releases/download/v{VER}");
    assert_eq!(m.get("CE_MANIFEST_VERSION").map(String::as_str), Some(VER));
    assert_eq!(m.get("CE_BASE_URL").map(String::as_str), Some(url.as_str()));
    for (nth, (name, key)) in roster().iter().enumerate() {
        assert_eq!(
            m.get(key).map(String::as_str),
            Some(pin(nth).as_str()),
            "{name}"
        );
    }
    // the same command again, over its own uncommitted output
    let again = said(&run(&dir, &[]), "the identical retry");
    assert!(again.contains("no line moved"), "the retry said: {again}");
    assert_eq!(written(&dir), after, "the retry rewrote the manifest");
}

#[test]
fn the_retry_that_also_blesses_is_not_refused() {
    let dir = sandbox("pin-release-bless");
    said(&run(&dir, &[]), "the first pin");
    let out = said(&run(&dir, &["--bless"]), "the --bless retry");
    assert!(
        out.contains("no line moved"),
        "the --bless retry said: {out}"
    );
    assert!(
        out.contains("ver:pin#v") && out.contains("CE_BLESS=1"),
        "the --bless retry reached the docs-facts step: {out}"
    );
}

#[test]
fn a_draft_whose_assets_are_not_the_roster_is_refused_untouched() {
    let want = format!("assets, expected {}", roster().len() + 1);
    for (case, drop, add) in [
        ("pin-release-missing", format!("ce-{VER}-aarch64-linux"), ""),
        ("pin-release-extra", String::new(), "ce-9.9.9-riscv64-plan9"),
    ] {
        let dir = sandbox(case);
        let pristine = written(&dir);
        if !drop.is_empty() {
            std::fs::remove_file(dir.join("draft").join(&drop)).expect("drop an asset");
        }
        if !add.is_empty() {
            std::fs::write(dir.join("draft").join(add), b"").expect("add an asset");
        }
        let err = refused(&run(&dir, &[]), case);
        assert!(err.contains(&want), "{case} refused with: {err}");
        assert_eq!(
            written(&dir),
            pristine,
            "{case} wrote to the manifest anyway"
        );
    }
}

#[test]
fn a_manifest_carrying_a_pin_key_off_the_roster_is_refused_by_name() {
    let dir = sandbox("pin-release-stray");
    let stray = "CE_SHA256_PPC64_AIX_CE";
    let text = format!("{}\n{stray}=\"\"\n", written(&dir).trim_end());
    std::fs::write(dir.join("manifest.env"), text).expect("stray key");
    let err = refused(&run(&dir, &[]), "the stray key");
    assert!(
        err.contains(&format!("does not pin v{VER}")) && err.contains(stray),
        "refused with: {err}"
    );
}
