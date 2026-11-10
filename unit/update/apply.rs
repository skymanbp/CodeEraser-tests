use super::*;
use std::process::Command;

/// An independent system SHA-256 implementation, as used by release tooling.
fn system_sha256(path: &Path) -> String {
    let mut cmd = if cfg!(windows) {
        let mut cmd = Command::new("powershell.exe");
        let path = path.to_string_lossy().replace('\'', "''");
        cmd.args(["-NoProfile", "-NonInteractive", "-Command"])
            .arg(format!(
                "(Get-FileHash -LiteralPath '{path}' -Algorithm SHA256).Hash"
            ));
        cmd
    } else {
        let mut cmd = Command::new(if cfg!(target_os = "macos") {
            "shasum"
        } else {
            "sha256sum"
        });
        if cfg!(target_os = "macos") {
            cmd.args(["-a", "256"]);
        }
        cmd.arg(path);
        cmd
    };
    let out = cmd.output().expect("system SHA-256 tool");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .expect("ASCII hash")
        .split_whitespace()
        .next()
        .expect("SHA-256 output")
        .to_ascii_lowercase()
}

#[test]
fn release_asset_keeps_the_pins_spelling() {
    // A release replay supplies the published asset; ordinary cargo test
    // still exercises a binary with the running test executable.
    let asset = std::env::var_os("CE_UPDATE_PIN_ASSET")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_exe().expect("test executable"));
    let pin = system_sha256(&asset);
    assert_eq!(pin.len(), 64);
    assert_eq!(sha256_hex(&asset).expect("asset SHA-256"), pin);
    assert_eq!(hex(&Sha256::digest(std::fs::read(asset).unwrap())), pin);
}

#[test]
fn hex_keeps_leading_zeroes_and_lowercase_digits() {
    assert_eq!(hex(&[0, 1, 15, 16, 127, 128, 255]), "00010f107f80ff");
    assert_eq!(hex(&[]), "");
}

/// v2.35 G9 (R3-39): the two renames are one act — when `ce-core`
/// cannot be placed, the `ce` just placed is rolled back to the copy it
/// retired, and the old core stays where it was.
#[test]
fn a_failed_second_placement_rolls_the_first_back() {
    let dir = crate::testutil::scratch("update-pair");
    let (ce, core) = (dir.join("ce"), dir.join("ce-core"));
    std::fs::write(&ce, "old ce").unwrap();
    std::fs::write(&core, "old core").unwrap();
    let new_ce = dir.join("ce.download.1");
    std::fs::write(&new_ce, "new ce").unwrap();
    let missing = dir.join("ce-core.download.1"); // never fetched
    let err = super::place_pair((&new_ce, &ce), (&missing, &core)).expect_err("no core to place");
    assert!(format!("{err:#}").contains("ce is back"), "{err:#}");
    assert_eq!(std::fs::read_to_string(&ce).unwrap(), "old ce");
    assert_eq!(std::fs::read_to_string(&core).unwrap(), "old core");
    assert!(!dir.join("ce.old").exists() && !dir.join("ce-core.old").exists());

    let new_core = dir.join("ce-core.download.2");
    std::fs::write(&new_ce, "new ce").unwrap();
    std::fs::write(&new_core, "new core").unwrap();
    super::place_pair((&new_ce, &ce), (&new_core, &core)).expect("both land");
    assert_eq!(std::fs::read_to_string(&ce).unwrap(), "new ce");
    assert_eq!(std::fs::read_to_string(&core).unwrap(), "new core");
    let _ = std::fs::remove_dir_all(&dir);
}
