use super::*;
use crate::update::version::Platform;

/// The manifest the plugin ships — the very file ce.sh sources.
fn shipped() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../plugin/bin/manifest.env");
    std::fs::read_to_string(path).expect("plugin/bin/manifest.env")
}

#[test]
fn the_shipped_manifest_parses_as_sh_reads_it() {
    let m = parse(&shipped());
    let ver = &m["CE_MANIFEST_VERSION"];
    assert!(
        crate::update::version::parse(ver).is_some(),
        "CE_MANIFEST_VERSION {ver:?} is not a release version"
    );
    // the tag leg of release.yml asserts the same suffix
    assert!(m["CE_BASE_URL"].ends_with(&format!("/download/v{ver}")));
    // comments never become keys; quotes are stripped exactly once
    assert!(m.keys().all(|k| !k.starts_with('#')));
    assert!(m.values().all(|v| !v.starts_with('"') && !v.ends_with('"')));
}

/// Every roster target's three keys are in the shipped manifest; the
/// targets this manifest's version built carry real pins, the rest
/// (a manifest before FULL_ROSTER_SINCE) are empty and refused by name.
#[test]
fn every_built_target_has_its_three_pins_and_the_rest_are_empty() {
    use crate::update::version::{TARGETS, built};
    let text = shipped();
    let m = parse(&text);
    let ver = m["CE_MANIFEST_VERSION"].clone();
    for key in TARGETS {
        let plat = Platform::of_key(key);
        let mk = plat.manifest_key();
        let tail = plat.bundle().expect("bundle").manifest_tail();
        for k in [
            format!("CE_SHA256_{mk}_CE"),
            format!("CE_SHA256_{mk}_CECORE"),
            format!("CE_SHA256_{mk}_{tail}"),
        ] {
            assert!(m.contains_key(&k), "{key}: the manifest lacks {k}");
        }
        if !built(&ver).contains(&key) {
            let err = pins(&text, &plat).unwrap_err().to_string();
            assert!(err.contains(&format!("CE_SHA256_{mk}_CE")), "{key}: {err}");
            continue;
        }
        let p = pins(&text, &plat).expect("pins");
        for pin in [
            &p.ce,
            &p.ce_core,
            p.installer.as_ref().expect("installer pin"),
        ] {
            assert_eq!(pin.len(), 64, "{key}: {pin}");
            assert!(pin.bytes().all(|b| b.is_ascii_hexdigit()), "{key}: {pin}");
        }
        assert_eq!(
            p.asset_url("ce-core", &plat),
            format!(
                "{}/ce-core-{}-{}{}",
                p.base_url, p.version, plat.key, plat.ext
            )
        );
    }
}

#[test]
fn an_empty_or_missing_pin_is_refused_by_name() {
    let plat = Platform::of("linux", "x86_64");
    let text = concat!(
        "CE_MANIFEST_VERSION=\"9.9.9\"\n",
        "CE_BASE_URL=\"file:///x\"\n",
        "CE_SHA256_X86_64_LINUX_CE=\"\"\n",
        "CE_SHA256_X86_64_LINUX_CECORE=\"abc\"\n",
    );
    let err = pins(text, &plat).unwrap_err().to_string();
    assert!(err.contains("CE_SHA256_X86_64_LINUX_CE"), "{err}");
    let unsupported = pins(text, &Platform::of("freebsd", "x86_64"))
        .unwrap_err()
        .to_string();
    assert!(
        unsupported.contains("CE_SHA256_UNSUPPORTED_CE"),
        "{unsupported}"
    );
    // the installer pin alone may be absent: a platform with no bundle
    let bare = "CE_MANIFEST_VERSION=1.0.0\nCE_BASE_URL=u\nCE_SHA256_X86_64_LINUX_CE=a\nCE_SHA256_X86_64_LINUX_CECORE=b\n";
    let ok = pins(bare, &plat).expect("pins");
    assert_eq!(ok.installer, None);
    assert_eq!(ok.ce, "a");
}
