//! The data-folder rule over one value: the folder counts only when
//! its last path segment is the plugin's name or that name and `-`,
//! whichever separator spells it and with or without a trailing one.
//! Every path below is a fixture value: nothing is read or written.

use super::own;
use std::ffi::OsStr;
use std::path::PathBuf;

/// The values whose answer is not `want_ours`.
fn misread(values: &[&str], want_ours: bool) -> Vec<String> {
    values
        .iter()
        .filter(|v| own(Some(OsStr::new(v))).is_some() != want_ours)
        .map(|v| (*v).to_string())
        .collect()
}

#[test]
fn the_folder_counts_only_when_its_last_segment_is_the_plugins() {
    let ours = [
        "/srv/plugins/data/codeeraser",
        "/srv/plugins/data/codeeraser-codeeraser",
        "/srv/codeeraser-",
        "/srv/codeeraser/",
        r"D:\plugins\data\codeeraser-codeeraser",
        r"D:\srv\codeeraser\",
        "codeeraser",
    ];
    let theirs = [
        "/srv/plugins/data/codex-openai-codex",
        "/srv/codeeraserx",
        "/srv/codeeraser/other",
        "/srv/my-codeeraser",
        r"D:\srv\codeeraserx",
        r"D:\srv\codeeraser\other\",
        "",
    ];
    assert_eq!(
        misread(&ours, true),
        Vec::<String>::new(),
        "refused though ours"
    );
    assert_eq!(
        misread(&theirs, false),
        Vec::<String>::new(),
        "accepted though not ours"
    );
}

#[test]
fn the_folder_is_returned_as_spelled_and_unset_names_nothing() {
    let native = r"D:\srv\codeeraser-x\";
    assert_eq!(own(Some(OsStr::new(native))), Some(PathBuf::from(native)));
    assert_eq!(own(None), None);
}
