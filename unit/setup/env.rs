//! The pure halves of setup's environment reads: account identity
//! across the `DOMAIN\name` / `name` spellings (O73), and PATH
//! membership across the spellings one directory can take.

use super::{dir_in, same_user};
use std::ffi::OsString;

#[test]
fn the_same_account_is_read_through_domain_and_case() {
    let rows = [
        (r"DESKTOP\Alice", "alice", true),
        ("alice", "ALICE", true),
        (r"CORP\alice", r"desktop\Alice", true),
        ("bob", "alice", false),
        (r"DESKTOP\Bob", "alice", false),
    ];
    let got: Vec<bool> = rows.iter().map(|(a, b, _)| same_user(a, b)).collect();
    let want: Vec<bool> = rows.iter().map(|r| r.2).collect();
    assert_eq!(got, want);
}

#[test]
fn path_membership_survives_trailing_separators_and_absent_entries() {
    let here = std::env::current_dir().expect("cwd");
    let mut with_slash = here.clone().into_os_string();
    with_slash.push(std::path::MAIN_SEPARATOR_STR);
    let path = std::env::join_paths([OsString::from("no-such-dir-for-setup-env-test"), with_slash])
        .expect("join");
    assert!(dir_in(&here, &path));
    assert!(!dir_in(&here.join("deeper"), &path));
    assert!(!dir_in(&here, &OsString::new()));
}
