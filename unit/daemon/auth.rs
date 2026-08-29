use super::*;

/// Re-establish replaces the previous token wholesale — the
/// fresh-per-serve contract every battery leans on.
#[test]
fn establish_mints_hex_and_replaces_prior_token() {
    let root = crate::testutil::scratch("auth-mint");
    let first = establish(&root).expect("first");
    assert_eq!(first.len(), 64, "32 random bytes, hex");
    let second = establish(&root).expect("second");
    assert_ne!(first, second, "fresh per serve");
    assert_eq!(read(&root), second, "disk holds the latest");
}

/// A symlink squatting on the token path must never route the
/// write: the daemon would truncate the link's TARGET with its
/// own authority (review 2026-08-20 #2). The mint removes the
/// link and writes a regular file; the victim keeps its bytes.
#[cfg(unix)]
#[test]
fn establish_never_writes_through_a_symlink() {
    let root = crate::testutil::scratch("auth-symlink");
    let victim = root.join("victim.txt");
    std::fs::write(&victim, "precious").expect("victim");
    let token_path = root.join(TOKEN_FILE);
    std::fs::create_dir_all(token_path.parent().expect("parent")).expect("mkdir");
    std::os::unix::fs::symlink(&victim, &token_path).expect("plant link");
    establish(&root).expect("establish");
    assert_eq!(
        std::fs::read_to_string(&victim).expect("victim survives"),
        "precious"
    );
    let meta = std::fs::symlink_metadata(&token_path).expect("meta");
    assert!(meta.file_type().is_file(), "regular file, not a link");
}

/// A pre-existing wide-open token file must not keep its mode:
/// truncate-in-place preserved 0644 (review 2026-08-20 #8); the
/// remove + create-exclusive path is 0600 from the first byte.
#[cfg(unix)]
#[test]
fn establish_tightens_a_preexisting_wide_mode() {
    use std::os::unix::fs::PermissionsExt;
    let root = crate::testutil::scratch("auth-mode");
    let token_path = root.join(TOKEN_FILE);
    std::fs::create_dir_all(token_path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&token_path, "stale").expect("seed");
    std::fs::set_permissions(&token_path, std::fs::Permissions::from_mode(0o644)).expect("widen");
    establish(&root).expect("establish");
    let mode = std::fs::metadata(&token_path)
        .expect("meta")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600, "owner-only after re-mint");
}
