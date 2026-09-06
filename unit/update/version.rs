use super::*;

#[test]
fn the_release_grammar_parses_three_numbers_and_nothing_else() {
    assert_eq!(parse("1.2.0"), Some((1, 2, 0)));
    assert_eq!(parse(" 10.0.7 "), Some((10, 0, 7)));
    for bad in ["1.2", "1.2.0.1", "v1.2.0", "1.2.x", ""] {
        assert_eq!(parse(bad), None, "{bad:?}");
    }
    // tuple order IS release order, and a version outside the
    // grammar is never newer than a real one
    assert!(parse("1.3.0") > parse("1.2.9"));
    assert!(parse("1.10.0") > parse("1.9.9"));
    assert!(parse("garbage") < parse("0.0.1"));
}

#[test]
fn a_tag_sheds_its_v_once() {
    assert_eq!(of_tag("v1.3.0"), "1.3.0");
    assert_eq!(of_tag("1.3.0"), "1.3.0");
    assert_eq!(of_tag("vv1"), "v1");
}

/// The key grammar release.yml stages under and ce.sh's plat_key
/// derives — one table, every cell the workflow's own spelling; the
/// two targets O70 added ship a bundle like the first three.
#[test]
fn the_platform_key_matches_the_release_matrix() {
    let cases = [
        (
            "windows",
            "x86_64",
            "x86_64-windows",
            ".exe",
            "X86_64_WINDOWS",
            Some("CodeEraser-1.3.0-x86_64-windows-setup.exe"),
        ),
        (
            "linux",
            "x86_64",
            "x86_64-linux",
            "",
            "X86_64_LINUX",
            Some("CodeEraser-1.3.0-x86_64-linux.AppImage"),
        ),
        (
            "macos",
            "aarch64",
            "aarch64-macos",
            "",
            "AARCH64_MACOS",
            Some("CodeEraser-1.3.0-aarch64-macos.dmg"),
        ),
        (
            "macos",
            "x86_64",
            "x86_64-macos",
            "",
            "X86_64_MACOS",
            Some("CodeEraser-1.3.0-x86_64-macos.dmg"),
        ),
        (
            "linux",
            "aarch64",
            "aarch64-linux",
            "",
            "AARCH64_LINUX",
            Some("CodeEraser-1.3.0-aarch64-linux.AppImage"),
        ),
        // built by no matrix leg: a key with no pins and no bundle
        ("freebsd", "x86_64", "unsupported", "", "UNSUPPORTED", None),
    ];
    for (os, arch, key, ext, mkey, asset) in cases {
        let p = Platform::of(os, arch);
        assert_eq!((p.key, p.ext, p.manifest_key().as_str()), (key, ext, mkey));
        assert_eq!(p.installer_asset("1.3.0").as_deref(), asset, "{key}");
    }
}

/// The roster is closed under its own derivations — every key round
/// trips through (os, arch), ships a bundle, spells its manifest key —
/// and a manifest before FULL_ROSTER_SINCE pins the first three only.
#[test]
fn the_roster_round_trips_and_widens_at_one_version() {
    for key in TARGETS {
        let (arch, os) = key.rsplit_once('-').expect("arch-os");
        let p = Platform::of(os, arch);
        assert_eq!(p.key, key);
        assert!(p.bundle().is_some(), "{key}");
        assert_eq!(p.manifest_key(), key.to_ascii_uppercase().replace('-', "_"));
    }
    assert!(Platform::of_key("unsupported").bundle().is_none());
    assert_eq!(built("1.6.0"), &TARGETS[..3]);
    assert_eq!(built(FULL_ROSTER_SINCE), &TARGETS);
    assert_eq!(built("2.0.0"), &TARGETS);
    assert_eq!(built("garbage"), &TARGETS[..3]);
}
