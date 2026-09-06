//! One release roster (plan v2.29 step 10, O70): `update::version::TARGETS`
//! is the typed spelling — the manifest's key set, release.yml's build
//! matrix and verify loop, ci.yml's rehearsal matrix, scripts/roster.js
//! and bootstrap_e2e's key table are hosts that cannot read it, so each
//! is held to it here and a target lands in every reader or in none.
//! The BUNDLE table (asset suffix and manifest tail per os) is spelled
//! three more times — roster.js, release.yml's `bundle()` and
//! build-target.yml's staging case — and is held the same way.
//! The tag gate's two claims about ci.yml are held the same way, both
//! read off one parse of that file (release_roster_parts): the checks
//! that MAY skip are its schedule-only jobs, and the checks that MUST
//! be present and green are its unconditional ones.

use crate::common::repo_root;
use crate::facts::read;
use crate::release_roster_parts::{quoted_list, schedule_only_jobs, unconditional_checks};
use codeeraser::update::version::{Platform, TARGETS};
use std::collections::{BTreeMap, BTreeSet};

/// The `key: <roster key>` values in a workflow's matrix rows (flow or
/// block style); cache keys and the like never end in an os word.
fn matrix_keys(yml: &str) -> Vec<String> {
    yml.lines()
        .filter_map(|l| l.split("key: ").nth(1))
        .map(|v| v.split([',', '}']).next().unwrap_or("").trim().to_string())
        .filter(|v| {
            ["-windows", "-linux", "-macos"]
                .iter()
                .any(|os| v.ends_with(os))
        })
        .collect()
}

#[test]
fn the_manifest_carries_exactly_the_roster_keys() {
    let m = codeeraser::update::manifest::parse(&read(&repo_root(), "plugin/bin/manifest.env"));
    let got: BTreeSet<String> = m
        .keys()
        .filter(|k| k.starts_with("CE_SHA256_"))
        .cloned()
        .collect();
    let want: BTreeSet<String> = TARGETS
        .iter()
        .flat_map(|k| {
            let p = Platform::of_key(k);
            let mk = p.manifest_key();
            let tail = p
                .bundle()
                .expect("every target ships a bundle")
                .manifest_tail();
            [
                format!("CE_SHA256_{mk}_CE"),
                format!("CE_SHA256_{mk}_CECORE"),
                format!("CE_SHA256_{mk}_{tail}"),
            ]
        })
        .collect();
    assert_eq!(got, want);
}

#[test]
fn every_host_spells_the_roster() {
    let want: Vec<String> = TARGETS.iter().map(|k| k.to_string()).collect();
    let release = read(&repo_root(), ".github/workflows/release.yml");
    assert_eq!(matrix_keys(&release), want, "release.yml build matrix");
    assert!(
        release.contains(&format!("TARGETS=\"{}\"", want.join(" "))),
        "release.yml verify loop spells the roster in order"
    );
    assert_eq!(
        matrix_keys(&read(&repo_root(), ".github/workflows/ci.yml")),
        want,
        "ci.yml release-rehearsal matrix"
    );
    let js = read(&repo_root(), "scripts/roster.js");
    let array = js
        .split("const TARGETS = [")
        .nth(1)
        .expect("roster.js declares TARGETS")
        .split(']')
        .next()
        .expect("closing bracket");
    let got: Vec<String> = array
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();
    assert_eq!(got, want, "scripts/roster.js");
    let e2e = read(&repo_root(), ".github/scripts/bootstrap_e2e.sh");
    for key in TARGETS {
        assert!(
            e2e.contains(&format!("\"{key}\"")),
            "bootstrap_e2e.sh lacks {key}"
        );
    }
}

/// `os -> (asset suffix, manifest key tail)` from the roster's own
/// keys — Bundle is the only place that decides, and every os the
/// roster reaches appears exactly once.
fn bundles() -> BTreeMap<&'static str, (&'static str, &'static str)> {
    TARGETS
        .iter()
        .map(|k| {
            let b = Platform::of_key(k)
                .bundle()
                .expect("every target ships a bundle");
            let os = k.rsplit_once('-').expect("arch-os").1;
            (os, (b.suffix(), b.manifest_tail()))
        })
        .collect()
}

/// A wrong suffix in any host that cannot read `Bundle` names a
/// release asset nobody built — build-target.yml would stage it, the
/// draft carry it and pin_release.js refuse it, three jobs later.
#[test]
fn every_host_spells_the_same_bundle_per_os() {
    let js = read(&repo_root(), "scripts/roster.js");
    let release = read(&repo_root(), ".github/workflows/release.yml");
    let target = read(&repo_root(), ".github/workflows/build-target.yml");
    for (os, (suffix, tail)) in bundles() {
        for (what, hay, needle) in [
            (
                "scripts/roster.js BUNDLE",
                &js,
                format!("{os}: [\"{suffix}\", \"{tail}\"]"),
            ),
            (
                "release.yml bundle()",
                &release,
                format!("*-{os}) echo \"{suffix} {tail}\" ;;"),
            ),
            (
                "build-target.yml staging",
                &target,
                format!("dist/CodeEraser-$VER-$TARGET_KEY{suffix}\""),
            ),
        ] {
            assert!(hay.contains(&needle), "{what} lacks {needle:?} for {os}");
        }
    }
}

#[test]
fn the_tag_gate_excuses_every_schedule_only_job_by_name() {
    let mut want = schedule_only_jobs(&read(&repo_root(), ".github/workflows/ci.yml"));
    assert!(
        want.contains("setup-wiring") && want.contains("release-rehearsal"),
        "the scan found {want:?}"
    );
    // release.yml's own dispatch-phase jobs skip on the tag push too
    want.insert("build".into());
    want.insert("draft".into());
    let release = read(&repo_root(), ".github/workflows/release.yml");
    assert_eq!(quoted_list(&release, "SKIPPED_OK", ' '), want);
}

/// The other half of the same contract (plan v2.29 step 12, item 12b).
/// Excusing the skipped legs by name only says what MAY be absent; it
/// never said what must be PRESENT, and a commit whose checks have not
/// been created yet answers with an empty list — nothing pending,
/// nothing failed, "all checks green" on no evidence at all. The tag
/// gate now requires ci.yml's unconditional jobs by their surfaced
/// names, and this holds that list to ci.yml's own job keys and
/// matrix, so a renamed runner cannot leave the gate waiting on a
/// ghost. Verified against run 34040780731 / commit 02372fa, whose
/// seven check runs are `build (ubuntu-latest)`,
/// `build (windows-latest)`, `build-macos` and the four schedule-only
/// legs, all skipped.
#[test]
fn the_tag_gate_requires_every_unconditional_ci_check_by_name() {
    let want = unconditional_checks(&read(&repo_root(), ".github/workflows/ci.yml"));
    assert!(
        want.contains("build-macos") && want.iter().any(|n| n.starts_with("build (")),
        "the scan found {want:?}"
    );
    let release = read(&repo_root(), ".github/workflows/release.yml");
    assert_eq!(quoted_list(&release, "REQUIRED", '|'), want);
}
