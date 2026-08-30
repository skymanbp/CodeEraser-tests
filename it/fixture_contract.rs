//! O66 (plan v2.18 step #14, 6.4.0): the golden fixture ledger,
//! DERIVED. VERSIONING §3 spells a hand-written triple — the request
//! anchor, the count of request lines standing at it, the version the
//! server answers — and each consumer (core/test/Spec.hs, this suite)
//! carries its own list of the files. This leg derives the triple
//! from the files and reads the prose, and checks the two lists are
//! one, so a regeneration that moved a count, a golden pair added to
//! one consumer, or a reply line left at an old version fails by
//! name instead of waiting for a human to recount.

use crate::common::repo_root;
use crate::facts::ver::ANCHOR;
use codeeraser::corelink::PROTO;
use std::path::PathBuf;

/// The wire golden files both consumers read, in Spec.hs's order.
pub const GOLDEN_FILES: [&str; 12] = [
    "handshake/hello-ok.ndjson",
    "handshake/wire-errors.ndjson",
    "fourclass/golden.ndjson",
    "graph/golden.ndjson",
    "clone/golden.ndjson",
    "docdup/golden.ndjson",
    "verdict/golden.ndjson",
    "scan/golden.ndjson",
    "structure/golden.ndjson",
    "trend/golden.ndjson",
    "erase/golden.ndjson",
    "audit/golden.ndjson",
];

fn fixture(rel: &str) -> PathBuf {
    repo_root().join("contracts/fixtures").join(rel)
}

fn lines(rel: &str) -> Vec<String> {
    std::fs::read_to_string(fixture(rel))
        .unwrap_or_else(|e| panic!("{rel}: {e}"))
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

fn proto_of(line: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(line)
        .ok()?
        .get("proto")?
        .as_str()
        .map(str::to_string)
}

/// (anchored request lines, reply lines not at PROTO) over the files.
fn tally() -> (usize, Vec<String>) {
    let (mut anchored, mut stale) = (0, Vec::new());
    for rel in GOLDEN_FILES {
        let rows = lines(rel);
        assert_eq!(rows.len() % 2, 0, "{rel}: request/reply pairs");
        for (i, line) in rows.iter().enumerate() {
            let proto = proto_of(line);
            if i % 2 == 1 {
                if proto.as_deref() != Some(PROTO) {
                    stale.push(format!("{rel} pair {}: {proto:?}", i / 2 + 1));
                }
            } else if rel.ends_with("/golden.ndjson") {
                assert_eq!(proto.as_deref(), Some(ANCHOR), "{rel} pair {}", i / 2 + 1);
                anchored += 1;
            }
        }
    }
    (anchored, stale)
}

#[test]
fn every_reply_answers_the_current_proto_and_the_handshake_follows_it() {
    let (_, stale) = tally();
    assert!(stale.is_empty(), "reply lines behind {PROTO}: {stale:?}");
    let hello = lines("handshake/hello-ok.ndjson");
    assert_eq!(
        proto_of(&hello[0]).as_deref(),
        Some(PROTO),
        "the handshake request follows the server (§3)"
    );
}

#[test]
fn the_versioning_triple_is_derived_from_the_files() {
    let (anchored, _) = tally();
    let text =
        std::fs::read_to_string(repo_root().join("contracts/VERSIONING.md")).expect("VERSIONING");
    // the count sits on two lines of the prose; the anchor, the §4
    // row and the `当前` value are chips (facts_chips.rs)
    for want in [
        format!("（{anchored} 行，server 恒答 {PROTO}）"),
        format!("server 走 {PROTO}）"),
    ] {
        assert!(text.contains(&want), "VERSIONING carries {want:?}");
    }
}

#[test]
fn both_consumers_read_the_same_files() {
    let spec = std::fs::read_to_string(repo_root().join("core/test/Spec.hs")).expect("Spec.hs");
    let listed: Vec<&str> = spec
        .lines()
        .filter_map(|l| {
            l.trim().strip_prefix("goldenPairs \"").or_else(|| {
                l.trim()
                    .strip_prefix(", goldenPairs \"")
                    .or_else(|| l.trim().strip_prefix("[ goldenPairs \""))
            })
        })
        .filter_map(|rest| rest.split('"').next())
        .collect();
    assert_eq!(
        listed,
        GOLDEN_FILES.to_vec(),
        "Spec.hs's list is this one, in order"
    );
}
