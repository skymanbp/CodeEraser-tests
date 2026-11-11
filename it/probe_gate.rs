//! M3 probe core tests: the content gate must find verified T2
//! matches against the index, exclude the file being edited (its
//! indexed version is pre-edit), stay silent below thresholds, and
//! round-trip over the daemon socket.

use codeeraser::corelink::sent_on_this_thread;
use codeeraser::daemon::client;
use codeeraser::daemon::proto::{Request, Response};
use codeeraser::dedup::{Params, index::Index, pairs, probe};
use codeeraser::scan::lang::Lang;
use std::path::{Path, PathBuf};

use crate::common;
use crate::common::{rust_fn, tmp};

fn filter(p: Params) -> pairs::Filter {
    pairs::Filter {
        min_tokens: p.guarantee(),
        min_distinct: pairs::default_min_distinct(),
    }
}

/// Index a.rs on disk in a fresh directory, then probe content that
/// T2-clones it. The completeness stamp is honest — a.rs IS the whole
/// corpus — and it is what keeps the daemon leg schedule-free: an
/// unstamped index sends the daemon into its ADR-003 first build, and a
/// probe racing that build gets the cold-start refusal (CI caught the
/// window).
fn seeded(name: &str) -> (PathBuf, Index) {
    let dir = tmp(name);
    let p = Params::default();
    std::fs::write(dir.join("a.rs"), rust_fn(1)).expect("write a.rs");
    let mut idx = Index::open(&dir.join(".ce/index.db"), p).expect("open");
    idx.refresh_file("a.rs", rust_fn(1).as_bytes(), Lang::Rust, p, false)
        .expect("seed");
    idx.mark_full_build().expect("stamp");
    (dir, idx)
}

/// One probe of `content` as `rel` through the seeded index — the
/// Target/probe stanza used to be pasted per test (three standing
/// dedup-ratchet blocks, repaid with the ADR-008 P3 batch).
fn probed(idx: &Index, dir: &Path, rel: &'static str, content: &[u8]) -> Vec<probe::Match> {
    let p = Params::default();
    let target = probe::Target {
        rel,
        content,
        lang: Lang::Rust,
    };
    probe::probe(idx, dir, target, p, filter(p)).expect("probe")
}

/// A probe of `content` as `rel` in a fresh seeded directory finds
/// nothing — the one shape of every silent case (the self-exclusion, the
/// sub-threshold and the unrelated content used to spell it out each).
fn silent(name: &str, rel: &'static str, content: &[u8], why: &str) {
    let (dir, idx) = seeded(name);
    let m = probed(&idx, &dir, rel, content);
    assert!(m.is_empty(), "{why}, got {m:?}");
}

/// The probe finds the clone — and writes no line to any core (v2.35
/// imports lane, R2: the write-time probe reads its own verified runs;
/// the report path's selection is the core's). The tally is this
/// thread's (corelink::sent_on_this_thread), so no other leg moves it.
/// It is read after `seeded`: the index build has already loaded the
/// definition package, on this thread or another, so the leg asserts no
/// absolute count — only that the probe leaves it where it was, and that
/// the report path (the control) moves it.
#[test]
fn probe_finds_t2_clone_of_indexed_file() {
    let (dir, idx) = seeded("probe-hit");
    let written = sent_on_this_thread();
    // new file whose content T2-clones a.rs (renamed ids, new literals)
    let m = probed(&idx, &dir, "b.rs", &rust_fn(2).into_bytes());
    assert!(!m.is_empty(), "T2 clone of a.rs must be flagged");
    assert_eq!(m[0].file, "a.rs");
    assert!(m[0].tokens >= Params::default().guarantee());
    let probe = "the write-time probe writes no line to a core";
    assert_eq!(sent_on_this_thread(), written, "{probe}");
    std::fs::write(dir.join("b.rs"), rust_fn(2)).expect("write b.rs");
    let found = codeeraser::dedup::analyze(&dir, None, None, None)
        .expect("analyze")
        .0;
    let control = "the report path asks the core";
    assert!(
        !found.blocks.is_empty() && sent_on_this_thread() > written,
        "{control}"
    );
}

#[test]
fn probe_excludes_the_edited_file_itself() {
    // re-writing a.rs with its own (slightly grown) content must not
    // self-flag against the pre-edit indexed version
    let grown = format!(
        "{}
fn extra() -> i64 {{ 42 }}
",
        rust_fn(1)
    );
    silent(
        "probe-self",
        "a.rs",
        grown.as_bytes(),
        "self-match must be excluded",
    );
}

#[test]
fn probe_ignores_short_and_unrelated_content() {
    let tiny = b"fn tiny() -> i64 { 1 }
";
    silent(
        "probe-quiet",
        "c.rs",
        tiny,
        "sub-threshold content stays silent",
    );
    let unrelated = "fn other(a: u8) -> u8 { a.wrapping_add(3) }
"
    .repeat(8);
    let why = "unrelated content stays silent";
    silent("probe-unrelated", "d.rs", unrelated.as_bytes(), why);
}

#[test]
fn probe_over_daemon_socket() {
    let (root, _) = seeded("probe-daemon");
    let child = common::spawn_daemon_ready(&root);
    let req = Request::Probe {
        file_path: root.join("b.rs").display().to_string(),
        content: rust_fn(9),
    };
    match client::request_if_running(&root, &req).expect("probe") {
        Response::ProbeReport {
            matches,
            elapsed_ms,
        } => {
            let arr = matches.as_array().expect("array");
            assert!(!arr.is_empty(), "socket probe must flag the T2 clone");
            assert_eq!(arr[0]["file"], "a.rs");
            println!("socket probe elapsed: {elapsed_ms} ms");
        }
        other => panic!("expected probe report, got {other:?}"),
    }
    // the shared explicit-daemon teardown, Bye asserted — this site
    // was the one weaker hand-rolled copy (batch 9 P13)
    common::shutdown_and_wait(&root, child, "daemon");
}
