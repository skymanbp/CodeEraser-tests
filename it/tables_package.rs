//! The definition package on the measuring side (plan v2.32 step 2;
//! design booklet docs/reference/authority-track.md §4.5): every table
//! `ce` measures with is read off the core's `tables/1`, through a cache
//! file kept beside the identity of the core that answered. Refusals
//! are named and exit 2; a stale or damaged cache is fetched again,
//! never refused; a core whose hello names another digest than the
//! package this run read is refused by name.

use crate::common::{self, ce_triple, stub_core};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// A one-file git tree: enough for `ce scan` to read a language row,
/// a scan table and a judgment.
pub(crate) fn seeded(name: &str) -> PathBuf {
    let dir = common::tmp(name);
    std::fs::write(dir.join("a.py"), "def f(x):\n    return x + 1\n").expect("a.py");
    dir
}

/// `ce scan .` against `core`: (exit code, stderr).
fn scan(dir: &Path, core: &str) -> (Option<i32>, String) {
    let (code, _, err) = ce_triple(dir, &["scan", "."], &[("CE_CORE_BIN", core)]);
    (code, err)
}

fn cache_file(dir: &Path) -> PathBuf {
    let name = format!(
        "tables-{}-{}.json",
        env!("CARGO_PKG_VERSION"),
        codeeraser::corelink::PROTO
    );
    dir.join(".ce").join(name)
}

fn head(dir: &Path) -> Value {
    let text = std::fs::read_to_string(cache_file(dir)).expect("the cache file");
    serde_json::from_str(&text).expect("the cache file reads")
}

/// Rewrite one head field of the cache file, the package bytes as
/// they were (so the file's own fnv still holds).
fn set_head(dir: &Path, key: &str, value: Value) {
    let file = cache_file(dir);
    let text = std::fs::read_to_string(&file).expect("the cache file");
    let old = format!("\"{key}\":{}", head(dir)[key]);
    assert_eq!(text.matches(&old).count(), 1, "{old}");
    std::fs::write(&file, text.replacen(&old, &format!("\"{key}\":{value}"), 1)).expect("write");
}

/// Legs 1, 2 and 5: no core, a core of an older major (refused at the
/// handshake since 8.0.0), a core without `tables/1`, and a package
/// missing a table each refuse by name, exit 2, before anything is
/// measured.
#[test]
fn a_run_without_the_package_is_refused_by_name() {
    let dir = seeded("tables-refusals");
    let mut short = stub_core::real_tables().clone();
    short.as_object_mut().expect("an object").remove("compdb");
    let cores = [
        (
            common::tmp("tables-no-core")
                .join("no-such-core.exe")
                .to_string_lossy()
                .into_owned(),
            "core unavailable",
            "tables/1",
        ),
        (
            stub_core::stub("tables-old-major", "7.10.0", None),
            "proto mismatch",
            "core 7.10.0 vs ce",
        ),
        (
            stub_core::stub("tables-old-core", codeeraser::corelink::PROTO, None),
            "pre-7.7.0 core",
            "no tables/1",
        ),
        (
            stub_core::stub("tables-short", codeeraser::corelink::PROTO, Some(&short)),
            "lacks a table",
            "compdb",
        ),
    ];
    for (core, what, names) in &cores {
        let (code, err) = scan(&dir, core);
        assert_eq!(code, Some(2), "{core}: {err}");
        assert!(err.contains(what) && err.contains(names), "{core}: {err}");
        assert!(!cache_file(&dir).exists(), "no package, no cache: {core}");
    }
}

/// Legs 3 and 4: the cache is written on first read and read back
/// without the core after; a changed core binary, a damaged file and a
/// file another `ce` or proto wrote are fetched again; a hello naming
/// another digest than the package read refuses.
#[test]
fn the_cache_holds_until_its_identity_moves() {
    let dir = seeded("tables-cache");
    let core = common::tmp("tables-cache-core").join(if cfg!(windows) {
        "ce-core.exe"
    } else {
        "ce-core"
    });
    std::fs::copy(common::gates::core_bin(), &core).expect("a core of our own");
    let core = core.to_string_lossy().into_owned();
    assert_eq!(scan(&dir, &core).0, Some(0), "first run");
    let first = std::fs::read(cache_file(&dir)).expect("written");
    let digest = &stub_core::real_tables()["digest"];
    assert_eq!(&head(&dir)["digest"], digest, "the core's own digest");
    let stamp = std::fs::metadata(cache_file(&dir))
        .and_then(|m| m.modified())
        .expect("mtime");
    assert_eq!(scan(&dir, &core).0, Some(0), "second run");
    assert_eq!(
        std::fs::read(cache_file(&dir)).expect("read"),
        first,
        "read back whole"
    );
    let again = std::fs::metadata(cache_file(&dir))
        .and_then(|m| m.modified())
        .expect("mtime");
    assert_eq!(again, stamp, "no fetch rewrote it: the core was not asked");
    let later = SystemTime::now() + Duration::from_secs(86_400);
    // never write-open the core: its exec write-deny can outlive the run
    common::touch::touch_modified(Path::new(&core), later);
    let before = head(&dir)["core"]["mtime_ns"].clone();
    assert_eq!(scan(&dir, &core).0, Some(0), "after the core moved");
    assert_ne!(
        head(&dir)["core"]["mtime_ns"],
        before,
        "fetched again under the new identity"
    );
    for (key, value) in [("ce", "\"0.0.0\""), ("proto", "\"0.0.0\"")] {
        set_head(&dir, key, serde_json::from_str(value).expect("json"));
        assert_eq!(scan(&dir, &core).0, Some(0), "{key} mismatch");
        assert_ne!(
            head(&dir)[key],
            serde_json::json!("0.0.0"),
            "{key}: fetched again"
        );
    }
    std::fs::write(cache_file(&dir), "not a package").expect("damage");
    assert_eq!(scan(&dir, &core).0, Some(0), "damaged file");
    assert_eq!(&head(&dir)["digest"], digest, "damaged file: fetched again");
    let wrong = digest.as_u64().expect("u64").wrapping_add(1);
    set_head(&dir, "digest", serde_json::json!(wrong));
    let (code, err) = scan(&dir, &core);
    assert_eq!(code, Some(2), "{err}");
    assert!(
        err.contains("names tables digest") && err.contains(&wrong.to_string()),
        "{err}"
    );
}
