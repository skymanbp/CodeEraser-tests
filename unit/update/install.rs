use super::*;
use crate::testutil::scratch;

/// One table of locations, each read for the ledger beside it.
#[test]
fn the_owner_is_read_off_the_binarys_own_location() {
    let plain = scratch("update-install-plain");
    let exe = plain.join("ce");
    std::fs::write(&exe, b"").expect("exe");
    assert_eq!(classify(&exe, None), Kind::Manual);
    // the plugin's data dir, however the two paths are spelled
    assert_eq!(classify(&exe, Some(&plain)), Kind::Plugin);
    let other = scratch("update-install-other");
    assert_eq!(classify(&exe, Some(&other)), Kind::Manual);
    // the starter's naming outside its data dir is still the starter's
    let bound = plain.join(format!("ce-{}-x86_64-linux", env!("CARGO_PKG_VERSION")));
    assert_eq!(classify(&bound, None), Kind::Plugin);
    // cargo's bin dir
    let cargo = scratch("update-install-cargo").join(".cargo").join("bin");
    std::fs::create_dir_all(&cargo).expect("cargo bin");
    assert_eq!(classify(&cargo.join("ce"), None), Kind::Cargo);
    // the bundle: the app sits beside us
    std::fs::write(plain.join("CodeEraser.exe"), b"").expect("app");
    assert_eq!(classify(&exe, None), Kind::Bundle);
    // the data-dir rule outranks the bundle rule
    assert_eq!(classify(&exe, Some(&plain)), Kind::Plugin);
}

/// The wire codes are frozen (ce.update-report `current.install`).
#[test]
fn the_install_codes_are_frozen() {
    assert_eq!(
        [
            Kind::Manual as u8,
            Kind::Bundle as u8,
            Kind::Cargo as u8,
            Kind::Plugin as u8
        ],
        [0, 1, 2, 3]
    );
}
