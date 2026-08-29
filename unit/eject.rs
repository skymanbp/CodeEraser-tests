use super::{released, run};
use interprocess::local_socket::traits::ListenerExt;
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
use std::path::{Path, PathBuf};

/// A daemon that holds the socket and answers nothing — what the
/// token-write race looks like from the client: up, but not (yet)
/// speaking for us. It accepts WITHOUT bound and is never joined:
/// a listener that stopped early would unbind the socket, and a
/// test whose stub disappears mid-run is a test that green-lights
/// the very deletion it exists to forbid.
fn deaf_daemon(root: &Path) {
    let ns = crate::daemon::proto::socket_name(root)
        .to_ns_name::<GenericNamespaced>()
        .expect("name");
    let listener = ListenerOptions::new().name(ns).create_sync().expect("bind");
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            drop(stream); // accepted, then hung up
        }
    });
}

/// A project with its own anchor, so `targets` resolves here and
/// the stray walk stays inside the scratch dir.
fn seeded(tag: &str) -> PathBuf {
    let root = crate::testutil::scratch(tag);
    std::fs::create_dir_all(root.join(".ce")).expect(".ce");
    for (name, body) in [("ce.toml", ""), ("ce-baseline.json", "{}")] {
        std::fs::write(root.join(name), body).expect(name);
    }
    root
}

/// --yes must remove NOTHING while a daemon still holds the
/// socket: discarding the shutdown's outcome let eject delete .ce
/// out from under a live daemon (review 2026-08-20 #7). State is
/// the assertion — ExitCode has no PartialEq to compare.
#[test]
fn yes_keeps_every_target_while_a_daemon_holds_the_socket() {
    let root = seeded("eject-live-daemon");
    deaf_daemon(&root);
    // gate first: if it ever opens, the run below is the delete
    assert!(!released(&root), "an unanswered shutdown never releases");
    let _ = run(&root, true);
    assert!(root.join(".ce").exists(), "cache kept for the live daemon");
    assert!(root.join("ce-baseline.json").exists(), "baseline kept");
}

/// ...and the gate does not block the cold path it was added to
/// protect: no socket, nobody to wait for. (Asserted through
/// `released`, not `run`: a --yes run here would also sweep this
/// process's real CLAUDE_PLUGIN_DATA.)
#[test]
fn a_root_with_no_daemon_is_released() {
    let root = crate::testutil::scratch("eject-no-daemon");
    assert!(released(&root), "an unheld socket releases at once");
}
