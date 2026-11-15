//! The SessionStart update notice end to end, over update_e2e's
//! hermetic release index: it rides only when a newer release exists,
//! is cached a day in the plugin's own data folder, and never writes
//! into another plugin's folder (the OS temp dir takes the cache then).

use crate::common::{self, tmp};
use crate::update_e2e::{files_in, release, slashes};
use std::path::Path;

fn health(dir: &Path, env: &[(&str, &str)]) -> String {
    common::session_start_line_env(dir, env).1
}

/// The notice armed against `base`, CLAUDE_PLUGIN_DATA naming `data`.
fn armed<'a>(base: &'a str, data: &'a str) -> [(&'a str, &'a str); 3] {
    [
        ("CE_UPDATE_BASE", base),
        ("CLAUDE_PLUGIN_DATA", data),
        ("CE_UPDATE_CHECK", "1"),
    ]
}

#[test]
fn the_session_notice_rides_only_when_newer_and_is_cached_a_day() {
    let r = release("update-notice", "v9.9.9", false);
    let project = tmp("update-notice-project");
    // the plugin's own folder: its name is the plugin's and a suffix
    let cache = tmp("codeeraser-update-notice");
    let cache_s = slashes(&cache);
    let on = armed(&r.base, &cache_s);
    let ctx = health(&project, &on);
    assert!(
        ctx.starts_with("[ce "),
        "the status line is untouched: {ctx}"
    );
    assert!(ctx.contains("\n[ce update: 9.9.9 available — "), "{ctx}");
    assert!(cache.join("codeeraser-update-check.json").is_file());
    // cached: the index gone, the notice still stands for a day
    std::fs::remove_file(r.dir.join("latest.json")).expect("rm index");
    assert!(health(&project, &on).contains("9.9.9 available"));
    // the harness default (off) and an explicit opt-out say nothing
    assert!(!health(&project, &on[..2]).contains("ce update:"));
    // a current binary says nothing either — and its own cache
    let same = release(
        "update-notice-same",
        &format!("v{}", env!("CARGO_PKG_VERSION")),
        false,
    );
    let cache2 = slashes(&tmp("codeeraser-update-notice2"));
    let quiet = health(&project, &armed(&same.base, &cache2));
    assert!(!quiet.contains("ce update:"), "{quiet}");
}

/// Another plugin's folder is not this plugin's: the notice still
/// rides, its cache lands in the OS temp dir (kept in scratch here:
/// std's temp dir reads TMP / TEMP on Windows, TMPDIR elsewhere), and
/// nothing is written into the other folder.
#[test]
fn the_notice_cache_never_lands_in_another_plugins_folder() {
    let r = release("update-notice-foreign", "v9.9.9", false);
    let project = tmp("update-notice-foreign-project");
    let foreign = tmp("update-notice-foreign-data");
    let temp = tmp("update-notice-foreign-temp");
    let (foreign_s, temp_s) = (slashes(&foreign), slashes(&temp));
    let mut env = armed(&r.base, &foreign_s).to_vec();
    env.extend([
        ("TMP", temp_s.as_str()),
        ("TEMP", &temp_s),
        ("TMPDIR", &temp_s),
    ]);
    assert!(health(&project, &env).contains("9.9.9 available"));
    assert_eq!(
        files_in(&foreign),
        0,
        "nothing written into another plugin's folder"
    );
    assert!(temp.join("codeeraser-update-check.json").is_file());
}
