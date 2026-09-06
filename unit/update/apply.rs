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
